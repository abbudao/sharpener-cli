use crate::error::*;
use crate::language::Language;
use crate::meta::Meta;
use flate2::read::GzDecoder;
use reqwest::{multipart::Form, Client, StatusCode, Url};
use serde::{de::Error as DeserializeError, Deserialize, Deserializer};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::Path;
use tar;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SubmissionStatus {
    Skipped,
    Submitted,
    Pending,
}

impl<'de> Deserialize<'de> for SubmissionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use SubmissionStatus::*;
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "skipped" => Ok(Skipped),
            "submitted" => Ok(Submitted),
            "pending" => Ok(Pending),
            _ => Err(DeserializeError::unknown_variant(
                &value,
                &["skipped", "submitted", "pending"],
            )),
        }
    }
}

#[derive(Deserialize)]
pub struct Submission {
    pub exercise_name: String,
    pub exercise_language: Language,
    pub download_url: String,
    pub submission_token: String,
    pub attempts: i32,
    pub submission_status: SubmissionStatus,
}

#[derive(Deserialize)]
struct ForfeitSubmission {
    pub success: bool,
    pub data: Option<Submission>,
}

impl Submission {
    pub fn list(client: Client, api: &Url, status: Option<SubmissionStatus>) -> Result<Vec<Self>> {
        let url = api.join("submissions").unwrap();
        let result = client
            .get(url)
            .send()
            .context(ServerRequest {})
            .and_then(|response| {
                ensure!(
                    response.status() == StatusCode::OK,
                    InvalidAPIResponse {
                        expected: StatusCode::OK,
                        received: response.status()
                    }
                );
                serde_json::from_reader(response).context(ParseSubmissionResponse {})
            });

        match status {
            Some(s) => result.map(|submissions: Vec<Self>| {
                submissions
                    .into_iter()
                    .filter(|sub| sub.submission_status == s)
                    .collect()
            }),
            None => result,
        }
    }

    pub fn get(client: Client, api: &Url, token: &str) -> Result<Self> {
        let suffix = format!("submissions/{}", token);
        let url = api.join(&suffix).unwrap();
        client
            .get(url)
            .send()
            .context(ServerRequest {})
            .and_then(|response| {
                ensure!(
                    response.status() == StatusCode::OK,
                    InvalidAPIResponse {
                        expected: StatusCode::OK,
                        received: response.status()
                    }
                );
                serde_json::from_reader(response).context(ParseSubmissionResponse {})
            })
    }

    pub fn download(&self) -> Result<()> {
        let download = reqwest::get(&self.download_url).context(ExerciseDownload {})?;
        let decoded = GzDecoder::new(download);
        let mut archive = tar::Archive::new(decoded);
        archive.unpack("./").context(UnpackTar {})?;

        let directory_path = Path::new(&self.exercise_name);
        let meta_path = directory_path.join(".meta.json");
        let mut meta_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(meta_path)
            .context(OpenMetaFile {})?;

        let mut meta: Meta = serde_json::from_reader(&meta_file).context(ParseMetaFile {})?;
        meta.submission_token = Some(self.submission_token.clone());
        meta.hints_seen = Some(0);
        meta_file
            .seek(SeekFrom::Start(0))
            .context(OpenMetaFile {})?;
        serde_json::to_writer_pretty(&meta_file, &meta).context(WriteMetaFile {})?;
        Ok(())
    }

    pub fn forfeit(client: Client, api: &Url, meta: &Meta) -> Result<Self> {
        let suffix = format!(
            "submissions/{}/forfeit",
            &meta.submission_token.as_ref().context(MissingMeta {})?
        );
        let url = api.join(&suffix).unwrap();
        let response = client.post(url).send().context(ServerRequest {})?;
        ensure!(
            response.status() == StatusCode::OK,
            InvalidAPIResponse {
                expected: StatusCode::OK,
                received: response.status()
            }
        );
        let forfeit: ForfeitSubmission =
            serde_json::from_reader(response).context(ParseSubmissionResponse {})?;
        match (forfeit.success, forfeit.data) {
            (true, Some(submission)) => Ok(submission),
            _ => Err(Error::InvalidForfeit),
        }
    }

    pub fn submit(client: Client, api: &Url) -> Result<()> {
        let (meta, path) = Meta::get()?;
        let token = meta.submission_token.context(MissingMeta)?;

        println!("Running tests");
        let output = meta.language.test_command().output().context(TestCommand)?;

        let test_output = String::from_utf8(output.stdout).context(InvalidTestOutput)?;

        let test_coverage = meta.language.parse_test_coverage(&test_output);
        let parent = path.parent().unwrap();
        let test_file_path = parent.join(meta.language.test_file_path());
        let test_file_checksum = checksum_file(&test_file_path)?;

        let solution_file_path = parent.join(meta.language.solution_file_path());

        let form = Form::new()
            .text("test_output", test_output)
            .text("test_coverage", test_coverage)
            .text("test_checksum", test_file_checksum)
            .file("solution", &solution_file_path)
            .context(OpenSubmissionFile {
                filename: solution_file_path,
            })?;

        println!("Submitting results");
        let suffix = format!("submissions/{}", token);
        let url = api.join(&suffix).unwrap();
        let response = client
            .post(url)
            .multipart(form)
            .send()
            .context(ServerRequest)?;

        ensure!(
            response.status() == StatusCode::OK,
            InvalidAPIResponse {
                expected: StatusCode::OK,
                received: response.status()
            }
        );

        Ok(())
    }
}

fn checksum_file(path: &Path) -> Result<String> {
    use md5::Context;
    use std::io::{BufRead, BufReader};

    let file = File::open(path).with_context(|| OpenSubmissionFile {
        filename: path.to_path_buf(),
    })?;
    let mut buffed_file = BufReader::new(file);

    let mut context = Context::new();

    loop {
        let buffer = buffed_file.fill_buf().with_context(|| ReadSubmissionFile {
            filename: path.to_path_buf(),
        })?;

        if buffer.is_empty() {
            return Ok(format!("{:x}", context.compute()));
        }

        context.consume(&buffer);
        let consumed = buffer.len();
        buffed_file.consume(consumed);
    }
}
