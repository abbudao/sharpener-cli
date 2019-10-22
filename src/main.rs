use serde::Deserialize;
use structopt::StructOpt;
use tar;

static API_URI: &str = "http://localhost:5000/api";
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
        _ => println!("Oh noes"),
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
    println!("{:?}", download_uri);
    let mut tar_file = download_tar(&download_uri)?;
    tar_file.unpack("./")?;
    println!("Success");
    Ok(())
}

type TarArchive = tar::Archive<reqwest::Response>;

fn download_tar(target: &str) -> Result<TarArchive, CliError> {
    let response = reqwest::get(target)?;
    Ok(tar::Archive::new(response))
}

