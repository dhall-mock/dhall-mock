# Dhall mock server
![build](https://github.com/dhall-mock/dhall-mock/workflows/build/badge.svg?branch=master)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/dhall-mock/dhall-mock)

## Project

This project aims to provide a modern HTTP mock server for your daily project using [Dhall lang](https://github.com/dhall-lang/dhall-lang) configuration to describe mocks.

### Goal

Mocking http server can be painful. 
The installation and the configuration of servers and their responses could be handy and could lead you to use templates generations for some complex cases.

Dhall-mock project aim to be **quickly installable** as a standalone binary and **extensive** by using an external language to describe the mock configurations.

### Why dhall

We choose to base our configuration on [Dhall lang](https://github.com/dhall-lang/dhall-lang) for multiple reasons but here are the most important ones :

 - Complete **functional language** - Using a complete language empowers you, you could as easily define static responses as create a set of functions that will generate complex responses.
 - **Typed language** - you can compile and verify your configuration without running dhall-mock server. As soon as your configuration compile and expose configuration with the good typing it can be integrated.
 - **No side effects** - Use a real programming language with libraries, living ecosystem, could define complex configuration pipelines and be safe to integrate any configuration file.
 - **We wanted to** - Most importantly, we wanted to use dhall because we like this language and wanted to use it :wink: 

## Usage

### Install

**From release** ![GitHub release (latest by date)](https://img.shields.io/github/v/release/dhall-mock/dhall-mock)

Download your distribution binary in [release page](https://github.com/dhall-mock/dhall-mock/releases/latest), add it to you path and your are good to go :thumbsup: 

**Build from sources** :
```bash
> git clone git@github.com:dhall-mock/dhall-mock.git
> cd dhall-mock
> cargo build --release
> ./target/release/main --help
```

### Command line

```bash
> dhall-mock
dhall-mock 0.0.1

USAGE:
    main [FLAGS] [OPTIONS] [configuration-files]...

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information
    -w, --wait       wait to compile all configuration files before starting web servers

OPTIONS:
    -a, --admin-http-bind <admin-http-bind>    http binding for admin server [default: 0.0.0.0:8089]
    -h, --http-bind <http-bind>                http binding for server [default: 0.0.0.0:8088]

ARGS:
    <configuration-files>...    Dhall configuration files to parse

```

### Getting started

Create a static configuration file `static.dhall` :
```dhall
let Mock = https://raw.githubusercontent.com/dhall-mock/dhall-mock/master/dhall/Mock/package.dhall

let expectations = [
                       { request  = Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                                                      , path    = Some "/greet/pwet"
                                                      }
                       , response = Mock.HttpResponse::{ statusCode   = Mock.statusCreated
                                                       , body         = Some "Hello, pwet !"
                                                       }
                      }
                      ,{ request  = Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                                                      , path    = Some "/greet/wololo"
                                                      }
                      , response = Mock.HttpResponse::{ statusCode   = Mock.statusOK
                                                      , body         = Some "Hello, Wololo !"
                                                      }
                      }
                   ]

in expectations
```

If you have [dhall](https://docs.dhall-lang.org/howtos/How-to-integrate-Dhall.html#external-executable) installed on your machine you can test the file by running `dhall resolve --file static.dhall`
```bash
> dhall  resolve --file static.dhall
let Mock =
      { Body = < JSON : { json : Text } | TEXT : { text : Text } >
      ...

let expectations =
      [ { request = Mock.HttpRequest::{
          , method = Some Mock.HttpMethod.GET
          , path = Some "/greet/pwet"
          }
        , response = Mock.HttpResponse::{
          , statusCode = Mock.statusCreated
          , body = Some "Hello, pwet !"
          }
        }
      , { request = Mock.HttpRequest::{
          , method = Some Mock.HttpMethod.GET
          , path = Some "/greet/wololo"
          }
        , response = Mock.HttpResponse::{
          , statusCode = Mock.statusOK
          , body = Some "Hello, Wololo !"
          }
        }
      ]

in  expectations
```

Start the server using this configuration 
```bash
> dhall-mock static.dhall
[2020-05-06T19:52:10Z INFO  main] Start dhall mock project 👋
[2020-05-06T19:52:10Z INFO  dhall_mock::mock::service] Start load static.dhall config
[2020-05-06T19:52:10Z INFO  dhall_mock::web::mock] Http server started on http://0.0.0.0:8088
[2020-05-06T19:52:10Z INFO  dhall_mock::web::admin] Admin server started on http://0.0.0.0:8089
[2020-05-06T19:52:10Z INFO  dhall_mock::mock::service] Loaded static.dhall, in 0 secs
[2020-05-06T19:52:10Z INFO  main] Configuration static.dhall loaded
```

Try it out ! :rocket: 
```bash
> curl http://localhost:8088/greet/pwet
Hello, pwet !
> curl http://localhost:8088/greet/wololo
Hello, Wololo !
```

### Admin server

Admin server allows you to know which configurations are available on the server and create new configurations.

#### `GET /expectations` 

Will return all the current mock configured in the servers.

Example :
```bash
> curl http://localhost:8089/expectations | jq
[
  {
    "request": {
      "method": "GET",
      "path": "/greet/pwet",
      "body": null,
      "params": [],
      "headers": {}
    },
    "response": {
      "statusCode": 201,
      "statusReason": null,
      "body": "Hello, pwet ! Comment que ca biche ?",
      "headers": {}
    }
  },
  {
    "request": {
      "method": "GET",
      "path": "/greet/wololo",
      "body": null,
      "params": [],
      "headers": {}
    },
    "response": {
      "statusCode": 200,
      "statusReason": null,
      "body": "Hello, Wololo !",
      "headers": {}
    }
  }
]
```

#### `POST /expectations`  

Create a new configuration with the dhall configuration in the request body.

Result :
 - `201` : The configuration was successfully parsed and is usable
 - `400` : The configuration in the body is invalid, compilation error in the response body

Example :
```bash
curl -X POST -i --data-binary "static.dhall" http://localhost:8089/expectations
HTTP/1.1 201 Created
content-length: 0
date: Wed, 06 May 2020 20:10:58 GMT
```

## Configuration

### Request

Http request received are compared to configurations to search for eligible ones.  
Currently, the first configuration (by inserting order) to match all of a configuration criteria is used.  

You can add request criteria (`HttpRequest`) on: 
 - Path
 - Http method (`GET`, `POST`, `DELETE`, `PUT`, `HEAD`, `OPTION`)
 - Http header
 - Query param
 - Body (Json or Text), body filter is matching the totality of the body, no partial matching for the moment

All criteria are optional if a none is provided the configuration accept any request for this specific criteria.

### Response

Http response (`HttpResponse`) could be configured with:
 - Status code (default `200`)
 - Http header (default `empty`)
 - Body  (default `empty`)
 - Status reason  (default `empty`)

### Dhall types

TBD - A listing of all dhall configuration types.  
In the mean time you can read the exposed configuration [here](dhall/Mock/package.dhall)

### Configuration sample

A configuration that create responses for `GET` method on `/greet/pwet` and `/greet/wololo` : [configuration](dhall/static.dhall)

Configuration that create responses based on a list of users and for each create a `GET ["ContentType": "application/json"] /users/{id}` route with associated json body : [configuration](dhall/example.dhall)

## Contributing

### Nix environment

A nix configuration id available to work on the project.
It contains `rust`, `cargo`, `dhall` and configure pre-commit hooks

### Local setup

To build the project on your machine you only need `rust` and `cargo`.
We build on stable release.

`cargo build` :sunglasses: 

### Dev guideline 

TBD

## License

Code is provided under the Apache 2.0 license available at http://opensource.org/licenses/Apache-2.0, as well as in the LICENSE file.
