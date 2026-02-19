# job.toml

job.toml is the configuration file that describes how to run ffmpeg on the input files. This document describes the supported fields.

```toml
# An array of arguments passed to ffmpeg.
#
# Required
args = [
  '-map 0:v:0 -map 0:a:0',
  '-c:v libx265',
]

# An inline table that describes the output of the job.
#
# Required
output = {

  # The directory in which to place output files for a job. If using a docker container this must
  # be a path accessible in the container.
  #
  # Required
  directory = './'

  # The extension for all output files of a job. The string should not contain '.' delimiter.
  #
  # Required
  extension = 'mp4',
}

# If true, any input files that are subtitles will have their mappings automatically to to the
# ffmpeg command.
# 
# Subtitles are mapped in alphabetic order. If the file name postfix includes the
# ISO 639 Set 1 language code[1], subtitle title metadata is automatically encoded.
# For example for the given input files, video.mkv and video_en.srt the following ffmpeg args are added when subtitle auto mapping is enabled:
# '-map 1:s:0 -metadata:s:s1 language=eng'
#
# Optional
# [1] https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes
auto_map_external_subtitle_inputs = false
```