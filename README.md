# Xlsxc

## Attention!
Currently xlsxc is only tested on OSX, but probably runs on other UNIX like operating systems like linux.

## Dependencies
### BUILD
Xlsxc is written in rust and therefore needs to be build from source via cargo.

### CLI
The converter calls the dependent libs via the command line. Make sure to have the needed dependencies as cli tools installed.
- [ty-dlp](https://github.com/yt-dlp/yt-dlp)
- [ffmpeg](https://www.ffmpeg.org)

## What the code does
1. Parse command line input
2. Dowloads the src video from provided url
3. Extracts the frames of the video
4. Reads pixel color values of each frame
6. Write all pixel color values (rgb) for each frame into its own cell