use std::{
    io::{stdin, Write},
    path::PathBuf,
    process::exit,
    time,
};

use colored::Colorize;
use regex::Regex;

use crate::search_engine::{Search, SearchEngine};

pub struct Handle {
    welcome: String,
    command: String,
    engine: Search,
}
pub(crate) trait Handler {
    fn new() -> Self;

    fn welcome(&mut self);
    fn input(&mut self);

    fn handler(&mut self);

    fn browse(&self, data: &[PathBuf], found: &[usize]) -> bool;
}
impl Handler for Handle {
    fn new() -> Self {
        Handle {
            welcome: "
        ███████╗ █████╗ ███████╗████████╗    ███████╗███████╗ █████╗ ██████╗ ███████╗██╗  ██╗
        ██╔════╝██╔══██╗██╔════╝╚══██╔══╝    ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║
        ███████╗███████║███████╗   ██║       ███████╗█████╗  ███████║██████╔╝██║     ███████║
        ██╔════╝██╔══██║╚════██║   ██║       ╚════██║██╔══╝  ██╔══██║██╔═══╝ ██║     ██╔══██║
        ██║     ██║  ██║███████║   ██║       ███████║███████╗██║  ██║██║  ██╗███████╗██║  ██║
        ╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝       ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝"
                .to_string(),
            command: String::new(),
            engine: Search::new(),
        }
    }

    fn input(&mut self) {
        print!("{}", ">".green());
        std::io::stdout().flush().unwrap();
        self.command.clear();
        std::io::stdin().read_line(&mut self.command).unwrap();
        self.command = self.command.trim().to_string();
    }

    fn handler(&mut self) {
        match self.command.as_str() {
            ":?" => {
                println!("{}", "Usage:
.     - Matches any character except a newline.
^     - Matches the start of the string.
$     - Matches the end of the string.
*     - Matches 0 or more repetitions of the preceding pattern.
+     - Matches 1 or more repetitions of the preceding pattern.
?     - Matches 0 or 1 repetition of the preceding pattern.
{m,n} - Matches from m to n repetitions of the preceding pattern.
[]    - Matches any single character in the brackets.
|     - Matches either the pattern before or the pattern after the |.
()    - Groups patterns.
\nCommands:\n:C - Change directory\n:Q - Quit the application\n:U - Update the index (Add '*' to update given section)\n:D - Display search results\n:? - Show this help message".yellow());
            }
            ":C" => {
                let mut path = String::new();
                println!(
                    "{}",
                    format!(
                        "Please enter the new directory path (current: {}). Type ':x' to cancel:",
                        self.engine.get_root_dir().to_str().unwrap()
                    )
                    .yellow()
                );
                std::io::stdin().read_line(&mut path).unwrap();
                path = path.trim().to_string();
                if path == ":x" {
                    return;
                }
                self.engine.set_root_dir(PathBuf::from(path));
                self.engine.load_index();
            }
            ":Q" => exit(0),
            ":U" => {
                println!(
                    "{}",
                    "Generating index for the current directory...".yellow()
                );

                let start_time = time::SystemTime::now();
                self.engine.generate_index();
                let duration = start_time.elapsed().expect("Time went backwards");
                println!(
                    "{}",
                    format!(
                        "Index generation complete. Time taken: {:?}. Number of indexed items: {}",
                        duration,
                        self.engine.get_index().len()
                    )
                    .green()
                );
                self.engine.save_index();
            }
            _ => {
                let data = self.engine.get_index();
                if data.is_empty() {
                    return;
                }

                let mut counter = 0usize;
                let mut i = 0usize;
                let mut found = Vec::new();
                let regex = Regex::new(&self.command).unwrap_or(Regex::new("None").unwrap());
                for file in data {
                    i += 1;
                    let file_name = file.file_name().unwrap().to_str().unwrap();
                    if i == data.len() || counter == 20 {
                        counter = 0;
                        if self.browse(data, &found) {
                            return;
                        };
                    }
                    if !regex.is_match(file_name) {
                        continue;
                    }
                    let re = regex.find(file_name).unwrap();
                    let highlighted =
                        file_name.replace(re.as_str(), &format!("{}", re.as_str().green().bold()));
                    println!("{} [{}]", counter, highlighted);
                    found.insert(counter, i - 1);
                    counter += 1;
                }
            }
        }
    }

    fn welcome(&mut self) {
        self.engine.load_index();
        println!("{}", self.welcome.blue().bold())
    }

    fn browse(&self, data: &[PathBuf], found: &[usize]) -> bool {
        println!(
            "{}",
            "Tip: Enter 'q' to quit, 's' to get path, 'l<number>' to open the parent directory of the result, or just the number to open the result.".yellow()
        );
        loop {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            buf = buf.trim().to_string();
            match buf.as_str() {
                "" => return true,
                _ if buf.contains('s') => {
                    buf = buf.trim_matches('s').to_string();
                    if let Ok(index) = buf.parse::<usize>() {
                        if let Some(dir) = data.get(found[index]) {
                            println!("[{}]", dir.to_str().unwrap());
                        }
                    } else {
                        println!("{}", "Invalid input. Please enter a valid number.".red());
                    }
                }
                "q" => return true,
                _ => {
                    let (buf, p) = if buf.contains('l') {
                        (buf.trim_matches('l').to_string(), true)
                    } else {
                        (buf, false)
                    };

                    if let Ok(index) = buf.parse::<usize>() {
                        if let Some(mut dir) = data.get(found[index]) {
                            let path_buf = dir.parent().unwrap().to_path_buf();
                            if p {
                                dir = &path_buf;
                            }
                            if let Err(e) = open::that(dir) {
                                eprintln!("{}", format!("Failed to open directory: {}", e).red());
                            }
                        }
                    } else {
                        println!("{}", "Invalid input. Please enter a valid number.".red());
                    }
                }
            }
        }
    }
}
