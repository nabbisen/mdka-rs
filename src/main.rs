use std::env;

use mdka::{from_file, from_file_to_file, from_html, from_html_to_file};

struct ExecutableParam {
    html_text: Option<String>,
    html_filepath: Option<String>,
    markdown_filepath: Option<String>,
    overwrites: bool,
}

/// app entry point on executable
fn main() {
    let validated = validate_or_show_help();
    match validated {
        Ok(params_or_none) => match params_or_none {
            Some(params) => convert_html_to_markdown(params),
            None => (),
        },
        Err(_) => (),
    }
}

fn convert_html_to_markdown(params: ExecutableParam) {
    match params.markdown_filepath {
        Some(markdown_filepath) => {
            let _ = match params.html_filepath {
                Some(html_filepath) => from_file_to_file(
                    html_filepath.as_str(),
                    markdown_filepath.as_str(),
                    params.overwrites,
                ),
                None => from_html_to_file(
                    params.html_text.unwrap().as_str(),
                    markdown_filepath.as_str(),
                    params.overwrites,
                ),
            }
            .unwrap_or_else(|e| panic!("Failed ({})", e));
            println!("Done. output = {}", markdown_filepath);
        }
        None => {
            let md = match params.html_filepath {
                Some(html_filepath) => from_file(html_filepath.as_str()),
                None => Ok(from_html(params.html_text.unwrap().as_str())),
            }
            .unwrap_or_else(|e| panic!("Failed ({})", e));
            println!("{}", md);
        }
    }
}

/// validate parameters. show help and exit if either `-h` or `--help` is passed
fn validate_or_show_help() -> Result<Option<ExecutableParam>, String> {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help(&args[0]);
        return Ok(None);
    }
    let params = params(&args).unwrap_or_else(|err| {
        panic!(
            "Possibly invalid parameters: {}. Error = {}",
            args.iter()
                .skip(1)
                .cloned()
                .collect::<Vec<String>>()
                .join(","),
            err
        )
    });
    return Ok(Some(params));
}

/// get parameters from arguments
fn params(args: &Vec<String>) -> Result<ExecutableParam, String> {
    let mut html_text: Option<String> = None;
    let mut html_filepath: Option<String> = None;
    let mut markdown_filepath: Option<String> = None;
    let mut overwrites: bool = false;

    // start after the program name
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-i" => {
                if i + 1 < args.len() {
                    if html_text.is_some() {
                        panic!("'-i' HTML file path cannot be used with HTML text");
                    }
                    html_filepath = Some(args[i + 1].clone());
                    // skip the value for it
                    i += 2;
                } else {
                    panic!("Missing value for '-i' option");
                }
            }
            "-o" => {
                if i + 1 < args.len() {
                    markdown_filepath = Some(args[i + 1].clone());
                    // skip the value for it
                    i += 2;
                } else {
                    panic!("Missing value for '-o' option");
                }
            }
            "--overwrites" => overwrites = true,
            _ => {
                if html_filepath.is_some() {
                    panic!("HTML text cannot be used with '-i' HTML file path");
                }
                html_text = Some(args[i].clone());
                i += 1;
            }
        }
    }

    if html_text.as_ref().is_none() && html_filepath.as_ref().is_none() {
        panic!("Either HTML text or '-i' HTML file path must be specified.");
    }

    Ok(ExecutableParam {
        html_text,
        html_filepath,
        markdown_filepath,
        overwrites,
    })
}

/// show help
fn print_help(app_name: &str) {
    println!("Usage:");
    println!("  -h, --help             : Help is shown.");
    println!("  <html_text>            : Direct parameter is taken as HTML text to be converted. Either this or <html_filepath> is required.");
    println!("  -i <html_filepath>     : Read HTML text from it. Optional.");
    println!("  -o <markdown_filepath> : Write Markdown result to it. Optional.");
    println!("  --overwrites           : Overwrite if Markdown file exists. Optional.");
    println!("\nExamples:");
    println!("  {} \"<p>Hello, world.</p>\"", app_name);
    println!("  {} -i input.html", app_name);
    println!("  {} -o output.md \"<p>Hello, world.</p>\"", app_name);
    println!("  {} -i input.html -o output.md --overwrites", app_name);
}
