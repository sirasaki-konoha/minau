# minau
A command-line music player built with Rust using the *rodio* library.


## Quick usage

```
minau <path/to/music/files> [--volume <volume>]
```


## Details
minau is a lightweight command-line music player that uses the Rust *rodio* library. It is highly efficient and works even in resource-constrained environments.

### Command-line arguments

* **files: `<Vec<String>>`** — Accepts music files to play (multiple files can be specified).
* **volume** — Adjusts the playback volume. The maximum is 100 and the minimum is 1.

