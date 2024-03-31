use std::collections::HashMap;
use model::http_enum;
use crate::model;

pub struct HttpRequest {
    pub method: http_enum::HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub content: Option<HttpContent>,
}

pub struct HttpContent {
    pub content: String,
    pub binary_content: Option<Vec<u8>>,
    pub content_type: http_enum::HttpContentType,
}

pub struct HttpResponse {
    pub status_code: http_enum::HttpStatusCode,
    pub content: Option<HttpContent>,
}