use crate::expectation::model::Expectation;
use anyhow::{Context, Error};
use std::fs;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task, thread,
};

pub struct ConfLoadingFuture {
    shared_state: Arc<Mutex<ConfLoadingState>>,
}

impl ConfLoadingFuture {
    pub fn load_file(name: &str) -> Self {
        let shared_state = Arc::new(Mutex::new(ConfLoadingState {
            res: None,
            waker: None,
        }));

        // Spawn the new thread
        let thread_shared_state = shared_state.clone();
        let thread_name = String::from(name);
        thread::spawn(move || {
            let mut shared_state = thread_shared_state.lock().unwrap();

            let configuration_result = load_file(thread_name.as_ref())
                .and_then(|configuration| compile_configuration(&configuration))
                .context("Error load configuration");

            match configuration_result {
                Ok(expectations) => shared_state.res = Some(expectations),
                Err(e) => shared_state.res = Some(Vec::new()),
            }

            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        ConfLoadingFuture { shared_state }
    }
}

struct ConfLoadingState {
    res: Option<Vec<Expectation>>,
    waker: Option<task::Waker>,
}

impl Future for ConfLoadingFuture {
    type Output = Vec<Expectation>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();
        match shared_state.res.as_ref() {
            None => {
                shared_state.waker = Some(cx.waker().clone());
                task::Poll::Pending
            }
            Some(v) => task::Poll::Ready(v.clone()),
        }
    }
}

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
