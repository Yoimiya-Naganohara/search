<div align=center>

 [English](README.md) | [中文](README_CN.md) 

</div>

# 搜索应用程序

这是一个基于 Rust 的搜索应用程序，提供命令行界面 (CLI) 和图形用户界面 (GUI)，使用 `egui` 和 `eframe`。该应用程序会索引文件，并允许用户根据文件内容进行搜索。

## 功能

- **命令行界面 (CLI)**：允许用户通过终端与应用程序交互。
- **图形用户界面 (GUI)**：提供用户友好的文件搜索界面。
- **自动索引**：在后台定期更新文件索引。
- **可定制搜索**：用户可以设置索引的根目录，并根据文件内容进行搜索。

## 安装

1. **克隆仓库**：
    ```sh
    git clone https://github.com/Yoimiya-Naganohara/search.git
    cd search
    ```

2. **构建应用程序**：
    ```sh
    cargo build --release
    ```

## 使用

### 命令行界面 (CLI)

在 CLI 模式下运行应用程序时，提供任何参数即可：
```sh
./target/release/search_terminal some_argument
```

### 图形用户界面 (GUI)

在 GUI 模式下运行应用程序时，不需要提供任何参数：
```sh
./target/release/search
```

## 配置

### 设置根目录

在 GUI 模式下，您可以通过点击“设置”按钮并输入所需的目录路径来设置索引的根目录。

### 自动索引

应用程序每 10 分钟自动更新一次文件索引。这是由后台线程处理的。

## 代码概述

### main.rs

[`main.rs`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2Fsrc%2Fmain.rs%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "d:\search\src\main.rs") 文件包含应用程序的入口点。

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

包含 [`Search`](command:_github.copilot.openSymbolFromReferences?%5B%22%22%2C%5B%7B%22uri%22%3A%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2Fsrc%2Fmain.rs%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%22pos%22%3A%7B%22line%22%3A30%2C%22character%22%3A21%7D%7D%5D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "Go to definition") 和 `SearchEngine` 结构体的实现，处理文件索引和搜索。

### ui_handle.rs

包含 [`SearchApp`](command:_github.copilot.openSymbolFromReferences?%5B%22%22%2C%5B%7B%22uri%22%3A%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2Fsrc%2Fmain.rs%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%22pos%22%3A%7B%22line%22%3A21%2C%22character%22%3A26%7D%7D%5D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "Go to definition") 结构体的实现，使用 `egui` 和 `eframe` 处理 GUI 逻辑。

## 许可证

此项目使用 MIT 许可证。详情请参见 [`LICENSE`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2Fd%3A%2Fsearch%2FLICENSE%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%2C%2266d6ebe7-0450-42af-977c-2ff64ac7f4b4%22%5D "d:\search\LICENSE") 文件。

## 贡献

欢迎贡献！请在 GitHub 上打开问题或提交拉取请求。

## 致谢

- [egui](https://github.com/emilk/egui) 提供的 GUI 框架。
- [eframe](https://github.com/emilk/eframe) 提供的应用程序框架。
