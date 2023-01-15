use std::{fmt::Display, fs, path::PathBuf, str::FromStr};

use anyhow::anyhow;
use clap::{command, Parser, ValueEnum};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to transcode
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file to write
    #[arg(value_name = "OUTPUT", required_unless_present = "format")]
    output: Option<PathBuf>,

    /// Output format
    #[arg(value_enum, long, short)]
    format: Option<FileFormat>,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum FileFormat {
    Json,
    Yaml,
    Toml,
    Unknown,
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::Json => write!(f, "json"),
            FileFormat::Yaml => write!(f, "yml"),
            FileFormat::Toml => write!(f, "toml"),
            FileFormat::Unknown => write!(f, "txt"),
        }
    }
}

impl FromStr for FileFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            "toml" => Ok(Self::Toml),
            _ => Ok(Self::Unknown),
        }
    }
}

trait IO {
    fn path(&self) -> &PathBuf;
    fn format(&self) -> &FileFormat;
}

struct Input {
    path: PathBuf,
    format: FileFormat,
}

struct Output {
    path: PathBuf,
    format: FileFormat,
}

impl IO for Input {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn format(&self) -> &FileFormat {
        &self.format
    }
}

impl IO for Output {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn format(&self) -> &FileFormat {
        &self.format
    }
}

impl Input {
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

impl Output {
    fn new(path: PathBuf, format: Option<FileFormat>) -> Self {
        if let Some(format) = format {
            Self { path, format }
        } else {
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
}

pub fn start() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let input = Input::new(cli.clone().input);

    let output_path = match cli.output {
        None => {
            let mut path = cli.clone().input;
            path.set_extension(
                cli.format
                    .expect("If no output was given, we must have a format flag")
                    .to_string(),
            );
            path
        }
        Some(path) => path,
    };

    let output = Output::new(output_path, cli.format);

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

fn transcode(input: &impl IO, output: &impl IO) -> anyhow::Result<String> {
    match (input.format(), output.format()) {
        // Output YAML
        (FileFormat::Json, FileFormat::Yaml) => {
            let value =
                serde_json::from_str::<serde_yaml::Value>(&fs::read_to_string(input.path())?)?;
            Ok(serde_yaml::to_string(&value).unwrap())
        }
        (FileFormat::Toml, FileFormat::Yaml) => {
            let value = toml::from_str::<serde_yaml::Value>(&fs::read_to_string(input.path())?)?;
            Ok(serde_yaml::to_string(&value).unwrap())
        }

        // Output TOML
        (FileFormat::Json, FileFormat::Toml) => {
            let value = serde_json::from_str::<toml::Value>(&fs::read_to_string(input.path())?)?;
            Ok(toml::to_string(&value).unwrap())
        }
        (FileFormat::Yaml, FileFormat::Toml) => {
            let value = serde_yaml::from_str::<toml::Value>(&fs::read_to_string(input.path())?)?;
            Ok(toml::to_string(&value).unwrap())
        }

        // Output JSON
        (FileFormat::Yaml, FileFormat::Json) => {
            let value =
                serde_yaml::from_str::<serde_json::Value>(&fs::read_to_string(input.path())?)?;
            Ok(serde_json::to_string(&value).unwrap())
        }
        (FileFormat::Toml, FileFormat::Json) => {
            let value = toml::from_str::<serde_json::Value>(&fs::read_to_string(input.path())?)?;
            Ok(serde_json::to_string(&value).unwrap())
        }

        // Everything else
        (_, FileFormat::Unknown) => Err(anyhow!("Output format is unknown")),
        (FileFormat::Unknown, _) => Err(anyhow!("Input format is unknown")),
        (_, _) => Err(anyhow!("Invalid formats")),
    }
}
