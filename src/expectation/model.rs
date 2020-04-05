use serde::Deserialize;

#[derive(Debug, Deserialize, std::cmp::PartialEq)]
pub enum HttpMethod {
    HEAD,
    GET,
    PUT,
    POST,
}

#[derive(Debug, Deserialize)]
pub struct HttpRequest {
    pub method: Option<HttpMethod>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HttpResponse {
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
    #[serde(rename = "statusReason")]
    pub status_reason: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Expectation {
    pub request: HttpRequest,
    pub response: HttpResponse,
}
