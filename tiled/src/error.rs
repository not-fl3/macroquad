#[derive(Debug)]
pub enum Error {
    DeJsonErr {
        msg: String,
        line: usize,
        col: usize,
    },
    NonUniqueLayerName {
        layer: String,
    },
    TextureNotFound {
        texture: String
    }
}

impl From<nanoserde::DeJsonErr> for Error {
    fn from(error: nanoserde::DeJsonErr) -> Error {
        Error::DeJsonErr {
            msg: error.msg.clone(),
            line: error.line,
            col: error.col,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DeJsonErr { .. } | Error::TextureNotFound {..} => std::fmt::Debug::fmt(self, f),
            Error::NonUniqueLayerName { layer } => write!(
                f,
                "Layer name should be unique to load tiled level in macroquad, non-unique layer name: {}", layer
            ),
            
        }
    }
}
