#[derive(Debug)]
pub enum Error {
    FontError(&'static str),
    FileError {
        kind: miniquad::fs::Error,
        path: String,
    },
    ShaderError(miniquad::ShaderError),
    ImageError(image::ImageError),
    UnknownError(&'static str),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::UnknownError(s)
    }
}

impl From<miniquad::ShaderError> for Error {
    fn from(s: miniquad::ShaderError) -> Self {
        Error::ShaderError(s)
    }
}

impl From<image::ImageError> for Error {
    fn from(s: image::ImageError) -> Self {
        Error::ImageError(s)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {:?}", self)
    }
}

impl std::error::Error for Error {}
