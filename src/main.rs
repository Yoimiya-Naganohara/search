/// The main entry point for the search application.
///
/// This application allows users to interactively search for files and directories
/// within a specified path. The user can input commands to change the search directory,
/// update the search index, display search results, and quit the application.
///
/// # Commands
/// - `#C`: Change the search directory. The user will be prompted to input a new directory path.
/// - `#Q`: Quit the application.
/// - `#U`: Update the search index. The application will regenerate the search index for the current directory.
/// - `#D`: Display the search results for the last search query.
/// - `#?`: Show the help message with the list of available commands.
///
/// # Usage
/// The user can input a search query directly to search within the current directory.
/// If the query ends with `#D`, the search results will be displayed.
///
/// The search results include the time taken to perform the search and the number of results found.
/// If the `#D` command is used, the results will be printed to the console.
///
/// The user can also open a directory from the search results by inputting the index of the result prefixed with `#`.
///
/// # Example
/// ```
/// #C
/// C:\Users\
/// #U
/// #D
/// myfile.txt
/// #0
/// ```
///
/// This example changes the search directory to `C:\Users\`, updates the search index, displays the results,
/// searches for `myfile.txt`, and opens the first result from the search results.
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
    ██║     ██║  ██║███████║   ██║       ███████║███████╗██║  ██║██║  ██║███████╗██║  ██║
    ╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝       ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
    \r\n type #? to get help"
    );

    loop {
        let mut d = false;
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        buf.pop();
        buf.pop();
        if buf == "#?" {
            println!("Commands:\n#C - Change directory\n#Q - Quit\n#U - Update index\n#D - Display results\n#? - Show this help message");
        }

        if buf == "#C" {
            path.clear();
            std::io::stdin().read_line(&mut path).unwrap();
            path.pop();
            path.pop();
        }
        if buf == "#Q" {
            return;
        } else if buf.ends_with("#D") {
            d = true;
            buf.pop();
            buf.pop();
        } else if buf == "#U" {
            println!("Generating");
            let collected_path = [&path].iter().collect();
            engine.generate(collected_path);
            engine.store();
            println!("Done");
            buf.clear();
        }
        if buf.len() > 0 {
            engine.read(buf.remove(0));
        }
        let time = time::SystemTime::now();
        let data = engine.find(&buf);
        let endtime = time::SystemTime::now();
        let duration = endtime.duration_since(time).expect("Time went backwards");
        println!(
            "Search Done!  time cost:  {:?}  results: {}",
            duration,
            match data {
                Ok(x) => {
                    x.len()
                }
                Err(_) => {
                    0
                }
            }
        );
        if d {
            if data.is_ok() {
                println!("{:?}", data.unwrap());
                println!("\rtype #0-{} to open", data.unwrap().len());
            } else {
                println!("None");
            }
            buf.clear();
            std::io::stdin().read_line(&mut buf).unwrap();
            buf.pop();
            buf.pop();
            if buf.contains('#') {
                // dbg!(&buf);
                buf.remove(0);
                let dir = data.unwrap().get(buf.parse::<usize>().unwrap());
                if let Some(dir) = dir {
                    if let Err(e) = open::that(dir) {
                        eprintln!("Failed to open directory: {}", e);
                    }
                }
            }
        }
    }
}
