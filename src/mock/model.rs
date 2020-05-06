use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::mock::serde as serde_mock;

pub type QueryParams = HashMap<String, HashSet<String>>;

#[derive(Debug, Clone)]
pub struct IncomingRequest {
    pub method: HttpMethod,
    pub path: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub params: QueryParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum HttpMethod {
    CONNECT,
    DELETE,
    GET,
    HEAD,
    OPTIONS,
    PATCH,
    POST,
    PUT,
    TRACE,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RequestBody {
    JSON {
        #[serde(with = "serde_mock::json_string")]
        json: Value,
    },
    TEXT {
        text: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HttpRequest {
    pub method: Option<HttpMethod>,
    pub path: Option<String>,
    pub body: Option<RequestBody>,
    pub params: Vec<(String, String)>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HttpResponse {
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
    #[serde(rename = "statusReason")]
    pub status_reason: Option<String>,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Expectation {
    pub request: HttpRequest,
    pub response: HttpResponse,
}

impl Expectation {
    pub fn test(&self, req: &IncomingRequest) -> bool {
        let match_method = self
            .request
            .method
            .as_ref()
            .map(|m| m == &req.method)
            .unwrap_or(true);

        let match_path = self
            .request
            .path
            .as_ref()
            .map(|p| p == &req.path)
            .unwrap_or(true);

        let body_match = match &self.request.body {
            Some(RequestBody::JSON { json }) => serde_json::from_str(req.body.as_ref())
                .map(|body: Value| *json == body)
                .unwrap_or(false),
            Some(RequestBody::TEXT { text }) => *text == req.body,
            _ => true,
        };

        let mut header_match = true;
        for (k, v) in self.request.headers.iter() {
            match req.headers.get(k) {
                Some(vv) if v == vv => continue,
                _ => {
                    header_match = false;
                    break;
                }
            }
        }

        let mut params_match = true;
        for (k, v) in self.request.params.iter() {
            match req.params.get(k) {
                Some(set) if set.contains(v) => continue,
                _ => {
                    params_match = false;
                    break;
                }
            }
        }

        match_method && match_path && body_match && header_match && params_match
    }

    pub fn look_for_expectation<'a, 'b>(
        expectations: &'a Vec<Expectation>,
        req: &'b IncomingRequest,
    ) -> Option<&'a Expectation> {
        expectations.iter().find(|e| e.test(req))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::model::{HttpRequest, HttpResponse};
    use serde_json::json;

    #[test]
    fn test_deserialize_http_method() {
        assert_eq!(
            HttpMethod::HEAD,
            serde_dhall::from_str(
                r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpMethod.HEAD
        "###
            )
            .parse()
            .unwrap()
        );
        assert_eq!(
            HttpMethod::GET,
            serde_dhall::from_str(
                r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpMethod.GET
        "###
            )
            .parse()
            .unwrap()
        );
        assert_eq!(
            HttpMethod::PUT,
            serde_dhall::from_str(
                r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpMethod.PUT
        "###
            )
            .parse()
            .unwrap()
        );
        assert_eq!(
            HttpMethod::POST,
            serde_dhall::from_str(
                r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpMethod.POST
        "###
            )
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn test_deserialize_http_method_fail() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpMethod.UnknowOccurence
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpMethod>().is_err());
    }

    #[test]
    fn test_deserialize_request_textual_body() {
        assert_eq!(
            RequestBody::TEXT {
                text: String::from("carpe diem.")
            },
            serde_dhall::from_str(
                r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.Body.TEXT { text = "carpe diem." }
        "###
            )
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn test_deserialize_request_json_body() {
        assert_eq!(
            RequestBody::JSON {
                json: json!({ "maxime": "carpe diem." })
            },
            serde_dhall::from_str(
                r###"
                    let Mock = ./dhall/Mock/package.dhall
                    in Mock.Body.JSON { json = "{ \"maxime\": \"carpe diem.\" }" }
                "###
            )
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn test_deserialize_http_params() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpRequest::{ params = [ { key = "foo", value = "bar"}]
                                 }
        "###;

        assert_eq!(
            HttpRequest {
                method: None,
                path: None,
                body: None,
                headers: HashMap::new(),
                params: vec![(String::from("foo"), String::from("bar"))],
            },
            serde_dhall::from_str(data).parse().unwrap()
        );
    }

    #[test]
    fn test_deserialize_http_headers() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpRequest::{ headers = [ Mock.contentTypeJSON ]
                                 }
        "###;

        let mut headers = HashMap::new();
        headers.insert(
            String::from("Content-Type"),
            String::from("application/json"),
        );

        assert_eq!(
            HttpRequest {
                method: None,
                path: None,
                body: None,
                params: vec![],
                headers: headers
            },
            serde_dhall::from_str(data).parse().unwrap()
        );
    }

    #[test]
    fn test_deserialize_http_request() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpRequest::{ method = Some Mock.HttpMethod.GET
                                 , path = Some "/path"
                                 }
        "###;
        assert_eq!(
            HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/path".to_string()),
                body: None,
                params: vec![],
                headers: HashMap::new()
            },
            serde_dhall::from_str(data).parse().unwrap()
        );

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpRequest::{ method = None Mock.HttpMethod
                                 }
        "###;
        assert_eq!(
            HttpRequest {
                method: None,
                path: None,
                body: None,
                params: vec![],
                headers: HashMap::new()
            },
            serde_dhall::from_str(data).parse().unwrap()
        );
    }

    #[test]
    fn test_deserialize_http_request_fail() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { method = Mock.HttpMethod.GET
                                 , path = "/path"
                                 }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpRequest>().is_err());

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { method = Mock.HttpMethod.GET
               , path = Some "/path"
               }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpRequest>().is_err());

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { method = Some Mock.HttpMethod.GET
               , path = "/path"
               }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpRequest>().is_err());
    }

    #[test]
    fn test_deserialize_http_response() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpResponse::{ statusCode = Some 200
                                  , body = Some "Hello, world !"
                                  }
        "###;
        assert_eq!(
            HttpResponse {
                status_code: Some(200),
                status_reason: None,
                body: Some("Hello, world !".to_string()),
                headers: HashMap::new(),
            },
            serde_dhall::from_str(data).parse().unwrap()
        );

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpResponse::{ statusCode = Some 200
                                  , statusReason = Some "Everything went fine"
                                  }"###;
        assert_eq!(
            HttpResponse {
                status_code: Some(200),
                status_reason: Some("Everything went fine".to_string()),
                body: None,
                headers: HashMap::new(),
            },
            serde_dhall::from_str(data).parse().unwrap()
        );
    }

    #[test]
    fn test_deserialize_http_response_fail() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpResponse::{ statusCode = 200
                                  , body = Some "Hello, world !"
                                  }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpResponse>().is_err());

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpResponse::{ statusCode = Some 200
                                  , statusReason = "Random text"
                                  , body = Some "Hello, world !"
                                  }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpResponse>().is_err());

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in Mock.HttpResponse::{ statusCode = Some 200
                                  , statusReason = None Text
                                  , body = "Hello, world !"
                                  }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpResponse>().is_err());
    }

    #[test]
    fn test_deserialize_expectation() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { request  = Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                                              , path    = Some "/greet/pwet"
                                              }
            , response = Mock.HttpResponse::{ statusCode   = Mock.statusOK
                                            , body         = Some "Hello, pwet !"
                                            }
            }
        "###;
        let expected = Expectation {
            request: HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/greet/pwet".to_string()),
                body: None,
                params: vec![],
                headers: HashMap::new(),
            },
            response: HttpResponse {
                status_code: Some(200),
                status_reason: None,
                body: Some("Hello, pwet !".to_string()),
                headers: HashMap::new(),
            },
        };
        assert_eq!(expected, serde_dhall::from_str(data).parse().unwrap());
    }

    #[test]
    fn test_deserialize_expectation_fail() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { request  = { }
            , response = { }
            }
        "###;
        assert!(serde_dhall::from_str(data).parse::<Expectation>().is_err());
    }

    #[test]
    fn test_accept_matching_method() {
        let req = HttpRequest {
            method: Some(HttpMethod::GET),
            path: None,
            body: None,
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from(""),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_method() {
        let req = HttpRequest {
            method: Some(HttpMethod::POST),
            path: None,
            body: None,
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from(""),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(None, tested);
    }

    #[test]
    fn test_accept_matching_path() {
        let req = HttpRequest {
            method: None,
            path: Some(String::from("/foo/bar")),
            body: None,
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from(""),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_path() {
        let req = HttpRequest {
            method: None,
            path: Some(String::from("/foo/bar")),
            body: None,
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/users"),
            body: String::from(""),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(None, tested);
    }

    #[test]
    fn test_accept_matching_json_body() {
        let content = json!({ "maxime": "carpe diem." });

        let req = HttpRequest {
            method: None,
            path: None,
            body: Some(RequestBody::JSON { json: content }),
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("{\n \"maxime\": \"carpe diem.\" \n}"),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_json_body() {
        let content = json!({ "maxime": "carpe diem." });

        let req = HttpRequest {
            method: None,
            path: None,
            body: Some(RequestBody::JSON { json: content }),
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("{\n \"maxime\": \"this is not carpe diem.\" \n}"),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(None, tested);
    }

    #[test]
    fn test_accept_matching_text_body() {
        let req = HttpRequest {
            method: None,
            path: None,
            body: Some(RequestBody::TEXT {
                text: String::from("carpe diem."),
            }),
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("carpe diem."),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_text_body() {
        let req = HttpRequest {
            method: None,
            path: None,
            body: Some(RequestBody::TEXT {
                text: String::from("carpe diem."),
            }),
            params: vec![],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("this is not carpe diem."),
            params: HashMap::new(),
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(None, tested);
    }

    #[test]
    fn test_accept_matching_headers() {
        let mut headers = HashMap::new();
        headers.insert(
            String::from("Content-Type"),
            String::from("application/json"),
        );

        let req = HttpRequest {
            method: None,
            path: None,
            body: None,
            params: vec![],
            headers: headers,
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let mut incoming_headers = HashMap::new();
        incoming_headers.insert(
            String::from("Content-Type"),
            String::from("application/json"),
        );
        incoming_headers.insert(String::from("User-Agent"), String::from("Mozilla/5.0"));

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("carpe diem."),
            params: HashMap::new(),
            headers: incoming_headers,
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_headers() {
        let mut headers = HashMap::new();
        headers.insert(
            String::from("Content-Type"),
            String::from("application/json"),
        );

        let req = HttpRequest {
            method: None,
            path: None,
            body: None,
            params: vec![],
            headers: headers,
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let mut incoming_headers = HashMap::new();
        incoming_headers.insert(
            String::from("Content-Type"),
            String::from("wrong content type."),
        );
        incoming_headers.insert(String::from("User-Agent"), String::from("Mozilla/5.0"));

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("carpe diem."),
            params: HashMap::new(),
            headers: incoming_headers,
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(None, tested);
    }

    #[test]
    fn test_accept_matching_params() {
        let req = HttpRequest {
            method: None,
            path: None,
            body: None,
            params: vec![
                (String::from("baz"), String::from("foo")),
                (String::from("baz"), String::from("bar")),
            ],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let mut params = HashMap::new();
        params.insert(
            String::from("baz"),
            [String::from("foo"), String::from("bar")]
                .iter()
                .cloned()
                .collect(),
        );
        params.insert(
            String::from("dontcare"),
            [String::from("42")].iter().cloned().collect(),
        );

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("carpe diem."),
            params: params,
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_params() {
        let req = HttpRequest {
            method: None,
            path: None,
            body: None,
            params: vec![
                (String::from("baz"), String::from("foo")),
                (String::from("baz"), String::from("bar")),
            ],
            headers: HashMap::new(),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
            headers: HashMap::new(),
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let mut params = HashMap::new();
        params.insert(
            String::from("baz"),
            [String::from("foo")].iter().cloned().collect(),
        );
        params.insert(
            String::from("dontcare"),
            [String::from("42")].iter().cloned().collect(),
        );

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
            body: String::from("carpe diem."),
            params: params,
            headers: HashMap::new(),
        };

        let v = vec![exp.clone()];
        let tested = Expectation::look_for_expectation(&v, &income);

        assert_eq!(None, tested);
    }
}
