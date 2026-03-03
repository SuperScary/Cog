# Cog

Cog is a lightweight, terminal-based text editor written in Rust. It aims to provide a fast and efficient editing experience with essential features like syntax highlighting, multi-level undo/redo, and cross-platform clipboard support.

## Features

- **Lightweight & Fast**: Minimal overhead, built with Rust for performance.
- **Syntax Highlighting**: Built-in support for multiple languages including C, C++, Java, Rust, and Cloth.
- **Editing Essentials**:
    - Multi-level Undo/Redo history.
    - Text selection (using Shift + Arrow keys).
    - Cut, Copy, and Paste with system clipboard integration.
- **Encoding Support**: Automatic encoding detection (using `chardetng`) and support for various character sets.
- **Terminal Interface**: Powered by `crossterm` for a responsive and cross-platform TUI.

## Installation

To build Cog from source, you need to have [Rust and Cargo installed](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/SuperScary/Cog.git
cd cog
cargo build --release
```

The binary will be available at `target/release/cog`.

## Usage

Run Cog by specifying a file path or start with an empty document:

```bash
# Open an existing file
cog path/to/file.rs

# Start a new file
cog
```

### Key Bindings

| Key                | Action                             |
|--------------------|------------------------------------|
| **Arrow Keys**     | Move caret                         |
| **Shift + Arrows** | Select text                        |
| **Ctrl + S**       | Save file                          |
| **Ctrl + C**       | Copy selection                     |
| **Ctrl + X**       | Cut selection                      |
| **Ctrl + V**       | Paste from clipboard               |
| **Ctrl + Z**       | Undo                               |
| **Ctrl + Y**       | Redo                               |
| **Ctrl + Q**       | Quit (prompts to save if modified) |
| **Home / End**     | Move to start/end of line          |
| **Backspace**      | Delete character before caret      |
| **Delete**         | Delete character at caret          |

## Project Structure

- `src/`: Core editor logic, buffer management, and TUI rendering.
- `languages/`: Syntax definition and configuration files for supported languages.
- `Tests/`: Test files for various encodings and formats.

## License

This project is licensed under the [Apache 2.0](LICENSE).

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to Cog.