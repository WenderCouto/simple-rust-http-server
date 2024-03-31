#[derive(Debug, PartialEq)]
pub enum HttpMethod{
    GET,
    POST,
    PUT,
    DELETE
}

pub enum HttpContentType {
    ImageJpeg,
    ImageGif,
    ImagePng,
    ImageSvgXml,
    ImageWebp,
    ApplicationOctetStream,
    ApplicationJson,
    MultiPartFormData, //Upload de imagem, não retorno de imagem.
    TextPlain
}

#[allow(dead_code)]
pub enum HttpStatusCode {
    Ok = 200,
    Created = 201,
    NotModified = 304,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
    BadGateway = 502
}
