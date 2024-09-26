mod data;
mod generate;

use generate::{Search, SearchEngine};
use open;
use std::time;

fn main() {
    let mut engine = Search::new();
    let mut path = String::from("C:\\");
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
        let mut display_results = false;
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().to_string();

        match buf.as_str() {
            "#?" => println!("Commands:\n#C - Change directory\n#Q - Quit\n#U - Update index\n#D - Display results\n#? - Show this help message"),
            "#C" => {
                path.clear();
                println!("Enter new directory path: (Exit#)");
                std::io::stdin().read_line(&mut path).unwrap();
                path = path.trim().to_string();
                if path.contains('#') {
                    path = "C:\\".to_string();
                }
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
            _ => {}
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
            "Search Done!  time cost:  {:?}  results: {}",
            duration,
            data.as_ref().map_or(0, |x| x.len())
        );

        if display_results {
            if let Ok(results) = data {
                for (i, result) in results.iter().enumerate() {
                    println!("{} [{}]", i, result.to_str().unwrap());
                }
                println!("\r\ntype #0-{} to open \n     #X to cancel", results.len());

                loop {
                    buf.clear();
                    std::io::stdin().read_line(&mut buf).unwrap();
                    buf = buf.trim().to_string();

                    if buf.starts_with("#L") {
                        let index = buf.trim_start_matches("#L").parse::<usize>().unwrap_or(0);
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
                    } else if buf.starts_with('#') && !buf.contains("#X") {
                        let index = buf.trim_start_matches('#').parse::<usize>().unwrap_or(0);
                        if let Some(dir) = results.get(index) {
                            if let Err(e) = open::that(dir) {
                                eprintln!("Failed to open directory: {}", e);
                            }
                        }
                    } else {
                        println!("Exit");
                        break;
                    }
                }
            } else {
                println!("None");
            }
        }
    }
}
