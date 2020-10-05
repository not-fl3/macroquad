//! Cross platform files management functions.

use crate::exec;

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
