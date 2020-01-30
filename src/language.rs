use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::Path;
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

fn parse_rust_coverage(command_output: &str) -> String {
    let regex =
        Regex::new(r"test result: (?:ok|FAILED)\. ([0-9]+) passed; ([0-9]+) failed;").unwrap();
    let (passed, failed) = regex
        .captures_iter(command_output)
        .fold((0, 0), |acc, captures| {
            let passed: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
            let failed: u32 = captures.get(2).unwrap().as_str().parse().unwrap();
            (acc.0 + passed, acc.1 + failed)
        });
    if passed == 0 && failed == 0 {
        "No test results".to_owned()
    } else {
        format!("{}/{}", passed, passed + failed)
    }
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

    pub fn parse_test_coverage(&self, command_output: &str) -> String {
        match self {
            Self::Rust => parse_rust_coverage(command_output),
            _ => unimplemented_language!(self.as_str()),
        }
    }

    pub fn test_file_path(&self) -> &Path {
        match self {
            Self::Rust => Path::new("tests/tests.rs"),
            Self::Python => Path::new("tests/tests.py"),
            _ => unimplemented_language!(self.as_str()),
        }
    }

    pub fn solution_file_path(&self) -> &Path {
        match self {
            Self::Rust => Path::new("src/lib.rs"),
            Self::Python => Path::new("src/main.py"),
            _ => unimplemented_language!(self.as_str()),
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
