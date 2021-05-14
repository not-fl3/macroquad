//! Cross platform files management functions.

use crate::exec;

#[derive(Debug)]
pub struct FileError {
    pub kind: miniquad::fs::Error,
    pub path: String,
}

impl std::error::Error for FileError {}
impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Couldn't load file {}: {}", self.path, self.kind)
    }
}
impl FileError {
    pub fn new(kind: miniquad::fs::Error, path: &str) -> Self {
        Self {
            kind,
            path: path.to_string(),
        }
    }
}

/// Load file from the path and block until its loaded
/// Will use filesystem on PC and do http request on web
pub async fn load_file(path: &str) -> Result<Vec<u8>, FileError> {
    fn load_file_inner(path: &str) -> exec::FileLoadingFuture {
        use std::cell::RefCell;
        use std::rc::Rc;

        let contents = Rc::new(RefCell::new(None));
        let path = path.to_owned();

        {
            let contents = contents.clone();
            let err_path = path.clone();

            miniquad::fs::load_file(&path, move |bytes| {
                *contents.borrow_mut() =
                    Some(bytes.map_err(|kind| FileError::new(kind, &err_path)));
            });
        }

        exec::FileLoadingFuture { contents }
    }

    load_file_inner(path).await
}

/// Load string from the path and block until its loaded.
/// Right now this will use load_file and `from_utf8_lossy` internally, but
/// implementation details may change in the future
pub async fn load_string(path: &str) -> Result<String, FileError> {
    let data = load_file(path).await?;

    Ok(String::from_utf8_lossy(&data).to_string())
}
