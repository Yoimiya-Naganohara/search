mod data;
mod search_engine;

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
        "
    ███████╗ █████╗ ███████╗████████╗    ███████╗███████╗ █████╗ ██████╗ ███████╗██╗  ██╗
    ██╔════╝██╔══██╗██╔════╝╚══██╔══╝    ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║
    ███████╗███████║███████╗   ██║       ███████╗█████╗  ███████║██████╔╝██║     ███████║
    ██╔════╝██╔══██║╚════██║   ██║       ╚════██║██╔══╝  ██╔══██║██╔═══╝ ██║     ██╔══██║
    ██║     ██║  ██║███████║   ██║       ███████║███████╗██║  ██║██║  ██╗███████╗██║  ██║
    ╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝       ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
    \r\n type #? to get help"
    );

    loop {
        print!("Search:");
        std::io::stdout().flush().unwrap();
        let mut display_results = false;
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().to_string();

        match buf.as_str() {
            "#?" => println!("Commands:\n#C - Change directory\n#Q - Quit\n#U - Update index\n#D - Display results\n#? - Show this help message"),
            "#C" => {
                let pre=path.clone();
                path.clear();
                println!("Enter new directory path: (Exit#)");
                std::io::stdin().read_line(&mut path).unwrap();
                path = path.trim().to_string();
                if path.contains('#') {
                    path=pre;
                }
                engine.set_part(path.chars().next().unwrap());
            }
            "#Q" => return,
            _ if buf.ends_with("#D") => {
                display_results = true;
                buf = buf.trim_end_matches("#D").to_string();
            }
            "#U" => {
                println!("Generating index for the current directory...");
                engine.generate_index([&path].iter().collect());
                engine.save_index();
                println!("Index generation complete.");
            }
            _ => {println!("Undefined action");}
        }

        if buf.contains('#') {
            continue;
        }

        if !buf.is_empty() {
            engine.load_index(buf.remove(0));
        }

        let start_time = time::SystemTime::now();
        let data = engine.search(&buf);
        let duration = start_time.elapsed().expect("Time went backwards");

        println!(
            "Search completed. Time taken: {:?}. Number of results: {}\n",
            duration,
            data.as_ref().map_or(0, |x| x.len())
        );

        if display_results {
            if let Ok(results) = data {
                for (i, result) in results.iter().enumerate() {
                    println!("{} [{}]", i, result.to_str().unwrap());
                }
                println!("\r\nType a number between 0 and {} to open the corresponding result (and L to locate), or 'X' to cancel.", results.len() - 1);

                loop {
                    print!("Open:");
                    std::io::stdout().flush().unwrap();
                    buf.clear();
                    std::io::stdin().read_line(&mut buf).unwrap();
                    buf = buf.trim().to_string();
                    match buf.as_str() {
                        buf if buf.contains("L") => {
                            let index = buf.trim_matches('L').parse::<usize>().unwrap_or(0);
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
                                    eprintln!("Failed to open directory: {}", e);
                                }
                            }
                        }
                        "X" => {
                            println!("Exit");
                            break;
                        }
                        _ => {
                            let index = buf.parse::<usize>().unwrap_or(0);
                            if let Some(dir) = results.get(index) {
                                if let Err(e) = open::that(dir) {
                                    eprintln!("Failed to open directory: {}", e);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("None");
            }
        }
    }
}
