use clap::Parser;
use ffgo::file_constants::{JOB_CONFIG_FILE_NAME, READY_FILE_NAME};
use once_cell::sync::Lazy;
use uuid::Uuid;
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub static SCP_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new("^(?P<user>[a-z_][a-z0-9_-]*)@(?P<host>[a-zA-Z0-9.-]+)(?::(?P<path>.*))?$").unwrap()
});

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  
  /** Makes the destination path if it does not already exist. */
  #[arg(short)]
  path_make: bool,

  /** One or more files to copy. */
  #[arg(required = true, num_args = 1..)]
  source: Vec<PathBuf>,

  /** The destination of the copied file. Accepts scp remote URI scheme. */
  target: String,
}

#[derive(Debug)]
struct RemoteUrl {
  user: String,
  host: String,
  path: Option<PathBuf>,
}

impl RemoteUrl {
  fn parse(str: &str) -> Result<RemoteUrl, String> {
    let Some(captures) = SCP_URL_REGEX.captures(str) else { 
      return Err("Failed to parse remove URL".to_string());
    };

    let path = captures.name("path").map(|m| m.as_str());
    let path = match path {
      Some(path_str) => {
        let mut path_buf = PathBuf::new();
        path_buf.push(path_str);
        Some(path_buf)
      },
      _ => None
    };

    Ok(RemoteUrl {
      user: captures["user"].to_owned(),
      host: captures["host"].to_owned(),
      path: path,
    })
  }
}

fn files_contain_job_config(files: &Vec<PathBuf>) -> bool {
  files.iter()
    .filter(|p| p.file_name() == Some(*JOB_CONFIG_FILE_NAME))
    .count() > 0
}

fn make_dir_if_needed(target: &str) -> io::Result<()> {
  match RemoteUrl::parse(target) {
    Ok(remote_url) => {
      let Some(path) = remote_url.path else { 
        return Err(io::Error::new(io::ErrorKind::NotFound, "Remove URL does not have a path"));
      };
      println!("Creating directory {:?}", path);
      let mut command = Command::new("ssh");
      command.arg(format!("{}@{}", remote_url.user, remote_url.host));
      command.arg("mkdir -p");
      command.arg(path);

      let mut handle = command.spawn()?;
      handle.wait()?;
    },
    _ => {
      std::fs::create_dir_all(target)?;
    },
  }
  Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  if !files_contain_job_config(&args.source) {
    let mut buffer = String::new();
    let stdin = io::stdin();

    println!("Source files are missing {:?}. Would you like to proceed? (y/n)", *JOB_CONFIG_FILE_NAME);
    stdin.read_line(&mut buffer)?;
    if buffer.trim() != "y" {
      return Ok(());
    }
  }

  if args.path_make {
    println!("Creating parent directory if needed");
    make_dir_if_needed(&args.target)?;
  }

  let mut temp_dir = PathBuf::new();
  temp_dir.push(env::temp_dir());
  temp_dir.push(env!("CARGO_BIN_NAME"));
  temp_dir.push(Uuid::new_v4().to_string());
  std::fs::create_dir_all(&temp_dir)?;

  let mut ready_file = temp_dir.clone();
  ready_file.push(*READY_FILE_NAME);
  let _ = File::create_new(&ready_file);
  // TODO - Only fail if AlreadyExists.

  let mut command = Command::new("scp");
  command.args(&args.source);
  command.arg(&ready_file);
  command.arg(&args.target);

  let mut handle = command.spawn()?;
  handle.wait()?;

  // Cleanup.
  println!("Cleaning up temporary files");
  std::fs::remove_file(&ready_file)?;
  Ok(())
}