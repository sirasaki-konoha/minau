# minau

<p align="center">
  <img src="https://raw.githubusercontent.com/sirasaki-konoha/minau/refs/heads/master/icon/minau-icon.png" alt="minau icon" />
</p>

<p align="center">
    <a href="https://deepwiki.com/sirasaki-konoha/minau"><img alt="DeepWiki" src="https://img.shields.io/badge/DeepWiki-sirasaki--konoha%2Fminau-blue.svg?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACwAAAAyCAYAAAAnWDnqAAAAAXNSR0IArs4c6QAAA05JREFUaEPtmUtyEzEQhtWTQyQLHNak2AB7ZnyXZMEjXMGeK/AIi+QuHrMnbChYY7MIh8g01fJoopFb0uhhEqqcbWTp06/uv1saEDv4O3n3dV60RfP947Mm9/SQc0ICFQgzfc4CYZoTPAswgSJCCUJUnAAoRHOAUOcATwbmVLWdGoH//PB8mnKqScAhsD0kYP3j/Yt5LPQe2KvcXmGvRHcDnpxfL2zOYJ1mFwrryWTz0advv1Ut4CJgf5uhDuDj5eUcAUoahrdY/56ebRWeraTjMt/00Sh3UDtjgHtQNHwcRGOC98BJEAEymycmYcWwOprTgcB6VZ5JK5TAJ+fXGLBm3FDAmn6oPPjR4rKCAoJCal2eAiQp2x0vxTPB3ALO2CRkwmDy5WohzBDwSEFKRwPbknEggCPB/imwrycgxX2NzoMCHhPkDwqYMr9tRcP5qNrMZHkVnOjRMWwLCcr8ohBVb1OMjxLwGCvjTikrsBOiA6fNyCrm8V1rP93iVPpwaE+gO0SsWmPiXB+jikdf6SizrT5qKasx5j8ABbHpFTx+vFXp9EnYQmLx02h1QTTrl6eDqxLnGjporxl3NL3agEvXdT0WmEost648sQOYAeJS9Q7bfUVoMGnjo4AZdUMQku50McDcMWcBPvr0SzbTAFDfvJqwLzgxwATnCgnp4wDl6Aa+Ax283gghmj+vj7feE2KBBRMW3FzOpLOADl0Isb5587h/U4gGvkt5v60Z1VLG8BhYjbzRwyQZemwAd6cCR5/XFWLYZRIMpX39AR0tjaGGiGzLVyhse5C9RKC6ai42ppWPKiBagOvaYk8lO7DajerabOZP46Lby5wKjw1HCRx7p9sVMOWGzb/vA1hwiWc6jm3MvQDTogQkiqIhJV0nBQBTU+3okKCFDy9WwferkHjtxib7t3xIUQtHxnIwtx4mpg26/HfwVNVDb4oI9RHmx5WGelRVlrtiw43zboCLaxv46AZeB3IlTkwouebTr1y2NjSpHz68WNFjHvupy3q8TFn3Hos2IAk4Ju5dCo8B3wP7VPr/FGaKiG+T+v/+TQqIrOqMTL1VdWV1DdmcbO8KXBz6esmYWYKPwDL5b5FA1a0hwapHiom0r/cKaoqr+27/XcrS5UwSMbQAAAABJRU5ErkJggg=="/></a>
    <a href="https://crates.io/crates/minau"><img src="https://img.shields.io/crates/v/minau.svg"></a>
    <img alt="Crates.io License" src="https://img.shields.io/crates/l/minau"/>
</p>



A lightweight, efficient command-line music player built with Rust using *symphonia* for decoding and *cpal* for audio output.

## Features

- 🎵 **Simple and Fast** - Minimal overhead, quick startup time
- 📊 **Volume Control** - Adjustable playback volume from command line
- 🎼 **Multiple Format Support** - Supports common audio formats (MP3, WAV, FLAC, OGG, etc.)
- 🖼️ **Album Art Display** - View album artwork during playback with GUI mode
- 💻 **Cross-platform** - Works on Windows, macOS, and Linux
- ⚡ **Low Resource Usage** - Efficient even in resource-constrained environments
- 🎛️ **Easy to Use** - Intuitive command-line interface
- ⌨️ **Keyboard Controls** - Pause/play, skip tracks, adjust volume, and seek through tracks
- 📃 **M3U Playlist Support** - Play music from M3U playlist files
- 🔄 **Async Support** - Built with async/await for better performance
- ⏩ **Seek Control** - Jump forward or backward within tracks using keyboard shortcuts
- 🌐 **URL Streaming** - Stream audio directly from HTTP/HTTPS URLs

## Installation

### From crates.io (Recommended)

The easiest way to install minau is via cargo:

```bash
cargo install minau
```

This will install the latest stable version from [crates.io](https://crates.io/crates/minau).

### From Source

```bash
git clone https://github.com/sirasaki-konoha/minau.git
cd minau
cargo install --path .
```

## Usage

### Basic Usage

Play a single audio file:

```bash
minau path/to/music.mp3
```

Play multiple audio files:

```bash
minau song1.mp3 song2.mp3 song3.flac
```

Play all audio files in a directory:

```bash
minau path/to/music/folder/*
```

### URL Streaming

Stream audio directly from URLs:

```bash
minau https://example.com/audio.mp3
```

Stream with volume control:

```bash
minau https://example.com/stream.mp3 --volume 50
```

Multiple URL support:

```bash
minau https://example.com/1.mp3 https://example.com/2.mp3
```

### Volume Control

Set playback volume (1-100):

```bash
minau music.mp3 --volume 50
```

Maximum volume:

```bash
minau music.mp3 --volume 100
```

Minimum volume:

```bash
minau music.mp3 --volume 1
```

### GUI Mode

Display album artwork during playback:

```bash
minau music.mp3 --gui
```

Combine with volume control:

```bash
minau music.mp3 --gui --volume 75
```

### Keyboard Controls

During playback, you can use the following keyboard shortcuts:

| Key | Action |
|-----|--------|
| `Space` | Toggle pause/play |
| `q` | Quit the player |
| `Right Key` or `Shift + .` (>) | Skip to next track |
| `-` or `_` or `j` | Decrease volume by 5 |
| `+` or `=` or `k` | Increase volume by 5 |
| `h` | Seek backward by 5 seconds |
| `l` | Seek forward by 5 seconds |

### Examples

```bash
# Play a single file at 75% volume
minau ~/Music/favorite.mp3 --volume 75

# Play with GUI mode to display album art
minau ~/Music/favorite.mp3 --gui

# Play multiple files with GUI
minau song1.mp3 song2.wav song3.flac --gui

# Play all MP3 files in current directory
minau *.mp3

# Play with minimum volume
minau quiet-music.mp3 --volume 1

# GUI mode with custom volume
minau album.flac --gui --volume 60
```

## Command-line Arguments
- **`<FILES>...`** - One or more audio files or URLs to play (required)
  - Type: `Vec<String>`
  - Accepts file paths and HTTP/HTTPS URLs
  - Supports various audio formats including M3U playlists
  - Can handle both absolute and relative paths
  - Can stream audio from remote URLs

- **`--volume <VOLUME>, -v`** - Playback volume level (optional)
  - Type: `u16`
  - Range: 1-100
  - Default: 100 (maximum volume)
  - Values outside range will cause an error

- **`--gui, -g`** - Enable GUI mode to display album artwork (optional)
  - Shows embedded album art from audio file metadata
  - Works with files that have embedded cover images
  - Uses a native window for display

## Supported Audio Formats

minau supports a wide range of audio formats through the symphonia library:

- MP3
- WAV
- FLAC
- OGG Vorbis
- AAC
- And many more formats supported by symphonia

## Requirements

- Rust 1.82.0 or later (for building from source)
  - **Note:** This project uses Rust 2024 edition and requires a recent Rust version
- A system audio output device
- Cross-platform support: Windows, macOS, and Linux

## Architecture

minau uses a modular architecture with direct audio processing:

- **symphonia** - High-performance audio decoding library supporting various formats
- **cpal** - Cross-platform audio library for low-latency audio output
- Direct audio pipeline for minimal overhead and maximum performance

## Performance

minau is designed to be lightweight and efficient:

- Fast startup time
- Minimal memory footprint
- Low CPU usage during playback
- Direct audio pipeline without unnecessary abstractions
- Suitable for resource-constrained environments (e.g., embedded systems, older hardware)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the BSD-3-Clause License - see the [LICENSE](LICENSE) file for details.


## Author

[@sirasaki-konoha](https://github.com/sirasaki-konoha)

---

**Note:** Use keyboard controls during playback for pause/play, volume adjustment, and track navigation.
