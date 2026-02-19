# ffgo

## Introduction
ffgo is a lightweight command-line utility designed to automate ffmpeg workflows. It acts as a thin orchestration layer, giving you full access to the power of ffmpeg while handling the execution logic.

Built to streamline transcoding tasks without adding unnecessary bulk, ffgo provides granular control at the command level. It’s an ideal tool for users who need to automate repetitive processing routines while maintaining the flexibility of the original ffmpeg suite.

## Status
This project is under active development and is offered as beta software. Configuration schema, command line arguments, etc. may change during this period until a stable 1.0 is released.

## Features
- Run a ffmpeg configuration on a directory of inputs.
- Run in watch mode. Observe a directory for changes and run a configuration on added files.
- Auto-map subtitle inputs and automatically encodes language metadata.
- Run workloads in a docker container.
- Cleanup inputs after job completion.

## Installation
### Prerequisites
One must first install [rust](https://rust-lang.org/tools/install/) and [ffmpeg](https://ffmpeg.org/download.html)

### Build from Source
```
git clone https://github.com/nokeeo/ffgo.git
cd ffgo
cargo install
```

### Docker Image
The repository contains a docker file one can use to generate an image. The image contains drivers for intel integrated graphics. If you want to leverage other graphics hardware you must edit this file to install the appropriate drivers.

To generate an image:
```
docker build -t ffgo .
```

An example container configuration in docker-compose:
```yaml
  ffgo-watch:
    build:
      context: # Path to git directory
      dockerfile: # Path to Dockerfile in git directory
    image: ffgo:local
    container_name: ffgo-watch
    restart: unless-stopped
    command: 
      - "ffgo"
      - "--jobs"
      - "2"
      - "watch"
      - "/transcodes/"
    devices:
      - /dev/dri:/dev/dri
    volumes:
      # Update volumes to reflect your input/output directories.
      # Note: output directory in job.toml must use the volume's file system path.
      - /docker/ffgo/transcodes:/transcodes
      - /docker/ffgo/output:/output
```


## Usage
ffgo operates on a directory of input and spawns an ffmeg job for each file group in the directory given a configuration file. Each directory must have one configuration file and at least one input file.

job.toml is the configuration file that describes how to run ffmpeg on the input files. In job.toml one can specify fields like the ffmpeg arguments, output directory, output file extension, etc. 

When running ffgo on a directory, input files are grouped by file group prefix. Files following the naming scheme, `[file_group]_[fileName].[extension]`. For example:
```
-- MyVideos
 -- video1.mkv
 -- video1_en.srt
 -- video2.mkv
 -- video2_en.srt
 -- job.toml
```

This will spawn two ffmpeg jobs. One for the video1 file group and another for the video2 file group. Input is mapped like `-i video1.mkv video1_en.srt`. The file that defines the file group name (video1.mkv) is always the first input, all other inputs are ordered alphabetically. 

See job.toml [documentation](./documentation/job.md) for how to configure your ffgo jobs.

To run ffgo on a directory use the command:
```
ffgo [PATH_TO_DIRECTORY]
```

## Watch folders
One can configure ffgo to recursively watch a directory file tree and spawn jobs for observed added files.

To start observing a directory use the watch subcommand:
```
ffgo watch [PATH_TO_DIRECTORY]
```

ffgo watch for the addition of a `.ready` file. Once this is observed, ffgo will run on the directory which the ready was added to. We recommend the following watch directory structure:
```
-- WatchDirectory
 -- job1 
   -- file1.mkv
   -- file2.mkv
   -- job.toml
   -- .ready
 -- job2
   -- file3.mkv
   -- job.toml
   -- .ready
 [and so on]
```

## ffgo-cp
To make adding files to a watch directory simpler, the ffgo package provides the utility `ffgo-cp`. This utility takes at least one path to a source file and an destination directory. All inputs will be copied to that directory and only after they are copied, a `.ready` file is created in the output directory. For example this command copies all the files in `~/Videos` to `/watchDirectory` and after creates a `.ready` file:

```
ffgo-cp ~/Videos/* /watchDirectory
```

ffgo-cp is built on top of scp and uses the same URI scheme for remote URIs. One can copy a file to a remote host running ffgo using:
```
ffgo-cp ~/Videos/* [user]@[hostName]:[path]
``` 

## License

This project is published under PPL. Under this license, individuals, non-profits, and worker cooperatives may use, copy, modify, and distribute this software. **Other commercial use is prohibited**. See the [license](./LICENSE) for more information.