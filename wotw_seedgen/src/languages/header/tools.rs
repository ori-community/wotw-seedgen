use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use ansi_term::{Style, Colour};
use rustc_hash::FxHashMap;

use crate::{util::{
    self,
    constants::{HEADER_INDENT, NAME_COLOUR, UBERSTATE_COLOUR}, UberState,
}, Header, Item, item::{UberStateOperator, Command}};

use super::{HeaderContent, VResolve};

fn is_hidden(header: &Path) -> Result<bool, String> {
    let file = fs::File::open(header).map_err(|err| format!("Failed to open header from {:?}: {}", header, err))?;
    let mut file = BufReader::new(file);

    let mut line = String::new();
    file.read_line(&mut line).map_err(|err| format!("Failed to read header from {:?}: {}", header, err))?;

    Ok(line.trim() == "#hide")
}

fn headers_in_directory(directory: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(fs::read_dir(directory).map_err(|err| format!("Failed to read directory {:?}: {}", directory, err))?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if let Some("wotwrh") = extension.to_str() {
                        return Some(path);
                    }
                }
            }
            None
        })
        .collect())
}

fn find_headers(show_hidden: bool) -> Result<Vec<PathBuf>, String> {
    let mut headers = headers_in_directory(Path::new("."))?;
    if let Ok(mut more) = headers_in_directory(Path::new("./headers")) {
        headers.append(&mut more);
    }

    if !show_hidden {
        headers = headers.iter()
            .map(|header| is_hidden(header).map(|hidden| if hidden { None } else { Some(header) }))
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .filter_map(|&header| header)
            .cloned()
            .collect::<Vec<_>>();
    }

    Ok(headers)
}

fn summarize_headers(headers: &[PathBuf]) -> Result<String, String> {
    let mut output = String::new();

    for header in headers {
        let mut name = header.file_stem().unwrap().to_string_lossy().into_owned();
        let header = fs::read_to_string(header).map_err(|err| format!("Error reading header from {:?}: {}", header, err))?;

        let mut description = None;

        for line in header.lines() {
            if let Some(desc) = line.trim_start().strip_prefix("///") {
                let desc = desc.trim();
                if desc.is_empty() {
                    continue;
                }
                let first = description.is_none();
                description = Some(desc);
                if !first {
                    break;
                }
            }
        }

        util::add_trailing_spaces(&mut name, HEADER_INDENT);

        output += &format!("{}  {}\n", NAME_COLOUR.paint(name), description.unwrap_or("no description"));
    }

    Ok(output)
}

pub fn list() -> Result<(), String> {
    let mut output = String::new();

    let headers = find_headers(false)?;

    if headers.is_empty() {
        println!("No headers found");
        return Ok(());
    }

    let headers_length = headers.len();
    output += &format!("{}", Style::new().fg(Colour::Green).bold().paint(format!("{} header{} found\n\n", headers_length, if headers_length == 1 { "" } else { "s" })));

    output += &summarize_headers(&headers)?;
    output.push('\n');

    output += "Use 'headers <name>...' for details about one or more headers";

    println!("{}", output);
    Ok(())
}

pub fn inspect(headers: Vec<PathBuf>) -> Result<(), String> {
    let mut output = String::new();

    let hint = if headers.len() == 1 {
        let name = headers[0].file_stem().unwrap().to_string_lossy();
        format!("Use 'preset <name> -h {} ...' to add this and other headers to a preset", NAME_COLOUR.paint(name))
    } else {
        let mut arguments = headers.iter().fold(String::new(), |acc, header|
            format!("{}{} ", acc, header.file_stem().unwrap().to_string_lossy())
        );
        arguments.pop();

        format!("Use 'preset <name> -h {} ...' to add these headers to a preset", NAME_COLOUR.paint(arguments))
    };

    for mut header in headers {
        header.set_extension("wotwrh");
        let name = header.file_stem().unwrap().to_string_lossy();

        let contents = util::read_file(&header, "headers")?;

        let mut description = NAME_COLOUR.paint(format!("{} header:\n", name)).to_string();

        for line in contents.lines() {
            if let Some(desc) = line.trim_start().strip_prefix("///") {
                description.push_str(desc.trim());
                description.push('\n');
            }
        }

        if description.is_empty() {
            output += &Style::new().italic().paint("No description provided\n");
        } else {
            output += &description;
            output.push('\n');
        }
    }

    output += &hint;
    println!("{}", output);
    Ok(())
}

pub fn validate(path: Option<PathBuf>) -> Result<bool, String> {
    let mut output = String::new();

    let headers = match path {
        Some(path) => vec![path],
        None => find_headers(true)?,
    };

    let mut occupation_map = Vec::new();

    let length = headers.len();
    output += &format!("{}", Style::new().italic().paint(format!("validating {} header{}\n", length, if length == 1 { "" } else { "s" })));

    let mut passed = Vec::new();
    let mut failed = Vec::new();

    for header in headers {
        let contents = util::read_file(&header, "headers")?;
        let mut name = header.file_stem().unwrap().to_string_lossy().into_owned();

        match validate_header(&contents) {
            Ok((occupied, excludes)) => {
                occupation_map.push((name, occupied, excludes));
            },
            Err(err) => {
                util::add_trailing_spaces(&mut name, HEADER_INDENT);
                failed.push(format!("{}  {}\n", NAME_COLOUR.paint(name), err));
            },
        }
    }

    for index in 0..occupation_map.len() {
        let (header, occupied, excludes) = &occupation_map[index];
        let mut collision_message = String::new();

        'outer: for uber_state in occupied {
            // special cases because this system is not holding up to modern header logic
            if uber_state.identifier.uber_group == 9 && (
                uber_state.identifier.uber_id == 0 && ["250", "251", "999"].contains(&&uber_state.value[..])
                || uber_state.identifier.uber_id == 999 && uber_state.value == "200"
                || uber_state.identifier.uber_id == 150
            ) {
                continue;
            }

            for (other_header, other_occupied, _) in &occupation_map {
                if header == other_header || excludes.contains(other_header) {
                    continue;
                }
                if let Some(collision) = other_occupied.iter().find(|&other| {
                    let generic = uber_state.value.is_empty() || other.value.is_empty();
                    uber_state == other || (generic && uber_state.identifier == other.identifier)
                }) {
                    collision_message = format!("Collision between used state {} and {} using {}",
                        UBERSTATE_COLOUR.paint(uber_state.code()),
                        NAME_COLOUR.paint(other_header),
                        UBERSTATE_COLOUR.paint(collision.code())
                    );
                    break 'outer;
                }
            }
        }

        if collision_message.is_empty() {
            let mut occupied_summary = String::new();
            let mut last_value = i32::MIN;
            let mut range = false;

            for uber_state in occupied {
                if let Ok(value) = uber_state.value.parse::<i32>() {
                    if value == last_value + 1 {
                        range = true;
                    } else if range {
                        for _ in 0..2 { occupied_summary.pop(); }
                        occupied_summary += &format!("{}, ", UBERSTATE_COLOUR.paint(format!("..{}", last_value)));
                        range = false;
                    }
                    last_value = value;
                    if range {
                        continue;
                    }
                } else if range {
                    for _ in 0..2 { occupied_summary.pop(); }
                    occupied_summary += &format!("{}, ", UBERSTATE_COLOUR.paint(format!("..{}", last_value)));
                    range = false;
                }

                occupied_summary += &format!("{}, ", UBERSTATE_COLOUR.paint(uber_state.code()));
            }

            for _ in 0..2 { occupied_summary.pop(); }

            if range {
                occupied_summary += &format!("{}", UBERSTATE_COLOUR.paint(format!("..{}", last_value)));
            }

            let mut name = header.clone();
            util::add_trailing_spaces(&mut name, HEADER_INDENT);

            if occupied_summary.is_empty() {
                passed.push(format!("{}  --\n", NAME_COLOUR.paint(name)));
            } else {
                passed.push(format!("{}  uses {}\n", NAME_COLOUR.paint(name), occupied_summary));
            }
        } else {
            let mut name = header.clone();
            util::add_trailing_spaces(&mut name, HEADER_INDENT);
            failed.push(format!("{}  {}\n", NAME_COLOUR.paint(name), collision_message));
        }
    }

    let failed_length = failed.len();
    let valid = failed_length == 0;
    if !valid {
        output += &format!("{}", Colour::Red.paint(format!("\n{}/{} failed\n", failed_length, length)));

        for failed in failed {
            output += &failed;
        }
    }
    let passed_length = passed.len();
    if passed_length > 0 {
        output += &format!("{}", Colour::Green.paint(format!("\n{}/{} passed\n", passed_length, length)));

        for passed in passed {
            output += &passed;
        }
    }

    let mut check_free = |description, range, condition: fn(&UberState, i32) -> bool |
    {
        let mut first = None;
        for index in range {
            let occupied = occupation_map.iter().any(|(_, states, _)| states.iter().any(|state| condition(state, index)));
            if occupied {
                if let Some(first_value) = first {
                    let last = index - 1;
                    output += &format!("Free {description}: {first_value}..{last}\n");
                    first = None;
                }
            } else if first.is_none() {
                first = Some(index);
            }
        }
        if let Some(first_value) = first {
            output += &format!("Free {description}: {first_value}..\n");
        }
    };

    check_free("9|0", 1..1000, |state: &UberState, index: i32|
        state.identifier.uber_group == 9
        && state.identifier.uber_id == 0
        && state.value == index.to_string()
    );
    check_free("integer", 1..100, |state: &UberState, index: i32|
        state.identifier.uber_group == 9
        && i32::from(state.identifier.uber_id) == index
    );
    check_free("boolean", 100..150, |state: &UberState, index: i32|
        state.identifier.uber_group == 9
        && i32::from(state.identifier.uber_id) == index
    );
    check_free("float", 150..175, |state: &UberState, index: i32|
        state.identifier.uber_group == 9
        && i32::from(state.identifier.uber_id) == index
    );

    println!("{}", output);
    Ok(valid)
}

pub fn validate_header(contents: &str) -> Result<(Vec<UberState>, Vec<String>), String> {
    let mut default_parameters = FxHashMap::default();

    let header = Header::parse(contents.to_string(), &mut rand::thread_rng())
        .map_err(|errors| errors.verbose_display())?;
    header.fill_parameters(&mut default_parameters)?;
    let build = header.clone().build(default_parameters.clone())?;

    let mut occupied_states = vec![];

    for content in header.contents {
        match content {
            HeaderContent::Timer(timer) => {
                occupied_states.push(UberState {
                    identifier: timer.timer,
                    value: "++".to_string(),  // represent a timer so that the sort will put it alongside + and - commands
                });
            },
            HeaderContent::Pickup(pickup) => {
                if pickup.skip_validation { continue }

                let pickup = pickup.resolve(&default_parameters)?;
                if pickup.trigger.identifier.uber_group == 9 {
                    occupied_states.push(pickup.trigger.clone());
                }

                match pickup.item {
                    Item::UberState(uber_state_item) if uber_state_item.uber_identifier.uber_group == 9 => {
                        if let UberStateOperator::Value(mut value) = uber_state_item.operator {
                            if value == "false" || value == "0" { continue }
                            if value == "true" { value = String::from("1"); }

                            occupied_states.push(UberState {
                                identifier: uber_state_item.uber_identifier,
                                value,
                            });
                        }
                    },
                    Item::Command(Command::StopEqual { uber_state } |
                    Command::StopGreater { uber_state } |
                    Command::StopLess { uber_state }) => {
                        if pickup.trigger.identifier.uber_group == 9 {
                            if uber_state.identifier.uber_group == 9 {
                                occupied_states.push(uber_state);
                            }
                        } else {
                            return Err(format!("stop command on {} stops a multipickup outside of uber group 9. This may interact unpredictably with other headers.", pickup.trigger.code()));
                        }
                    }
                    _ => {},
                }
            },
            _ => {},
        }
    }

    occupied_states.sort_unstable();
    occupied_states.dedup();

    // remove redundancies, the sort beforehand put all timers, + and - commands in front
    let mut index = 0;
    while let Some(current) = occupied_states.get_mut(index) {
        if current.value.starts_with(&['+', '-'][..]) || current.value.is_empty() {
            current.value = String::new();
            let clone = current.clone();

            occupied_states.retain(|other| other == &clone || other.identifier != clone.identifier);
        }
        index += 1;
    }

    occupied_states.dedup();

    Ok((occupied_states, build.excludes))
}

#[cfg(test)]
mod tests {
    #[test]
    fn validate() {
        assert!(super::validate(None).unwrap());
    }
}
