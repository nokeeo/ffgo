use std::env;
use notify::Watcher;
use notify::{EventKind, event};
use std::{path::Path, process::Command, sync::mpsc};
use std::fs::DirEntry;

fn isVideoFile(entry: &DirEntry) -> bool {
    if let Some(extension) = entry.path().extension().and_then(|e| e.to_str()) {
        return match extension {
            "mov" | "m4v" | "mkv" | "mp4" => true,
            _ => false
        };
    }
    false
}

fn main() -> notify::Result<()>{
    let args: Vec<String> = env::args().collect();
    let paths = std::fs::read_dir(&args[1]).expect("Cannot get files at given path")
        .filter(|entry| isVideoFile(entry.as_ref().expect("Failed to get entry")));
    for entry in paths {
        println!("Starting encode of file");
        let path = entry.expect("Failed to get entry").path();
        let output = Command::new("ffmpeg")
            .args(["-vaapi_device", "/dev/dri/renderD128"]) 
            .args(["-i", path.to_str().expect("Failed to get file path")])
            .args(["-map", "0:v:0", "-map", "0:a:0", "-map", "0:s?"])
            .args(["-vf", "\"format=p010,hwupload\""])
            .args(["-c:v", "hevc_vaapi"])
            .args(["-profile:v",  "main10"])
            .args(["-rc_mode",  "1"])
            .args(["-qp", "22"])
            .args(["-tag:v", "hvc1"])
            .args(["-color_primaries", "9", "-color_trc", "16", "-colorspace", "9"])
            .args(["-c:a", "aac", "-ac", "2"])
            .args(["-c:s", "mov_text"])
            .args(["-metadata:s:s:0", "title=\"English\""])
            .args(["-metadata:s:s:0", "language=eng"])
            .arg(format!("./{}.mp4", path.file_stem().expect("Failed to get file name").to_str().expect("Failed to get file name")))
            .output()
            .expect("Failed to execute ffmpeg");
        
        println!("{:?}", output);
    }

    // let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

    // let mut watcher = notify::recommended_watcher(tx)?;
    // let path = Path::new(&args[1]);
    // watcher.watch(&path, notify::RecursiveMode::Recursive)?;
    // for res in rx {
    //     if let Ok(event) = res {
    //         println!("{:?}", event);
    //         match event.kind {
    //             EventKind::Modify(event::ModifyKind::Data(_)) => {
    //                 println!("Modiy event");
    //                 }
    //             },
    //             _ => {}
    //         }
    //     }
    // }
    Ok(())
}
