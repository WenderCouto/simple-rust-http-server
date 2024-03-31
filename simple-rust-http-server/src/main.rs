use std::{path::{Path, PathBuf}, string::String, fs::File, io::{Read, Write}, net::{TcpListener, TcpStream}, str::FromStr, thread::spawn, env};
use crate::model::http_enum::{HttpContentType, HttpMethod, HttpStatusCode};
use crate::model::http_struct::{HttpContent, HttpRequest, HttpResponse};
use percent_encoding::percent_decode_str;
use model::{http_struct, http_enum};
use walkdir::{DirEntry, WalkDir};
use serde_json::Value;

mod model{
    pub mod http_struct;
    pub mod http_enum;
}

mod implements{
    pub mod http_implement;
}

fn main() {
    let escuta_tcp: TcpListener = TcpListener::bind("127.0.0.1:8000").expect("Falha ao criar Listen TCP");

    for stream in escuta_tcp.incoming() {
        match stream {
            Ok(stream) => {
                spawn(|| {
                    handle_request(stream);
                });
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

fn handle_request(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_read: usize = stream.read(&mut buffer).expect("Falha ao ler a linha");

    let response:HttpResponse = match std::str::from_utf8(&buffer[..bytes_read]) {
        Ok(raw_request) => {
            let request: HttpRequest = HttpRequest::new(raw_request);

            if request.path.eq("/") {
                build_ok_response(None)
            } else if request.path.starts_with("/echo/") {
                let content: String = request.path.replace("/echo/", "");
                let decoded_content = percent_decode_str(&content).decode_utf8().expect("Erro ao decodificar URL.");
                build_ok_response(Some(&decoded_content))
            } else if request.path.starts_with("/user-agent") {
                build_ok_response(Some(
                    &request
                        .headers
                        .get("User-Agent")
                        .expect("Não foi possível obter o user-agent"),
                ))
            } else if request.path.starts_with("/files/") || request.path.starts_with("/file") {
                handle_file_request(request)
            } else {
                HttpResponse {
                    status_code: HttpStatusCode::NotFound,
                    content: None,
                }
            }
        }
        Err(e) => panic!("Sequencia UTF-8 Inválida: {}", e),
    };

    flush_response(stream, response)
}

fn flush_response(mut stream: TcpStream, response: HttpResponse) {
    let header: String = match &response.content {
        Some(content) => {
            if let Some(binary_content) = &content.binary_content {
                format!(
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                    response.status_code,
                    content.content_type,
                    binary_content.len()
                )
            } else {
                format!(
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    response.status_code,
                    content.content_type,
                    content.content.len(),
                    content.content
                )
            }
        }
        None => {
            format!("HTTP/1.1 {}\r\n\r\n", response.status_code)
        }
    };

    stream.write(header.as_bytes()).unwrap();
    if let Some(content) = response.content {
        if let Some(binary_content) = content.binary_content {
            stream.write(&binary_content).unwrap();
        }
    }
    stream.flush().unwrap();
}

fn build_ok_response(content: Option<&str>) -> HttpResponse {
    HttpResponse {
        status_code: HttpStatusCode::Ok,
        content: content.map(|c: &str| HttpContent {
            content: String::from(c),
            binary_content: None,
            content_type: HttpContentType::TextPlain,
        }),
    }
}

fn handle_file_request(mut request: HttpRequest) -> HttpResponse {
    if request.method != HttpMethod::POST {
        return HttpResponse {
            status_code: HttpStatusCode::BadGateway,
            content: None,
        };
    }
    let http_content: HttpContent = match request.content {
        Some(content) => content,
        None => panic!("Conteúdo não encontrado."),
    };

    let cleaned_content: String = http_content.content.replace("\0", "").replace("\\", "\\\\");
    let json_result: Result<Value, serde_json::Error> = serde_json::from_str(&cleaned_content);

    let json: Value = match json_result {
        Ok(json) => json,
        Err(_) => return HttpResponse {
            status_code: HttpStatusCode::BadGateway,
            content: None,
        },
    };

    let json_obj: Option<&Value> = if json.to_string().starts_with("[") {
        json.as_array().and_then(|arr: &Vec<Value> | arr.first())
    } else if json.to_string().starts_with("{") {
        Some(&json)
    } else {
        None
    };

    let mut file_path: String = match json_obj.and_then(|obj: &Value| find_image_path(obj)) {
        Some(path) => path,
        None => return HttpResponse {
            status_code: HttpStatusCode::BadGateway,
            content: Option::from(HttpContent {
                content: "Diretório vazio ou não existe.".to_string(),
                binary_content: None,
                content_type: HttpContentType::TextPlain,
            }),
        },
    };

    let base_dir: PathBuf = if cfg!(target_os = "windows") {
        if !(file_path.chars().nth(0).unwrap().is_alphabetic() && file_path.chars().nth(1).unwrap() == ':'
            && (file_path.chars().nth(2).unwrap() == '\\'
            || file_path.chars().nth(3).unwrap() == '\\')) {
            return HttpResponse {
                status_code: HttpStatusCode::BadGateway,
                content: Option::from(HttpContent {
                    content: "Para o sistema Windows informe o Caminho absoluto.".to_string(),
                    binary_content: None,
                    content_type: HttpContentType::TextPlain,
                }),
            }
        } else {
            PathBuf::from(file_path.clone())
        }
    } else {
        PathBuf::from(env::var("HOME").expect("Falha ao obter o diretório home, Linux"))
    };

    let target_dir : String = file_path.clone();
    let mut absolute_path : PathBuf = base_dir.clone();
    let mut file_path_is_relative: bool = false;

    if target_dir.starts_with(&*absolute_path.to_string_lossy()) {
        absolute_path = PathBuf::from(target_dir);
    } else {
        let file_path_clone: String = file_path.clone();
        let relative_parts: Vec<&str> = file_path_clone.strip_prefix("/").unwrap_or(&*file_path_clone).split('/').collect();

        let mut base_dir_path: PathBuf = PathBuf::from(&base_dir);

        for part in relative_parts.iter() {
            let mut is_file: bool = false;
            for entry in WalkDir::new(&base_dir_path)
                .into_iter()
                .filter_entry(|e: &DirEntry| !e.file_name().to_str().map(|s: &str| s.starts_with('.')).unwrap_or(false)) {

                let entry: DirEntry = entry.unwrap();
                let entry_name: String = entry.file_name().to_string_lossy().into_owned();
                if entry_name == **part {
                    base_dir_path = entry.clone().into_path();
                    is_file = entry.file_type().is_file();
                    break;
                }
            }
            if part == relative_parts.last().unwrap() && is_file {
                absolute_path = base_dir_path.clone();
                file_path = base_dir_path.to_str().unwrap().to_string();
                if file_path.starts_with("/") {
                    file_path_is_relative = true;
                }
            }
        }
    }

    if let Some(parent_path) = absolute_path.parent() {
        absolute_path = parent_path.to_path_buf();
    }

    if file_path_is_relative {
        absolute_path.push(&file_path);
    } else{
        absolute_path.push(file_path);
    }

    let path: &Path = Path::new(&absolute_path);
    let ext: &str = path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or("");
    let image_extensions: [&str; 7] = ["jpeg", "jpg", "jpe", "gif", "png", "svg", "webp"];

    if !image_extensions.contains(&ext) {
        return HttpResponse {
            status_code: HttpStatusCode::BadGateway,
            content: None,
        };
    }

    let mut file: File = File::open(&path).expect("Falha ao abrir o arquivo.");
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).expect("Falha ao ler o arquivo.");

    let content_type: HttpContentType = match ext {
        "jpeg" | "jpg" | "jpe" => HttpContentType::ImageJpeg,
        "gif" => HttpContentType::ImageGif,
        "png" => HttpContentType::ImagePng,
        "svg" => HttpContentType::ImageSvgXml,
        "webp" => HttpContentType::ImageWebp,
        _ => HttpContentType::ApplicationOctetStream,
    };

    let content_type_str: &str = content_type.as_str();
    request.headers.insert("Content-Type".to_string(), content_type_str.to_string());

    build_ok_image_response(Some(buffer), content_type_str)
}


fn build_ok_image_response(content: Option<Vec<u8>>, content_type: &str) -> HttpResponse {
    match HttpContentType::from_str(content_type) {
        Ok(content_type_enum) => {
            HttpResponse {
                status_code: HttpStatusCode::Ok,
                content: Option::from(HttpContent {
                    content: String::new(),
                    binary_content: content,
                    content_type: content_type_enum
                }),
            }
        },
        Err(_) => {
            HttpResponse {
                status_code: HttpStatusCode::InternalServerError,
                content: Option::from(HttpContent {
                    content: String::new(),
                    binary_content: None,
                    content_type: HttpContentType::TextPlain,
                }),
            }
        },
    }
}

fn find_image_path(json: &Value) -> Option<String> {
    let image_extensions: [&str; 7] = ["jpeg", "jpg", "jpe", "gif", "png", "svg", "webp"];
    if let Some(map) = json.as_object() {
        for (_, value) in map {
            if let Some(file_path) = value.as_str() {
                let mut file_path_string: String = file_path.to_string();
                if file_path_string.starts_with('/') {
                    file_path_string = file_path_string.clone();
                } else if file_path_string.contains(":\\") {
                    file_path_string = file_path_string.to_string();
                } else {
                    file_path_string = format!("/{}", file_path_string);
                }
                let path: &Path = Path::new(&file_path_string);
                let ext: &str = path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or("");
                if image_extensions.contains(&ext) {
                    return Some(file_path_string.to_string());
                }
            }
        }
    }
    None
}