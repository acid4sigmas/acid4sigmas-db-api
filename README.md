# acid4sigmas-db-api
the websocket "hybrid" database api of acid4sigmas.systems

## What is a hybrid database api?
In this project a hybrid database api is a mix out of manual and automatical database management which is exposed through and api backend
This is why i decided to call it a hybrid database api, because some things like table definitions are predefined through a schema.sql file.

## What is possible right now?
- websocket connection
- predefined table schema
- insert values into a table
- receive values from a table (**potentially unstable**)


## What is expected?
- updating values in a table
- deleting values in a table
- filters for receiving values
- conditions for deleting/updating values in a table
- proper authentication


## Get started.

You will need
- Rust language
- PostgreSQL
- and optionally postman for testing your websocket or any other websocket client.

**prepare your Secrets.toml**

```toml
DB_NAME="your-database"
DB_PW="your-password"
DB_PORT="5432" # default port, adjust to your needs
```

after that make sure postgreSQL is running and start the acid4sigmas-db-api in a termainl with `cargo run`

connect to the websocket via the following url
`ws://127.0.0.1:3453/db?token=secret-token-shhh`
and then try sending a message to it 
expected syntax
```json
{
  "table": "users",
  "action": "Retrieve"
}
```
or for inserting
```json
{
  "table": "users",
  "action": "Insert",
  "values": {
    "email":"sdsd@ad.cd",
    "email_verified":false,
    "owner":true,
    "uid":3243294239,
    "username": "skibidi4343"
  }
}
```

tutorial about using your own tables and structs will come sooner or later! please be patient 



