use super::model::Expectation;
use anyhow::{Context, Error};

pub fn compile_configuration(configuration_content: &str) -> Result<Vec<Expectation>, Error> {
    serde_dhall::from_str(configuration_content)
        .parse()
        .context("Error parsing shall configuration")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::model::{Expectation, HttpMethod, HttpRequest, HttpResponse};
    use std::collections::HashMap;

    #[test]
    fn test_compile_configuration() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall

            let expectations = [ { request  = Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                                                                , path    = Some "/greet/pwet"
                                                                }
                               , response = Mock.HttpResponse::{ statusCode   = Mock.statusOK
                                                               , body         = Some "Hello, pwet !"
                                                               }
                               }]

            in expectations
        "###;

        let expected = vec![Expectation {
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
        }];

        let actual = compile_configuration(data).unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_compile_configuration_fail_on_wrong_configuration_input() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall
            in { request  = { method  = Some Mock.HttpMethod.GET
                             , path    = Some "/greet/pwet"
                             }
                , response = { statusCode   = Some "200"
                             , statusReason = None
                             , body         = Some "Hello, pwet !"
                             }
                }
        "###;

        assert!(compile_configuration(data).is_err())
    }
}
