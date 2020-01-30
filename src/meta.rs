use crate::error::*;
use crate::language::Language;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub language: Language,
    pub difficulty: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints_seen: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_token: Option<String>,
}

impl Meta {
    pub fn get() -> Result<(Self, PathBuf)> {
        let working_dir = env::current_dir().context(IOError {})?;
        let mut current_dir = Some(working_dir.as_path());
        while let Some(dir) = current_dir {
            let path = dir.join(".meta.json");
            let result = File::open(&path)
                .map_err(|_| ())
                .and_then(|file| serde_json::from_reader(&file).map_err(|_| ()));
            match result {
                Ok(meta) => return Ok((meta, path)),
                Err(_) => {
                    current_dir = dir.parent();
                }
            }
        }
        Err(Error::MissingMeta)
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let meta_file = File::create(path).context(OpenMetaFile {})?;
        serde_json::to_writer_pretty(&meta_file, self).context(WriteMetaFile {})?;
        Ok(())
    }
}
