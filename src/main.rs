use std::env;
use flate2::read::GzDecoder;
use serde::Deserialize;
use structopt::StructOpt;
use tar;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

static API_URI: &str = "http://sharpener-cloud.appspot.com/api";
static BUCKET_URI: &str = "https://storage.googleapis.com/";
type CliError = Box<dyn std::error::Error>;
#[derive(StructOpt, Debug)]
#[structopt(name = "sharpener", about = "Sharpener CLI")]
#[structopt(rename_all = "kebab-case")]
enum Cli {
    #[structopt(name = "download", about = "Download an exercise")]
    Download {
        #[structopt(name = "language")]
        language: String,
        #[structopt(name = "name")]
        name: String,
    },
    #[structopt(name = "config", about = "Configure your user")]
    Config {
        #[structopt(name = "token")]
        token: String,
    },
}

fn main() {
    let args = Cli::from_args();
    match args {
        Cli::Download {
            language: l,
            name: n,
        } => download_exercise(l, n).unwrap(),
        Cli::Config {
            token: t,
        } =>  store_token_in_home(t).unwrap(),
    }
}

#[derive(Deserialize, Debug)]
struct Exercise {
    creator: String,
    description: String,
    language: String,
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

fn download_exercise(lang: String, name: String) -> Result<(), CliError> {
    let request_uri = format!(
        "{base}/exercises/{lang}/{name}",
        base = API_URI,
        lang = lang,
        name = name
    );
    let exercise: Exercise = reqwest::get(&request_uri)?.json()?;
    let download_uri = exercise.compressed.replace("gs://", BUCKET_URI);
    println!("Downloading exercise {}, for language {}...", name, lang);
    let mut tar_file = download_tar(&download_uri)?;
    tar_file.unpack("./")?;
    println!("Downloaded {}.", name);
    Ok(())
}

type TarArchive = tar::Archive<GzDecoder<reqwest::Response>>;

fn download_tar(target: &str) -> Result<TarArchive, CliError> {
    let response = reqwest::get(target)?;
    let decoded_response = GzDecoder::new(response);
    Ok(tar::Archive::new(decoded_response))
}


fn store_token_in_home(token: String) -> std::io::Result<()>{
    let home = env::var("HOME").map(PathBuf::from).unwrap();
    let config_path = home.join(".sharpener-config");
    let mut file = File::create(config_path)?;
    file.write_all(token.as_bytes())?;
    Ok(())
}
