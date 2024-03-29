use super::cli;
use super::log_init;

use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use ansi_term::{Colour, Style};
use log::LevelFilter;
use rustc_hash::FxHashMap;

use wotw_seedgen::files::{self, FileAccess, FILE_SYSTEM_ACCESS};
use wotw_seedgen::header::{self, Header};
use wotw_seedgen::util::constants::NAME_COLOUR;

pub fn headers(headers: Vec<String>, subcommand: Option<cli::HeaderCommand>) -> Result<(), String> {
    log_init::initialize_log(None, LevelFilter::Info, false)
        .unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

    match subcommand {
        Some(cli::HeaderCommand::Validate { path }) => validate(path).map(|_| ()),
        Some(cli::HeaderCommand::Parse { path }) => compile_seed(path),
        None => {
            if headers.is_empty() {
                list()
            } else {
                inspect(headers)
            }
        }
    }
}

fn compile_seed(mut path: PathBuf) -> Result<(), String> {
    if path.extension().is_none() {
        path.set_extension("wotwrh");
    }

    let identifier = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let header = fs::read_to_string(path.clone())
        .map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;

    let mut rng = rand::thread_rng();

    let header = Header::parse(header, &mut rng)
        .map_err(|errors| {
            (*errors)
                .iter()
                .map(|err| err.verbose_display())
                .collect::<Vec<_>>()
                .join("\n")
        })?
        .build(FxHashMap::default())?;

    path.set_extension("wotwr");
    files::write_file(&identifier, "wotwr", &header.seed_content, "target")?;
    log::info!("Compiled {}", identifier);

    Ok(())
}

const HEADER_INDENT: usize = 24; // Which column to align header descriptions on

pub fn list() -> Result<(), String> {
    let mut output = String::new();

    let headers = read_all()?;

    if headers.is_empty() {
        println!("No headers found");
        return Ok(());
    }

    let headers_length = headers.len();
    output += &format!(
        "{}",
        Style::new().fg(Colour::Green).bold().paint(format!(
            "{} header{} found\n\n",
            headers_length,
            if headers_length == 1 { "" } else { "s" }
        ))
    );

    for (mut identifier, content) in headers {
        let description = Header::parse_documentation(&content).description;

        add_trailing_spaces(&mut identifier, HEADER_INDENT);

        output += &format!(
            "{}  {}\n",
            NAME_COLOUR.paint(identifier),
            description.unwrap_or_else(|| "no description".to_string())
        );
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
        format!(
            "Use 'world-preset <name> -h {} ...' to add this and other headers to a preset",
            NAME_COLOUR.paint(name)
        )
    } else {
        let mut arguments = headers
            .iter()
            .fold(String::new(), |acc, header| format!("{}{} ", acc, header));
        arguments.pop();

        format!(
            "Use 'world-preset <name> -h {} ...' to add these headers to a preset",
            NAME_COLOUR.paint(arguments)
        )
    };

    for header in headers {
        let contents = FILE_SYSTEM_ACCESS.read_header(&header)?;
        let documentation = Header::parse_documentation(&contents);

        writeln!(output, "{} header:", NAME_COLOUR.paint(header)).unwrap();

        if let Some(name) = documentation.name {
            write!(output, "{name}\n\n").unwrap()
        }
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

    header::validate_headers(headers);
    Ok(())
}

fn read(path: impl AsRef<Path>) -> Result<String, String> {
    fs::read_to_string(path.as_ref()).map_err(|err| err.to_string())
}
fn identifier(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string()
}
fn read_all() -> Result<Vec<(String, String)>, String> {
    files::find_headers().map(|headers| {
        headers
            .into_iter()
            .map(|header| read(&header).map(|content| (identifier(&header), content)))
            .filter_map(|header| header.ok())
            .collect()
    })
}

fn add_trailing_spaces(string: &mut String, target_length: usize) {
    let mut length = string.len();
    while target_length > length {
        string.push(' ');
        length += 1;
    }
}
