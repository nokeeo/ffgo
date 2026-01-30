use ffgo::Config;
use ffgo::jobs;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> { 
    let args: Vec<String> = env::args().collect();
    let mut dir_path = PathBuf::new();
    dir_path.push(&args[1]);
    let config = fs::read_to_string(format!("{}/job.toml", &args[1]))?;
    let config: Config = toml::from_str(&config)?;
    let queue = jobs::Queue::new();
    queue.push_directory(dir_path.as_path(), &config).await;
    Ok(())
}
