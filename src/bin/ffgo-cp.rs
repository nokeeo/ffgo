use clap::Parser;
use ffgo::file_constants::{JOB_CONFIG_FILE_NAME, READY_FILE_NAME};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(required = true, value_delimiter = ' ', num_args = 1..)]
  source: Vec<PathBuf>,
  target: String,
}

fn files_contain_job_config(files: &Vec<PathBuf>) -> bool {
  files.iter()
    .filter(|p| p.file_name() == Some(*JOB_CONFIG_FILE_NAME))
    .collect::<Vec<_>>()
    .len() > 0
}

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  if !files_contain_job_config(&args.source) {
    let mut buffer = String::new();
    let stdin = io::stdin();

    println!("Source files are missing {:?}. Would you like to proceed? (y/n)", *JOB_CONFIG_FILE_NAME);
    stdin.read_line(&mut buffer);
    if buffer.trim() != "y" {
      return Ok(());
    }
  }

  let mut temp_dir = PathBuf::new();
  temp_dir.push(env::temp_dir());
  temp_dir.push(env!("CARGO_BIN_NAME"));
  std::fs::create_dir(&temp_dir);

  let mut ready_file = temp_dir.clone();
  ready_file.push(*READY_FILE_NAME);
  let _ = File::create_new(&ready_file);
  // TODO - Only fail if AlreadyExists.

  let mut command = Command::new("scp");
  command.args(&args.source);
  command.arg(&ready_file);
  command.arg(&args.target);

  let mut handle = command.spawn()?;
  handle.wait();

  // Cleanup.
  std::fs::remove_file(&ready_file);
  Ok(())
}