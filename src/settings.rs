use std::{io, path::PathBuf};

use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub path: PathBuf,
}

impl Settings {
    pub fn new() -> Result<Self, io::Error> {
        use std::env::current_dir;

        Ok(Settings {
            path: current_dir()?,
        })
    }
}

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::new().unwrap());
