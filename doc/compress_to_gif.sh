#!/bin/bash

# Script to compress video to 10-second GIF with 5x speed

if [ $# -eq 0 ]; then
    echo "Usage: $0 <input_video> [output_gif]"
    echo "Example: $0 recording.mp4 output.gif"
    exit 1
fi

INPUT="$1"
OUTPUT="${2:-compressed_output.gif}"

# Check if input file exists
if [ ! -f "$INPUT" ]; then
    echo "Error: Input file '$INPUT' not found!"
    exit 1
fi

# Check if ffmpeg is installed
if ! command -v ffmpeg &> /dev/null; then
    echo "Error: ffmpeg is not installed. Please install it first:"
    echo "  sudo apt-get install ffmpeg  # On Ubuntu/Debian"
    echo "  brew install ffmpeg          # On macOS"
    exit 1
fi

echo "Converting '$INPUT' to 10-second GIF..."

# Convert to GIF with:
# - setpts=0.2*PTS: Speed up 5x (50s -> 10s)
# - fps=15: Reasonable frame rate for GIF
# - scale=800:-1: Width 800px, maintain aspect ratio
# - flags=lanczos: High quality scaling
# - split/palettegen/paletteuse: Generate optimal palette for better quality
ffmpeg -i "$INPUT" -vf "setpts=0.2*PTS,fps=15,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" -t 10 "$OUTPUT" -y

if [ $? -eq 0 ]; then
    echo "Success! GIF saved as '$OUTPUT'"
    echo "File size: $(du -h "$OUTPUT" | cut -f1)"
else
    echo "Error: Conversion failed!"
    exit 1
fi