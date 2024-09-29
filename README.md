<div align=center>

 [English](README.md) | [中文](README_CN.md) 

</div>

# Search Application

This is a Rust-based search application that provides both a command-line interface (CLI) and a graphical user interface (GUI) using `egui` and `eframe`. The application indexes files and allows users to search for files based on their content.

## Features

- **Command-Line Interface (CLI)**: Allows users to interact with the application via the terminal.
- **Graphical User Interface (GUI)**: Provides a user-friendly interface for searching files.
- **Automatic Indexing**: Periodically updates the file index in the background.
- **Customizable Search**: Users can set the root directory for indexing and perform searches based on file content.

## Installation

1. **Clone the repository**:
    ```sh
    git clone https://github.com/Yoimiya-Naganohara/search.git
    cd search
    ```

2. **Build the application**:
    ```sh
    cargo build --release
    ```

## Usage

### Command-Line Interface (CLI)

To run the application in CLI mode, provide any argument when running the executable:
```sh
./target/release/search_terminal some_argument
```

### Graphical User Interface (GUI)

To run the application in GUI mode, simply run the executable without any arguments:
```sh
./target/release/search
```

## Configuration

### Setting the Root Directory

In GUI mode, you can set the root directory for indexing by clicking the "Set" button and entering the desired directory path.

### Automatic Indexing

The application automatically updates the file index every 10 minutes. This is handled by a background thread.

## Code Overview

### main.rs

The [`main.rs`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2Fsrc%2Fmain.rs%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "d:\search\src\main.rs") file contains the entry point for the application. 

```rust
#![windows_subsystem = "windows"]
mod search_engine;

use search_engine::{Search, SearchEngine};
use std::sync::mpsc::channel;
use std::thread::{self, sleep};
use std::time::Duration;
mod ui_handle;
use ui_handle::{SearchApp, SearchAppEngine};

fn main() {
    run_gui_mode();
}

fn run_gui_mode() {
    let (send, recv) = channel();
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Search",
        native_options,
        Box::new(|cc| {
            let mut app = SearchApp::new(cc);
            app.set_sender(send);
            start_background_threads(recv);
            Ok(Box::new(app))
        }),
    );
}

fn start_background_threads(recv: std::sync::mpsc::Receiver<String>) {
    let mut engine = Search::new();
    thread::spawn(move || loop {
        if let Ok(received) = recv.recv() {
            engine.set_root_dir([received].iter().collect());
            engine.generate_index();
            engine.save_index();
            engine.clear_index_files();
        }
    });

    let mut engine = Search::new();
    thread::spawn(move || loop {
        sleep(Duration::from_secs(600));
        for path in 'A'..='Z' {
            engine.set_root_dir([format!("{}:\\", path)].iter().collect());
            engine.generate_index();
            engine.save_index();
            engine.clear_index_files();
        }
    });
}
```

### search_engine.rs

Contains the implementation of the [`Search`](command:_github.copilot.openSymbolFromReferences?%5B%22%22%2C%5B%7B%22uri%22%3A%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2Fsrc%2Fmain.rs%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%22pos%22%3A%7B%22line%22%3A30%2C%22character%22%3A21%7D%7D%5D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "Go to definition") and `SearchEngine` structs, which handle file indexing and searching.

### ui_handle.rs

Contains the implementation of the [`SearchApp`](command:_github.copilot.openSymbolFromReferences?%5B%22%22%2C%5B%7B%22uri%22%3A%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2Fsrc%2Fmain.rs%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%22pos%22%3A%7B%22line%22%3A21%2C%22character%22%3A26%7D%7D%5D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "Go to definition") struct, which handles the GUI logic using `egui` and `eframe`.

## License

This project is licensed under the MIT License. See the [`LICENSE`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2FLICENSE%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "d:\search\LICENSE") file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Acknowledgements

- [egui](https://github.com/emilk/egui) for the GUI framework.
- [eframe](https://github.com/emilk/eframe) for the application framework.
