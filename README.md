
# Search Engine CLI

This is a command-line search engine application written in Rust. It allows you to search for files and directories, update the search index, and open or locate search results.

## Features

- **Search**: Search for files and directories.
- **Update Index**: Generate and save an index for the current directory.
- **Change Directory**: Change the directory to be indexed and searched.
- **Open Results**: Open or locate search results directly from the command line.

## Commands

- `#?` - Show help message
- `#C` - Change directory
- `#Q` - Quit the application
- `#U` - Update the search index for the current directory
- `#D` - Display search results

## Usage

1. **Clone the repository**:
   ```sh
   git clone <repository-url>
   cd <repository-directory>
   ```

2. **Build the project**:
   ```sh
   cargo build
   ```

3. **Run the application**:
   ```sh
   cargo run
   ```

4. **Use the commands**:
   - Type `#?` to see the list of available commands.
   - Type `#C` to change the directory.
   - Type `#Q` to quit the application.
   - Type `#U` to update the search index.
   - Type `#D` to display search results.

## Example

```sh
$ cargo run
    ███████╗ █████╗ ███████╗████████╗    ███████╗███████╗ █████╗ ██████╗ ███████╗██╗  ██╗
    ██╔════╝██╔══██╗██╔════╝╚══██╔══╝    ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║
    ███████╗███████║███████╗   ██║       ███████╗█████╗  ███████║██████╔╝██║     ███████║
    ██╔════╝██╔══██║╚════██║   ██║       ╚════██║██╔══╝  ██╔══██║██╔═══╝ ██║     ██╔══██║
    ██║     ██║  ██║███████║   ██║       ███████║███████╗██║  ██║██║  ██╗███████╗██║  ██║
    ╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝       ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
    
    type #? to get help

Search: #U
Generating index for the current directory...
Index generation complete.

Search: myfile.txt#D
Search completed. Time taken: 0.123s. Number of results: 3

0 [C:\Users\example\myfile.txt]
1 [C:\Users\example\Documents\myfile.txt]
2 [C:\Users\example\Downloads\myfile.txt]

Type a number between 0 and 2 to open the corresponding result (and L to locate), or 'X' to cancel.
Open: 1
```

## Dependencies

- [open](https://crates.io/crates/open)
- [serde](https://crates.io/crates/serde)
- [bincode](https://crates.io/crates/bincode)

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.

## Acknowledgements

- ASCII art generated using [patorjk.com](http://patorjk.com/software/taag/)
