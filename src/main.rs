use std::{env, process};

use mdka::from_html;

/// app entry point on executable
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <html content>", args[0]);
        process::exit(1);
    }

    let html = &args[1];

    println!("{}", from_html(html));
}
