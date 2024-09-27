mod data;
mod search_engine;

use colored::Colorize;
use open;
use search_engine::{Search, SearchEngine};
use std::{io::Write, time};

fn main() {
    let mut engine = Search::new();
    #[cfg(target_os = "windows")]
    let mut path = String::from("C:\\");

    #[cfg(target_os = "linux")]
    let mut path = String::from("/");
    println!(
        "{}",
        "
    ███████╗ █████╗ ███████╗████████╗    ███████╗███████╗ █████╗ ██████╗ ███████╗██╗  ██╗
    ██╔════╝██╔══██╗██╔════╝╚══██╔══╝    ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║
    ███████╗███████║███████╗   ██║       ███████╗█████╗  ███████║██████╔╝██║     ███████║
    ██╔════╝██╔══██║╚════██║   ██║       ╚════██║██╔══╝  ██╔══██║██╔═══╝ ██║     ██╔══██║
    ██║     ██║  ██║███████║   ██║       ███████║███████╗██║  ██║██║  ██╗███████╗██║  ██║
    ╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝       ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
    \r\n Type ':?' for help"
            .blue()
            .bold()
    );

    loop {
        print!("{}", "Search ".green().bold());
        std::io::stdout().flush().unwrap();
        let mut display_results = false;
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().to_string();

        match buf.as_str() {
            ":?" => {
                println!("{}", "Usage:\nFile name - Search for a file (Add '?' to the end of the file name to use a wildcard pattern)\n.extension name - Match file extension\n\nCommands:\n:C - Change directory\n:Q - Quit the application\n:U - Update the index\n:D - Display search results\n:? - Show this help message".yellow());
                continue;
            }
            ":C" => {
                let pre = path.clone();
                path.clear();
                println!(
                    "{}",
                    format!(
                        "Please enter the new directory path (current: {}). Type ':' to cancel:",
                        pre
                    )
                    .yellow()
                );
                std::io::stdin().read_line(&mut path).unwrap();
                path = path.trim().to_string();
                if path.contains(':') {
                    path = pre;
                }
                engine.set_part(path.chars().next().unwrap());
                continue;
            }
            ":Q" => return,
            _ if buf.ends_with(":D") => {
                display_results = true;
                buf = buf.trim_end_matches(":D").to_string();
            }
            ":U" => {
                println!(
                    "{}",
                    "Generating index for the current directory...".yellow()
                );
                let start_time = time::SystemTime::now();
                engine.generate_index([&path].iter().collect());
                let duration = start_time.elapsed().expect("Time went backwards");
                println!(
                    "{}",
                    format!(
                        "Index generation complete. Time taken: {:?}. Number of indexed items: {}",
                        duration,
                        engine.indexed()
                    )
                    .green()
                );
                engine.save_index();
                continue;
            }
            _ => {}
        }

        if buf.contains(':') {
            println!(
                "{}",
                "Undefined action. Please enter a valid command.".red()
            );
            continue;
        }

        if !buf.is_empty() {
            engine.load_index(buf.remove(0));
        }

        let start_time = time::SystemTime::now();
        let data = engine.search(&buf);
        let duration = start_time.elapsed().expect("Time went backwards");

        println!(
            "{}",
            format!(
                "Search completed. Time taken: {:?}. Number of results: {}\n",
                duration,
                data.as_ref().map_or(0, |x| x.len())
            )
            .green()
        );

        if display_results {
            if let Ok(results) = data {
                for (i, result) in results.iter().enumerate() {
                    println!("{}", format!("{} [{}]", i, result.to_str().unwrap()).cyan());
                }
                println!("{}", format!("\r\nType a number between 0 and {} to open the corresponding result (and l to locate), or 'x' to cancel.", results.len() - 1).yellow());

                loop {
                    print!("{}", "Open ".green().bold());
                    std::io::stdout().flush().unwrap();
                    buf.clear();
                    std::io::stdin().read_line(&mut buf).unwrap();
                    buf = buf.trim().to_string();
                    match buf.as_str() {
                        buf if buf.contains("l") => {
                            let index = buf.trim_matches('L').parse::<usize>();
                            if let Ok(index) = index {
                                if let Some(dir) = results.get(index) {
                                    let dir_str = dir.to_str().unwrap_or_default();
                                    let parent_dir = dir_str.replace(
                                        dir.file_name()
                                            .unwrap_or_default()
                                            .to_str()
                                            .unwrap_or_default(),
                                        "",
                                    );
                                    if let Err(e) = open::that(parent_dir) {
                                        eprintln!(
                                            "{}",
                                            format!("Failed to open directory: {}", e).red()
                                        );
                                    }
                                }
                            } else {
                                println!("{}", "Invalid input. Please enter a valid number.".red());
                            }
                        }
                        "x" => {
                            println!("{}", "Exit".yellow());
                            break;
                        }
                        _ => {
                            if let Ok(index) = buf.parse::<usize>() {
                                if let Some(dir) = results.get(index) {
                                    if let Err(e) = open::that(dir) {
                                        eprintln!(
                                            "{}",
                                            format!("Failed to open directory: {}", e).red()
                                        );
                                    }
                                }
                            } else {
                                println!("{}", "Invalid input. Please enter a valid number.".red());
                            }
                        }
                    }
                }
            } else {
                println!("{}", "None".red());
            }
        }
    }
}
