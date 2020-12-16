//! Cross platform files management functions.

use crate::exec;

pub use crate::exec::FileError;

/// Load file from the path and block until its loaded
/// Will use filesystem on PC and do http request on web
pub fn load_file(path: &str) -> exec::FileLoadingFuture {
    use std::cell::RefCell;
    use std::rc::Rc;

    let contents = Rc::new(RefCell::new(None));
    let path = path.to_owned();

    {
        let contents = contents.clone();
        let err_path = path.clone();

        miniquad::fs::load_file(&path, move |bytes| {
            *contents.borrow_mut() =
                Some(bytes.map_err(|kind| exec::FileError::new(kind, &err_path)));
        });
    }

    exec::FileLoadingFuture { contents }
}

/// Load string from the path and block until its loaded.
/// Right now this will use load_file and `from_utf8_lossy` internally, but
/// implementatio details may change in the future
pub async fn load_string(path: &str) -> Result<String, exec::FileError> {
    let data = load_file(path).await?;

    Ok(String::from_utf8_lossy(&data).to_string())
}
