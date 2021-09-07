//! Cross platform file management functions.

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
        use std::sync::{Arc, Mutex};

        let contents = Arc::new(Mutex::new(None));
        let path = path.to_owned();

        {
            let contents = contents.clone();
            let err_path = path.clone();

            miniquad::fs::load_file(&path, move |bytes| {
                *contents.lock().unwrap() =
                    Some(bytes.map_err(|kind| FileError::new(kind, &err_path)));
            });
        }

        exec::FileLoadingFuture { contents }
    }

    #[cfg(target_os = "ios")]
    let _ = std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap());

    #[cfg(not(target_os = "android"))]
    let path = if let Some(ref pc_assets) = crate::get_context().pc_assets_folder {
        format!("{}/{}", pc_assets, path)
    } else {
        path.to_string()
    };

    load_file_inner(&path).await
}

/// Load string from the path and block until its loaded.
/// Right now this will use load_file and `from_utf8_lossy` internally, but
/// implementation details may change in the future
pub async fn load_string(path: &str) -> Result<String, FileError> {
    let data = load_file(path).await?;

    Ok(String::from_utf8_lossy(&data).to_string())
}

/// There are super common project layout like this:
/// ```skip
///    .
///    ├── assets
///    ├── └── nice_texture.png
///    ├── src
///    ├── └── main.rs
///    └── Cargo.toml
/// ```
/// when such a project being run on desktop assets should be referenced as
/// "assets/nice_texture.png".
/// While on web or android it usually is just "nice_texture.png".
/// The reason: on PC assets are being referenced relative to current active directory/executable path. In most IDEs its the root of the project.
/// While on, say, android it is:
/// ```skip
/// [package.metadata.android]
/// assets = "assets"
/// ```
/// And therefore on android assets are referenced from the root of "assets" folder.
///
/// In the future there going to be some sort of meta-data file for PC as well.
/// But right now to resolve this situation and keep pathes consistent across platforms
/// `set_pc_assets_folder("assets");`call before first `load_file`/`load_texture` will allow using same pathes on PC and Android.
pub fn set_pc_assets_folder(path: &str) {
    crate::get_context().pc_assets_folder = Some(path.to_string());
}
