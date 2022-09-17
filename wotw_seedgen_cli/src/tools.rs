use std::fs;
use std::path::{Path, PathBuf};
use std::fmt::Write;

use ansi_term::{Style, Colour};

use wotw_seedgen::Header;
use wotw_seedgen::files::{find_headers, FILE_SYSTEM_ACCESS, FileAccess};
use wotw_seedgen::header::validate_headers;
use wotw_seedgen::util::constants::NAME_COLOUR;

const HEADER_INDENT: usize = 24;  // Which column to align header descriptions on

pub fn list() -> Result<(), String> {
    let mut output = String::new();

    let headers = read_all()?;

    if headers.is_empty() {
        println!("No headers found");
        return Ok(());
    }

    let headers_length = headers.len();
    output += &format!("{}", Style::new().fg(Colour::Green).bold().paint(format!("{} header{} found\n\n", headers_length, if headers_length == 1 { "" } else { "s" })));

    for (mut identifier, content) in headers {
        let description = Header::parse_documentation(&content).description;

        add_trailing_spaces(&mut identifier, HEADER_INDENT);

        output += &format!("{}  {}\n", NAME_COLOUR.paint(identifier), description.unwrap_or_else(|| "no description".to_string()));
    }

    output.push('\n');

    output += "Use 'headers <name>...' for details about one or more headers";

    println!("{}", output);
    Ok(())
}

pub fn inspect(headers: Vec<String>) -> Result<(), String> {
    let mut output = String::new();

    let hint = if headers.len() == 1 {
        let name = &headers[0];
        format!("Use 'world-preset <name> -h {} ...' to add this and other headers to a preset", NAME_COLOUR.paint(name))
    } else {
        let mut arguments = headers.iter().fold(String::new(), |acc, header|
            format!("{}{} ", acc, header)
        );
        arguments.pop();

        format!("Use 'world-preset <name> -h {} ...' to add these headers to a preset", NAME_COLOUR.paint(arguments))
    };

    for header in headers {
        let contents = FILE_SYSTEM_ACCESS.read_header(&header)?;
        let documentation = Header::parse_documentation(&contents);

        writeln!(output, "{} header:", NAME_COLOUR.paint(header)).unwrap();

        if let Some(name) = documentation.name { write!(output, "{name}\n\n").unwrap() }
        match documentation.description {
            Some(description) => write!(output, "{description}\n\n").unwrap(),
            None => output.push_str("no description provided\n\n"),
        }
    }

    output.push_str(&hint);
    println!("{}", output);
    Ok(())
}

pub fn validate(path: Option<PathBuf>) -> Result<(), String> {
    let headers = match path {
        Some(path) => vec![(identifier(&path), read(&path)?)],
        None => read_all()?,
    };

    validate_headers(headers);
    Ok(())
}

fn read(path: impl AsRef<Path>) -> Result<String, String> {
    fs::read_to_string(path.as_ref()).map_err(|err| err.to_string())
}
fn identifier(path: impl AsRef<Path>) -> String {
    path.as_ref().file_stem().unwrap().to_string_lossy().to_string()
}
fn read_all() -> Result<Vec<(String, String)>, String> {
    find_headers().map(|headers| headers.into_iter()
        .map(|header| read(&header).map(|content| (identifier(&header), content)))
        .filter_map(|header| header.ok())
        .collect())
}

fn add_trailing_spaces(string: &mut String, target_length: usize) {
    let mut length = string.len();
    while target_length > length {
        string.push(' ');
        length += 1;
    }
}
