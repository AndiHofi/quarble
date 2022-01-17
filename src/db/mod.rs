use std::fs::{DirEntry, File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind};
use std::ops::RangeBounds;
use std::path::{Path, PathBuf};

use crate::data::{ActiveDay, Day, RecentIssuesData};
use crate::parsing::time::Time;
use thiserror::Error;

#[cfg(test)]
mod test;

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
            prev_work_day.and_then(|w| w.current_issue(Time::MAX)),
        );

        eprintln!("New: {:?}", new_day);

        Ok(new_day)
    }

    pub fn load_day(&self, day: Day) -> DBResult<Option<ActiveDay>> {
        let to_load = self.work_day_path(day);
        self.read_file(to_load)
    }

    pub fn list_days(&self, range: impl RangeBounds<Day>) -> DBResult<Vec<Day>> {
        let dirs =
            std::fs::read_dir(&self.root).map_err(|e| DBErr::NotADirectory(e.to_string()))?;

        let result = dirs
            .filter_map(|e| e.ok())
            .filter(is_file)
            .filter_map(|e| e.file_name().into_string().ok())
            .filter_map(|e| e.strip_suffix(".json").and_then(|s| Day::parse(s).ok()))
            .filter(|d| range.contains(d))
            .collect();

        Ok(result)
    }

    pub fn store_day(&self, day: Day, work_day: &ActiveDay) -> DBResult<()> {
        let to_store = self.work_day_path(day);

        let file = Self::open_for_write(&to_store)?;

        let write = BufWriter::new(file);
        serde_json::to_writer_pretty(write, work_day)
            .map_err(|_| DBErr::FailedToWrite(to_store.clone()))?;

        eprintln!("Stored: {:?}", work_day);
        Ok(())
    }

    fn open_for_write(to_store: &Path) -> DBResult<File> {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&to_store)
            .map_err(|e| DBErr::CannotOpen(to_store.to_owned(), e))
    }

    pub fn load_recent(&self) -> DBResult<RecentIssuesData> {
        let to_load = self.recent_issues_file();
        let loaded: Option<RecentIssuesData> = self.read_file(to_load)?;
        Ok(loaded.unwrap_or_default())
    }

    fn recent_issues_file(&self) -> PathBuf {
        self.root.join("recent.json")
    }

    pub fn store_recent(&self, data: &RecentIssuesData) -> DBResult<()> {
        let to_store = self.recent_issues_file();
        let file = Self::open_for_write(&to_store)?;
        let write = BufWriter::new(file);

        serde_json::to_writer(write, data).map_err(|_| DBErr::FailedToWrite(to_store.clone()))
    }

    fn work_day_path(&self, day: Day) -> PathBuf {
        let formatted = format!("{}.json", day);
        self.root.join(formatted)
    }

    fn read_file<T: serde::de::DeserializeOwned>(&self, to_load: PathBuf) -> DBResult<Option<T>> {
        if let Some(file) = handle_not_found(File::open(&to_load))
            .map_err(|e| DBErr::CannotOpen(to_load.clone(), e))?
        {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).map_err(|e| DBErr::InvalidDBFile(to_load.clone(), e))
        } else {
            Ok(None)
        }
    }
}
fn handle_not_found<T>(e: std::io::Result<T>) -> std::io::Result<Option<T>> {
    match e {
        Ok(t) => Ok(Some(t)),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().map(|t| t.is_file()).unwrap_or_default()
}
