use crate::expectation::model::{Expectation, HttpMethod};

pub struct IncomingRequest {
    pub method: HttpMethod,
    pub path: String,
}

pub fn look_for_expectation(
    expectations: &Vec<Expectation>,
    req: IncomingRequest,
) -> Option<&Expectation> {
    expectations.iter().find(|e| {
        let match_method = e
            .request
            .method
            .as_ref()
            .map(|m| m == &req.method)
            .unwrap_or(true);

        let match_path = e
            .request
            .path
            .as_ref()
            .map(|p| p == &req.path)
            .unwrap_or(true);

        match_method && match_path
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expectation::model::{HttpRequest, HttpResponse};

    #[test]
    fn test_accept_matching_method() {
        let req = HttpRequest {
            method: Some(HttpMethod::GET),
            path: None,
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
        };

        let v = vec![exp.clone()];
        let tested = look_for_expectation(&v, income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_method() {
        let req = HttpRequest {
            method: Some(HttpMethod::POST),
            path: None,
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
        };

        let v = vec![exp.clone()];
        let tested = look_for_expectation(&v, income);

        assert_eq!(None, tested);
    }

    #[test]
    fn test_accept_matching_path() {
        let req = HttpRequest {
            method: None,
            path: Some(String::from("/foo/bar")),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/foo/bar"),
        };

        let v = vec![exp.clone()];
        let tested = look_for_expectation(&v, income);

        assert_eq!(Some(&exp), tested);
    }

    #[test]
    fn test_refuse_wrong_path() {
        let req = HttpRequest {
            method: None,
            path: Some(String::from("/foo/bar")),
        };

        let resp = HttpResponse {
            status_code: Some(200),
            status_reason: None,
            body: None,
        };

        let exp = Expectation {
            request: req,
            response: resp,
        };

        let income = IncomingRequest {
            method: HttpMethod::GET,
            path: String::from("/users"),
        };

        let v = vec![exp];
        let tested = look_for_expectation(&v, income);

        assert_eq!(None, tested);
    }
}
