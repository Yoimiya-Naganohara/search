use handle::{Handle, Handler};

mod handle;
mod search_engine;
fn main() {
    let mut handler = Handle::new();
    handler.welcome();
    loop {
        handler.input();
        handler.handler();
    }
}
