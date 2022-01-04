use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::data::{ActiveDay, Day, JiraIssue};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBErr {
    #[error("DB location is not a directory: {0}")]
    NotADirectory(String),
    #[error("DB directory could not be created: {0}")]
    FailedCreation(std::io::Error),
    #[error("Cannot open file '{0}': {1}")]
    CannotOpen(PathBuf, std::io::Error),
    #[error("Invalid db file {0}: {1}")]
    InvalidDBFile(PathBuf, serde_json::Error),
    #[error("Failed to write {0}")]
    FailedToWrite(PathBuf),
}

type DBResult<T> = Result<T, DBErr>;

#[derive(Debug, Clone)]
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
            let buf = std::env::current_dir().unwrap().join(location);

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

    pub fn get_day(&self, day: Day) -> DBResult<ActiveDay> {
        let work_day = self.load_day(day)?;
        if let Some(work_day) = work_day {
            Ok(work_day)
        } else {
            self.new_day(day)
        }
    }

    pub fn new_day(&self, day: Day) -> DBResult<ActiveDay> {
        let mut prev_day = day.prev_day();
        let mut remaining = 6;
        let prev_work_day = loop {
            if let Some(work_day) = self.load_day(prev_day)? {
                break Some(work_day);
            } else if remaining <= 0 {
                break None;
            }
            {
                remaining -= 1;
                prev_day = prev_day.prev_day();
            }
        };

        let new_day = ActiveDay::new(
            day,
            prev_work_day
                .as_ref()
                .map(|w| w.main_location().clone())
                .unwrap_or_default(),
            prev_work_day.and_then(|w| w.active_issue().map(JiraIssue::clone)),
        );

        eprintln!("New: {:?}", new_day);

        Ok(new_day)
    }

    pub fn load_day(&self, day: Day) -> DBResult<Option<ActiveDay>> {
        let to_load = self.work_day_path(day);
        if to_load.exists() {
            let file = File::open(&to_load).map_err(|e| DBErr::CannotOpen(to_load.clone(), e))?;
            let reader = BufReader::new(file);
            let work_day: ActiveDay = serde_json::from_reader(reader)
                .map_err(|e| DBErr::InvalidDBFile(to_load.clone(), e))?;
            eprintln!("Loaded: {:?}", work_day);
            Ok(Some(work_day))
        } else {
            Ok(None)
        }
    }

    pub fn store_day(&self, day: Day, work_day: &ActiveDay) -> DBResult<()> {
        let to_store = self.work_day_path(day);

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&to_store)
            .map_err(|e| DBErr::CannotOpen(to_store.clone(), e))?;

        let write = BufWriter::new(file);
        serde_json::to_writer_pretty(write, work_day)
            .map_err(|_| DBErr::FailedToWrite(to_store.clone()))?;

        eprintln!("Stored: {:?}", work_day);
        Ok(())
    }

    fn work_day_path(&self, day: Day) -> PathBuf {
        let formatted = format!("{}.json", day);
        self.root.join(formatted)
    }
}
