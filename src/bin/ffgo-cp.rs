use clap::Parser;
use std::env;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(required = true, value_delimiter = ' ', num_args = 1..)]
  source: Vec<PathBuf>,
  target: String,
}

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  let mut temp_dir = PathBuf::new();
  temp_dir.push(env::temp_dir());
  temp_dir.push(env!("CARGO_BIN_NAME"));
  std::fs::create_dir(&temp_dir);

  let mut ready_file = temp_dir.clone();
  ready_file.push(".ready");
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