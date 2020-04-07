use crate::expectation::model::Expectation;
use anyhow::{Context, Error};
use std::fs;

pub fn load_file(name: &str) -> Result<String, Error> {
    fs::read_to_string(name).context(format!("Error reading file {} content", name))
}

pub fn compile_configuration(configuration_content: &str) -> Result<Vec<Expectation>, Error> {
    serde_dhall::from_str(configuration_content)
        .parse()
        .context("Error parsing shall configuration")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expectation::model::{HttpMethod, HttpRequest, HttpResponse};

    #[test]
    fn test_compile_configuration() {
        let data = r###"
            let Mock = ./dhall/Mock/package.dhall

            let expectations = [{ request  = { method  = Some Mock.HttpMethod.GET
                             , path    = Some "/greet/pwet"
                             }
                , response = { statusCode   = Some +200
                             , statusReason = None Text
                             , body         = Some "Hello, pwet !"
                             }
                }
            ]

            in expectations
        "###;

        let expected = vec![Expectation {
            request: HttpRequest {
                method: Some(HttpMethod::GET),
                path: Some("/greet/pwet".to_string()),
            },
            response: HttpResponse {
                status_code: Some(200),
                status_reason: None,
                body: Some("Hello, pwet !".to_string()),
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
