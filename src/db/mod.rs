use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBErr {
    #[error("DB location is not a directory: {0}")]
    NotADirectory(String),
    #[error("DB directory could not be created: {0}")]
    FailedCreation(std::io::Error),
}

type DBResult<T> = Result<T, DBErr>;

pub struct DB {
    root: PathBuf,
}

impl DB {
    pub fn init(location: &Path) -> DBResult<DB> {
        if location.is_dir() {
            Ok(DB {
                root: location.to_path_buf(),
            })
        } else if location.exists() {
            Err(DBErr::NotADirectory(location.display().to_string()))
        } else {
            let mut buf = std::env::current_dir().unwrap().join(location);

            log::info!("Creating database at {}", buf.display());
            if let Err(e) = std::fs::create_dir_all(location) {
                Err(DBErr::FailedCreation(e))
            } else {
                Ok(DB {
                    root: location.to_path_buf(),
                })
            }
        }
    }
}
