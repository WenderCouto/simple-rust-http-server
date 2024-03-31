use crate::model::http_enum::{HttpMethod, HttpStatusCode};
use std::{collections::HashMap, str::FromStr};
use crate::model::http_struct::HttpContent;
use crate::model::{http_struct, http_enum};
use core::fmt;

impl http_struct::HttpRequest {
    pub fn new(raw_request: &str) -> http_struct::HttpRequest {
        match raw_request.split_once("\r\n\r\n") {
            Some((headers, body)) => {
                let lines: Vec<&str> = headers.split("\r\n").collect();
                let (method, path) = Self::parse_start_line(lines[0]);
                let headers: HashMap<String, String> = Self::parse_headers(&lines[1..]);

                let default_content_type = http_enum::HttpContentType::TextPlain.to_string();
                let content_type: &String = headers.get("Content-Type").unwrap_or(&default_content_type);
                let content: Option<HttpContent> = match http_enum::HttpContentType::from_str(content_type) {
                    Ok(content_type_enum) => match content_type_enum {
                        http_enum::HttpContentType::TextPlain | http_enum::HttpContentType::ApplicationJson => {
                            Self::parse_text_content(body)
                        },
                        http_enum::HttpContentType::ImageJpeg | http_enum::HttpContentType::ImageGif |
                        http_enum::HttpContentType::ImagePng | http_enum::HttpContentType::ImageSvgXml |
                        http_enum::HttpContentType::ImageWebp | http_enum::HttpContentType::ApplicationOctetStream => {
                            let binary_body = Some(body.as_bytes().to_vec());
                            Self::parse_binary_content(binary_body, content_type)
                        },
                        _ => {
                            None
                        },
                    },
                    Err(_) => {
                        None
                    },
                };

                http_struct::HttpRequest {
                    method,
                    path,
                    headers,
                    content
                }
            }
            None => panic!("Formato HTTP incorreto."),
        }
    }

    fn parse_start_line(linha: &str) -> (HttpMethod, String) {
        let resultado : Vec<&str> = linha.split( ' ').collect();
        return (
            HttpMethod::from_str(resultado[0]).expect("Metodo HTTP inesperado."),
            resultado[1].to_string(),
        );
    }

    fn parse_headers(headers: &[&str]) -> HashMap<String, String> {
        let mut headers_map: HashMap<String, String> = HashMap::new();

        for linha in headers{
            linha.split_once(":").and_then(|(header, value)| {
                headers_map.insert(header.to_string(), value.trim().to_string())
            });
        }
        return headers_map;
    }

    fn parse_text_content(content: &str) -> Option<http_struct::HttpContent> {
        if content.is_empty() {
            None
        } else {
            Some(http_struct::HttpContent {
                content: content.to_string(),
                binary_content: None,
                content_type: http_enum::HttpContentType::TextPlain,
            })
        }
    }

    fn parse_binary_content(content: Option<Vec<u8>>, content_type: &str) -> Option<http_struct::HttpContent> {
        match http_enum::HttpContentType::from_str(content_type) {
            Ok(content_type_enum) => {
                Some(http_struct::HttpContent {
                    content: String::new(),
                    binary_content: content,
                    content_type: content_type_enum,
                })
            },
            Err(_) => {
                None
            },
        }
    }

}

impl FromStr for HttpMethod{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            _ => Err(())
        }
    }
}


impl fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.status_code_matches())
    }
}

impl HttpStatusCode {
    fn status_code_matches(&self) -> &str { //REF: https://www.w3schools.com/tags/ref_httpmessages.asp
        match self {
            HttpStatusCode::Ok => "200 OK",
            HttpStatusCode::Created => "201 Created",
            HttpStatusCode::NotModified => "304 Not Modified",
            HttpStatusCode::BadRequest => "400 Bad Request",
            HttpStatusCode::NotFound => "404 Found",
            HttpStatusCode::InternalServerError => "500 Internal Server Error",
            HttpStatusCode::BadGateway => "502 Bad Gateway",
        }
    }
}

impl fmt::Display for http_struct::HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.content {
            Some(content) => {
                write!(
                    f,
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    self.status_code,
                    content.content_type,
                    content.content.len(),
                    content.content
                )
            }
            None => {
                write!(f, "HTTP/1.1 {}\r\n\r\n", self.status_code)
            }
        }
    }
}

impl http_enum::HttpContentType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            http_enum::HttpContentType::ImageJpeg => "image/jpeg",
            http_enum::HttpContentType::ImageGif => "image/gif",
            http_enum::HttpContentType::ImagePng => "image/png",
            http_enum::HttpContentType::ImageSvgXml => "image/svg+xml",
            http_enum::HttpContentType::ImageWebp => "image/webp",
            http_enum::HttpContentType::ApplicationOctetStream => "application/octet-stream",
            http_enum::HttpContentType::ApplicationJson => "application/json",
            http_enum::HttpContentType::MultiPartFormData => "multipart/form-data",
            http_enum::HttpContentType::TextPlain => "text/plain",
        }
    }
}


impl fmt::Display for http_enum::HttpContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for http_enum::HttpContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "image/jpeg" => Ok(http_enum::HttpContentType::ImageJpeg),
            "image/gif" => Ok(http_enum::HttpContentType::ImageGif),
            "image/png" => Ok(http_enum::HttpContentType::ImagePng),
            "image/svg+xml" => Ok(http_enum::HttpContentType::ImageSvgXml),
            "image/webp" => Ok(http_enum::HttpContentType::ImageWebp),
            "application/octet-stream" => Ok(http_enum::HttpContentType::ApplicationOctetStream),
            "application/json" => Ok(http_enum::HttpContentType::ApplicationJson),
            "multipart/form-data" => Ok(http_enum::HttpContentType::MultiPartFormData),
            "text/plain" => Ok(http_enum::HttpContentType::TextPlain),
            _ => Err(()),
        }
    }
}
