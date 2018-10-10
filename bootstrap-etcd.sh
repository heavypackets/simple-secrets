#!/bin/bash

curl http://127.0.0.1:2379/v2/keys/users/user1/password -XPUT --data value='%24argon2i%24v%3D19%24m%3D4096%2Ct%3D3%2Cp%3D1%24c2FsdHlzcGl0dG9vbg%24aegHILhzY1FqoI%2FTbL%2BMBjngofmCpF2CGwY4bL%2FonFk'
