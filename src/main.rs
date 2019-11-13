mod config;
mod error;
mod language;
mod meta;
mod submission;

use crate::config::Config;
use crate::error::*;
use crate::meta::Meta;
use language::Language;
use reqwest::Client;
use serde::Deserialize;
use structopt::StructOpt;
use submission::{Submission, SubmissionStatus};

static API_URI: &str = "http://localhost:5000/api/";
static BUCKET_URI: &str = "https://storage.googleapis.com/";

#[derive(StructOpt, Debug)]
#[structopt(name = "sharpener", about = "Sharpener CLI")]
#[structopt(rename_all = "kebab-case")]
enum Cli {
    #[structopt(name = "download", about = "Download an exercise")]
    Download {
        #[structopt(name = "token")]
        token: String,
    },
    #[structopt(about = "Run automated tests")]
    Test,
    #[structopt(about = "List pending submissions")]
    List,
    #[structopt(about = "Show a hint for the current exercise")]
    Hint,
    #[structopt(about = "Submit current exercise solution")]
    Submit,
    #[structopt(
        about = "Get the solution to the current exercise, and a new exercise of equivalent difficulty"
    )]
    Solution,
    #[structopt(name = "config", about = "Configure your user")]
    Config {
        #[structopt(name = "token")]
        token: String,
    },
}

fn create_client(config: Config) -> Result<Client> {
    use reqwest::header::{self, HeaderMap, HeaderValue};
    let value = HeaderValue::from_str(&config.token).context(InvalidToken {})?;
    let mut headers = HeaderMap::new();
    headers.append(header::AUTHORIZATION, value);
    Client::builder()
        .default_headers(headers)
        .build()
        .context(ClientBuild {})
}

fn run_cli() -> Result<()> {
    let args = Cli::from_args();
    if let Cli::Config { token } = args {
        Config::create(token)?;
        println!("Configuration saved successfully");
        return Ok(());
    }

    let config = Config::load()?;
    let url = reqwest::Url::parse(API_URI).unwrap();
    let client = create_client(config)?;

    match args {
        Cli::Download { token } => {
            let submission = Submission::get(client, &url, &token)?;
            submission.download()?;
        }
        Cli::List => {
            let pending_submissions =
                Submission::list(client, &url, Some(SubmissionStatus::Pending))?;

            for submission in pending_submissions {
                println!(
                    "{} - {} (Submission token: {})",
                    submission.exercise_language,
                    submission.exercise_name,
                    submission.submission_token
                );
            }
        }
        Cli::Test => {
            let (meta, _) = Meta::get()?;
            let mut test_command = meta.language.test_command();
            test_command
                .spawn()
                .and_then(|mut child| child.wait())
                .context(TestCommand {})?;
        }
        Cli::Hint => {
            let (mut meta, path) = Meta::get()?;
            match meta.hints.as_ref() {
                None => {
                    println!("This exercise has no hints. Good luck!");
                    return Ok(());
                }
                Some(hints) => {
                    let hints_seen = std::cmp::min(meta.hints_seen.unwrap_or(0) + 1, hints.len());
                    for (index, hint) in hints[..hints_seen].iter().enumerate() {
                        println!("Hint #{}:\n{}\n", index + 1, hint);
                    }

                    meta.hints_seen = Some(hints_seen);
                    meta.write(&path)?;
                }
            }
        }
        Cli::Solution => {
            let (meta, _) = Meta::get()?;
            let submission = Submission::forfeit(client, &url, &meta)?;
            println!(
                "Exercise successfully forteited. To download the replacement exercise, run the following command:\n\n\tsharpener download {}\n", 
                submission.submission_token
            );
        }
        Cli::Submit => {
            Submission::submit(client, &url)?;
        }
        Cli::Config { .. } => unreachable!(),
    }
    Ok(())
}

fn main() {
    match run_cli() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[derive(Deserialize, Debug)]
struct Exercise {
    creator: String,
    description: String,
    language: Language,
    name: String,
    readme: String,
    solution: String,
    starting_point: String,
    test: String,
    compressed: String,
    difficulty: Option<i32>,
    hint: Option<String>,
    topics: Option<Vec<String>>,
}
