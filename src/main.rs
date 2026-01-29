use ffgo::Config;
use ffgo::job::get_jobs;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use ffgo::job_queue::Job;
use ffgo::job_queue::JobQueue;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> { 
    let args: Vec<String> = env::args().collect();
    let mut dir_path = PathBuf::new();
    dir_path.push(&args[1]);
    let config = fs::read_to_string(format!("{}/job.toml", &args[1]))?;
    let config: Config = toml::from_str(&config)?;

    let queue = JobQueue::new();
    let mut job_handles = Vec::<oneshot::Receiver<bool>>::new();
    for (_, mut files) in get_jobs(dir_path.as_path()) {
        files.sort();
        println!("pusing: {:?}", files);
        job_handles.push(queue.push(Job::new(config.clone(), files)).await);
    }

    for handle in job_handles {
        let result = handle.await?;
        println!("Result: {}", result);
    }

    Ok(())
}
