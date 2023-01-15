use std::{fs, path::PathBuf};

use anyhow::anyhow;
use clap::{command, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to transcode
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file to write
    #[arg(value_name = "OUTPUT")]
    output: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum FileFormat {
    Json,
    Yaml,
    Toml,
    Unknown,
}

struct IO {
    path: PathBuf,
    format: FileFormat,
}

impl IO {
    fn new(path: PathBuf) -> Self {
        let format = match path.extension() {
            None => FileFormat::Unknown,
            Some(os_str) => match os_str.to_str() {
                Some("json") => FileFormat::Json,
                Some("toml") => FileFormat::Toml,
                Some("yaml") | Some("yml") => FileFormat::Yaml,
                _ => FileFormat::Unknown,
            },
        };

        Self { path, format }
    }
}
pub fn start() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let input = IO::new(cli.input);
    let output = IO::new(cli.output);

    if input.format == output.format {
        return Err(anyhow!("Input format is the same as output format."));
    }

    let content = transcode(&input, &output)?;

    if fs::write(&output.path, content).is_ok() {
        println!(
            "Wrote {} to {}",
            input.path.to_str().unwrap(),
            output.path.to_str().unwrap()
        );
        Ok(())
    } else {
        Err(anyhow!("Failed to write file"))
    }
}

fn transcode(input: &IO, output: &IO) -> anyhow::Result<String> {
    match (input.format, output.format) {
        // Output YAML
        (FileFormat::Json, FileFormat::Yaml) => {
            let value =
                serde_json::from_str::<serde_yaml::Value>(&fs::read_to_string(&input.path)?)?;
            Ok(serde_yaml::to_string(&value).unwrap())
        }
        (FileFormat::Toml, FileFormat::Yaml) => {
            let value = toml::from_str::<serde_yaml::Value>(&fs::read_to_string(&input.path)?)?;
            Ok(serde_yaml::to_string(&value).unwrap())
        }

        // Output TOML
        (FileFormat::Json, FileFormat::Toml) => {
            let value = serde_json::from_str::<toml::Value>(&fs::read_to_string(&input.path)?)?;
            Ok(toml::to_string(&value).unwrap())
        }
        (FileFormat::Yaml, FileFormat::Toml) => {
            let value = serde_yaml::from_str::<toml::Value>(&fs::read_to_string(&input.path)?)?;
            Ok(toml::to_string(&value).unwrap())
        }

        // Output JSON
        (FileFormat::Yaml, FileFormat::Json) => {
            let value =
                serde_yaml::from_str::<serde_json::Value>(&fs::read_to_string(&input.path)?)?;
            Ok(serde_json::to_string(&value).unwrap())
        }
        (FileFormat::Toml, FileFormat::Json) => {
            let value = toml::from_str::<serde_json::Value>(&fs::read_to_string(&input.path)?)?;
            Ok(serde_json::to_string(&value).unwrap())
        }

        // Everything else
        (_, FileFormat::Unknown) => Err(anyhow!("Output format is unknown")),
        (FileFormat::Unknown, _) => Err(anyhow!("Input format is unknown")),
        (_, _) => Err(anyhow!("Invalid formats")),
    }
}
