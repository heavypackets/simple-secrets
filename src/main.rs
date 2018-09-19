extern crate iron;
extern crate router;
extern crate etcd;
extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate argonautica;

use iron::prelude::*;
use iron::headers::*;
use router::Router;
use etcd::kv::{self};
use futures::Future;
use tokio_core::reactor::Core;

use std::error::Error;

fn main() {
    let context = context();

    let mut router = Router::new();
    router.get("/login", move |request: &mut Request| login(request, &context), "login");
    router.get("/get/:name", fetch_secret, "get_secret");
    router.post("/set/:name/:value", set_secret, "set_secret");

    Iron::new(router).http("localhost:3000").unwrap();
}

#[derive(Debug, Default)]
struct Context {
    etcd_hosts: String
}

fn context() -> Context {
    let mut context = Context::default();
    context.etcd_hosts = match std::env::var("ETCD_CLUSTER_MEMBERS") {
        Ok(val) => val,
        Err(_) => String::from("http://localhost:2379")
    };
    context
}

fn new_etcd_client(core: &Core, opts: &Context) -> Result<etcd::Client<hyper::client::HttpConnector>, etcd::Error> {
    let handle = core.handle();
    etcd::Client::new(&handle, 
        opts.etcd_hosts.split(",").collect::<Vec<&str>>().as_slice(),
        None)
}

#[derive(Debug, Default)]
struct UserInfo {
    username: String,
    password: String,
    id: String,
    encoded_password: String,
}

fn fetch_user_info(user_info: &mut UserInfo, context: &Context) -> Result<(), Box<Error>> {
    let mut core = Core::new()?;
    {
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

                }
                // println!("{:?}", user_info); 
            } else {
                user_info.encoded_password = String::from("");
                user_info.id = String::from("-1");
            }

            Ok(())
        });
        
        if let Err(_) = core.run(fetched_user)
        {
            Err("User not found")?;
        }
    }

    // Check result of fetch
    if user_info.id == "" {
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
    if verify_password(&user_info)
    {
        Ok(Response::with(iron::status::Ok))
    } else {
        println!("Invalid password");
        Ok(Response::with(iron::status::Unauthorized))
    }

}

fn set_secret(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}

fn fetch_secret(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}
