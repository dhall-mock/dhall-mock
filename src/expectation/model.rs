use serde::Deserialize;
use std::cmp::PartialEq;

#[derive(Debug, Deserialize, PartialEq)]
pub enum HttpMethod {
    HEAD,
    GET,
    PUT,
    POST,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct HttpRequest {
    pub method: Option<HttpMethod>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct HttpResponse {
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
    #[serde(rename = "statusReason")]
    pub status_reason: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Expectation {
    pub request: HttpRequest,
    pub response: HttpResponse,
}

#[cfg(test)]
mod test {
    use super::*;

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
    fn test_deserialize_http_request() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { method = Some Mock.HttpMethod.GET
                , path = Some "/path"
            }
        "###;
        assert_eq!(
            HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/path".to_string())
            },
            serde_dhall::from_str(data).parse().unwrap()
        );

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { method = None Mock.HttpMethod
                , path = None Text
            }
        "###;
        assert_eq!(
            HttpRequest {
                method: None,
                path: None
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
            in { statusCode = Some 200
                , statusReason = None Text
                , body = Some "Hello, world !"
            }
        "###;
        assert_eq!(
            HttpResponse {
                status_code: Some(200),
                status_reason: None,
                body: Some("Hello, world !".to_string())
            },
            serde_dhall::from_str(data).parse().unwrap()
        );

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { statusCode = Some 200
                , statusReason = Some "Everything went fine"
                , body = None Text
            }"###;
        assert_eq!(
            HttpResponse {
                status_code: Some(200),
                status_reason: Some("Everything went fine".to_string()),
                body: None
            },
            serde_dhall::from_str(data).parse().unwrap()
        );
    }

    #[test]
    fn test_deserialize_http_response_fail() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { statusCode = 200
                , statusReason = None Text
                , body = Some "Hello, world !"
            }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpResponse>().is_err());

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { statusCode = Some 200
                , statusReason = "Random text"
                , body = Some "Hello, world !"
            }
        "###;
        assert!(serde_dhall::from_str(data).parse::<HttpResponse>().is_err());

        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { statusCode = Some 200
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
            in { request  = { method  = Some Mock.HttpMethod.GET
                             , path    = Some "/greet/pwet"
                             }
            , response = { statusCode   = Some +200
                         , statusReason = None Text
                         , body         = Some "Hello, pwet !"
                         }
            }
        "###;
        let expected = Expectation {
            request: HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/greet/pwet".to_string()),
            },
            response: HttpResponse {
                status_code: Some(200),
                status_reason: None,
                body: Some("Hello, pwet !".to_string()),
            },
        };
        assert_eq!(expected, serde_dhall::from_str(data).parse().unwrap());
    }

    fn test_deserialize_expectation_fail() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { request  = { }
            , response = { }
            }
        "###;
        assert!(serde_dhall::from_str(data).parse::<Expectation>().is_err());
    }
}
