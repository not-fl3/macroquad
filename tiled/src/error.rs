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
        texture: String,
    },
    TileMapLoadingFailed {
        inner_error: Box<Error>,
    },
    TilesetLoadingFailed {
        tileset: String,
        inner_error: Box<Error>,
    },
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
            Error::TileMapLoadingFailed { inner_error } => write!(f, "Tile map failed to load: {}", inner_error),
            Error::TilesetLoadingFailed { tileset, inner_error } => write!(f, "Tileset {} failed to load: {}", tileset, inner_error)
        }
    }
}

impl std::error::Error for Error {}
