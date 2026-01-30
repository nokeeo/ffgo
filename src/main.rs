use clap::Parser;
use ffgo::Config;
use ffgo::jobs;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: String,

    #[arg(short, long, default_value_t = 1)]
    jobs: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> { 
    let args = Args::parse();
    let mut dir_path = PathBuf::new();
    dir_path.push(&args.path);
    let config = fs::read_to_string(format!("{}/job.toml", &args.path))?;
    let config: Config = toml::from_str(&config)?;
    let queue = jobs::Queue::new(args.jobs);
    queue.push_directory(dir_path.as_path(), &config).await;
    Ok(())
}
