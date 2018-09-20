extern crate iron;
extern crate router;
extern crate etcd;
extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate argonautica;
extern crate rand;
extern crate uuid;

use iron::prelude::*;
use iron::headers::*;
use router::Router;
use etcd::kv::{self};
use futures::Future;
use tokio_core::reactor::Core;

use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use std::error::Error;

fn main() {
    let context = context();

    let mut router = Router::new();
    {
        let context = context.clone();
        router.get("/login", move |request: &mut Request| login(request, &context), "login");
    } 
    router.get("/get/:name", fetch_secret, "get_secret");
    {
        let context = context.clone();
        router.post("/set/:name/:value", move |request: &mut Request| set_secret(request, &context), "set_secret");
    }

    Iron::new(router).http("localhost:3000").unwrap();
}

#[derive(Debug, Clone)]
struct Context {
    etcd_hosts: String,
    token_expiration_secs: u64
}

impl Default for Context {
    fn default() -> Context {
        Context {
            etcd_hosts: String::from("http://localhost:2379"),
            token_expiration_secs: 600
        }
    }
}

fn context() -> Context {
    let mut context = Context::default();
    if let Ok(val) = std::env::var("ETCD_CLUSTER_MEMBERS") {
        context.etcd_hosts = val; 
    }
    if let Ok(val) = std::env::var("TOKEN_EXPIRATION_SECS") {
        context.token_expiration_secs = str::parse::<u64>(val.as_str()).unwrap_or(600); 
    }

    context
}

fn new_etcd_client(core: &Core, opts: &Context) -> Result<etcd::Client<hyper::client::HttpConnector>, etcd::Error> {
    let handle = core.handle();
    etcd::Client::new(&handle, 
        opts.etcd_hosts.split(",").collect::<Vec<&str>>().as_slice(),
        None)
}

type AuthToken = String;

#[derive(Debug, Default)]
struct UserInfo {
    username: String,
    password: String,
    id: String,
    encoded_password: String,
    token: AuthToken,
}

fn fetch_user_info(user_info: &mut UserInfo, context: &Context) -> Result<(), Box<Error>> {
    let mut core = Core::new()?;
    let client = match new_etcd_client(&core, &context) {
        Ok(client) => client,
        Err(_) => Err("Unable to create etcd client")?
    };

    let fetched_user = kv::get(&client, format!("/users/{}", user_info.username).as_str(), kv::GetOptions {recursive: true, ..kv::GetOptions::default()}).and_then(|response| {
        if let Some(user_nodes) = response.data.node.nodes {
            for node in user_nodes {
                let key = node.key.unwrap_or("".to_string());
                let value = node.value.unwrap_or("".to_string());
                // println!("{}: {}", key, value);

                if key == format!("/users/{}/password", user_info.username) 
                { 
                    user_info.encoded_password = value;
                } 
                else if key == format!("/users/{}/id", user_info.username)
                {
                    user_info.id = value;
                }
                else if key == format!("/users/{}/token", user_info.username)
                {
                    user_info.token = value;
                }
            }
            // println!("{:?}", user_info); 
        } else {
            user_info.encoded_password = String::from("");
            user_info.id = String::from("-1");
        }

        Ok(())
    });
    
    if let Err(e) = core.run(fetched_user)
    {
        println!("{:?}", e);
        Err("Cannot fetch user information")?;
    }

    Ok(())
}

fn verify_password(user_info: &UserInfo) -> bool {
    let mut verifier = argonautica::Verifier::default();
    if let Ok(true) = verifier
        .with_hash(&user_info.encoded_password)
        .with_password(&user_info.password)
        .verify()
    {
       true
    } else {
        false
    }
}

fn login(req: &mut Request, context: &Context) -> IronResult<Response> {
    // Parse username and password from request
    let auth = match req.headers.get::<Authorization<Basic>>() {
        Some(auth) => auth,
        None => return Ok(Response::with(iron::status::Unauthorized))
    };

    let mut user_info = UserInfo::default();
    user_info.username = auth.username.clone();
    user_info.password = match auth.password.clone() {
        Some(password) => password,
        None  => return Ok(Response::with(iron::status::Unauthorized))
    };
    
    // Fetch user information from etcd
    if let Err(e) = fetch_user_info(&mut user_info, &context) {
        println!("{}", e);
        return Ok(Response::with(iron::status::Unauthorized))
    }

    // Check password
    if !verify_password(&user_info)
    {
        println!("Invalid password");
        return Ok(Response::with(iron::status::Unauthorized))
    }

    // Generate and set new token
    user_info.token = generate_authorization_token();
    if let Ok(_) = update_user_token(&user_info, &context) {
        Ok(Response::with((iron::status::Ok, user_info.token)))
    } else {
        println!("Unable to update user token");
        Ok(Response::with(iron::status::InternalServerError))
    }    
}

fn generate_authorization_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(24)
        .collect()
}

fn update_user_token(user_info: &UserInfo, context: &Context) -> Result<(), Box<Error>> {
    let mut core = Core::new()?;
    let client = match new_etcd_client(&core, &context) {
        Ok(client) => client,
        Err(_) => Err("Unable to create etcd client")?
    };
    let set_token = kv::set(&client, format!("/session_tokens/{}", user_info.token).as_str(), user_info.username.as_str(), Some(context.token_expiration_secs));
    core.run(set_token).or(Err(format!("Unable to update etcd token value for user {}", user_info.username)))?;
    
    Ok(())
}

fn set_secret(req: &mut Request, context: &Context) -> IronResult<Response> {
    // Parse name/value from URL
    let args;
    
    match req.extensions.get::<Router>()
    {
        Some(params) => args = (params.find("name").unwrap_or(""), params.find("value").unwrap_or("")),
        None => return Ok(Response::with(iron::status::BadRequest))
    };
    
    // Validate token
    let token = match req.url.query() {
        Some(val) => val.replace("token=", ""),
        None => return Ok(Response::with(iron::status::BadRequest))
    };
    if let Err(e) = validate_token(&token, &context) {
        println!("{}", e);
        return Ok(Response::with((iron::status::Unauthorized, "Bad token")));
    }
    // println!("{} {} {}", args.0, args.1, token);

    // Set secret
    let uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, args.0.as_bytes()); // Use secret name to gen SHA1-based UUID
    if let Err(e) = set_etcd_key(&format!("/secrets/{}/name", uuid), args.0, &context, None) {
        println!("{}", e);
        return Ok(Response::with(iron::status::InternalServerError));
    }
    if let Err(e) = set_etcd_key(&format!("/secrets/{}/value", uuid), args.1, &context, None) {
        println!("{}", e);
        return Ok(Response::with(iron::status::InternalServerError));
    }
    
    Ok(Response::with((iron::status::Ok, format!("{}", uuid))))
}

fn set_etcd_key(key: &str, value: &str, context: &Context, expiration: Option<u64>) -> Result<(), Box<Error>> {
    let mut core = Core::new()?;
    let client = match new_etcd_client(&core, &context) {
        Ok(client) => client,
        Err(_) => Err("Unable to create etcd client")?
    };

    let set_token = kv::set(&client, key, value, expiration);
    core.run(set_token).or(Err(format!("Unable to update etcd key {}", key)))?;

    Ok(())
}

fn validate_token(token: &str, context: &Context) -> Result<(), Box<Error>> {
    let mut core = Core::new()?;
    let client = match new_etcd_client(&core, &context) {
        Ok(client) => client,
        Err(_) => Err("Unable to create etcd client")?
    };

    let fetch_token = kv::get(&client, &format!("/session_tokens/{}", token), kv::GetOptions::default());
    core.run(fetch_token).or(Err(format!("Token {} not found", token)))?;
    
    Ok(())
}

fn fetch_secret(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}
