use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::process::Command;

#[derive(Clone, Debug)]
pub enum Language {
    Python,
    Rust,
    Other(String),
}

macro_rules! unimplemented_language {
    ($lang:expr) => {
        panic!("Found an unexpected language \"{}\". You may need to update your sharpener CLI before proceeding.", $lang);
    };
}

impl Language {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::Rust => "rust",
            Self::Other(s) => s.as_str(),
        }
    }

    pub fn test_command(&self) -> Command {
        match self {
            Self::Python => Command::new("pytest"),
            Self::Rust => {
                let mut command = Command::new("cargo");
                command.arg("test");
                command
            }
            Self::Other(s) => unimplemented_language!(s),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use Language::*;
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "python" => Python,
            "rust" => Rust,
            _ => Other(value),
        })
    }
}
