use snafu::Snafu;
pub use snafu::{ensure, ErrorCompat, OptionExt, ResultExt};
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Error while writting config file: {}", source))]
    ConfigWrite { source: serde_json::error::Error },

    #[snafu(display("Could not open config file at {}: {}", filename.display(), source))]
    OpenConfigFile {
        filename: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "Invalid config found: {}\n\nRun `sharpener config` to generate a new config",
        source
    ))]
    ConfigParsing { source: serde_json::error::Error },

    #[snafu(display(
        "The CLI token is invalid, please generate a new one with `sharpener config`"
    ))]
    InvalidToken {
        source: reqwest::header::InvalidHeaderValue,
    },

    #[snafu(display("An internal error occurred: {}", source))]
    ClientBuild { source: reqwest::Error },

    #[snafu(display("Could not complete request to Sharpener server: {}", source))]
    ServerRequest { source: reqwest::Error },

    #[snafu(display("Could not parse submission response from server: {}", source))]
    ParseSubmissionResponse { source: serde_json::error::Error },

    #[snafu(display(
        "Got invalid response from server: expected {}, got {}",
        expected,
        received
    ))]
    InvalidAPIResponse {
        expected: reqwest::StatusCode,
        received: reqwest::StatusCode,
    },

    #[snafu(display("Unable to download exercise: {}", source))]
    ExerciseDownload { source: reqwest::Error },

    #[snafu(display("Unable to download exercise: {}", source))]
    UnpackTar { source: std::io::Error },

    #[snafu(display("Unable to load exercise metadata: {}", source))]
    OpenMetaFile { source: std::io::Error },

    #[snafu(display("Unable to load exercise metadata: {}", source))]
    ParseMetaFile { source: serde_json::error::Error },

    #[snafu(display("Unable to write exercise metadata: {}", source))]
    WriteMetaFile { source: serde_json::error::Error },

    #[snafu(display("IO Error: {}", source))]
    IOError { source: std::io::Error },

    #[snafu(display("Exercise metadata not found. Make sure you're inside a directory created by `sharpener download`."))]
    MissingMeta,

    #[snafu(display("Unable to run test command: {}", source))]
    TestCommand { source: std::io::Error },

    #[snafu(display(
        "Unable to forfeit current exercise: there are no available exercice to replace it."
    ))]
    InvalidForfeit,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
