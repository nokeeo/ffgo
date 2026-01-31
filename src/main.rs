use clap::{Parser, Subcommand};
use ffgo::Config;
use ffgo::jobs;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: Option<String>,

    #[arg(short, long, default_value_t = 1)]
    jobs: usize,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Watch {
        path: PathBuf 
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> { 
    let args = Args::parse();
    match &args.command {
        Some(Command::Watch { path }) => {
            // TODO: Implement folder watch.
            println!("Not implemented");
        }

        None => {
            let Some(path) = &args.path else {
                return Err("Must specify a PATH".into());
            };
            let mut dir_path = PathBuf::new();
            dir_path.push(path);
            let config = fs::read_to_string(format!("{}/job.toml", path))?;
            let config: Config = toml::from_str(&config)?;
            let queue = jobs::Queue::new(args.jobs);
            queue.push_directory(dir_path.as_path(), &config).await;
        }
    }
    Ok(())
}
