mod preprocess;
mod postprocess;
mod parse_item;
mod header_command;

pub(super) use preprocess::preprocess;
pub use postprocess::postprocess;

use std::str::FromStr;

use rustc_hash::FxHashMap;

use crate::{
    VItem,
    util::{VUberState, extensions::StrExtension, UberIdentifier},
    seed::parser::{parse_uber_identifier, parse_uber_state},
};

use super::{HeaderCommand, Annotation, HeaderDocumentation, HeaderContent, TimerDefinition, VPickup, V, VString};

fn trim_comment(input: &str) -> &str {
    input.find("//").map_or(input, |index| &input[..index]).trim_end()
}

fn parse_v_uber_state_trigger<'a, I>(parts: &mut I) -> Result<VUberState, String>
where
    I: Iterator<Item = &'a str>,
{
    let uber_group = parts.next().ok_or_else(|| "missing uber group".to_string())?;
    let uber_group = uber_group.parse().map_err(|_| format!("invalid uber group {uber_group}"))?;

    let uber_id = parts.next().ok_or_else(|| "missing uber id".to_string())?;
    let mut id_parts = uber_id.splitn(2, '=');
    let uber_id = id_parts.next().unwrap().parse().map_err(|_| format!("invalid uber id {uber_id}"))?;

    let value = id_parts.next().unwrap_or("");

    Ok(VUberState {
        identifier: UberIdentifier {
            uber_group,
            uber_id,
        },
        value: V::wrap(value),
    })
}

/// Returns the annotations of a header and removes them from the input string
pub(super) fn parse_annotations(input: &mut String) -> Result<Vec<Annotation>, String> {
    let mut annotations = vec![];
    let mut after_annotations = input.len();

    for range in input.line_ranges() {
        let range_start = range.start;
        let line = &input[range];
        if let Some(annotation) = line.strip_prefix('#') {
            let annotation = trim_comment(annotation);
            let annotation = Annotation::from_str(annotation)?;
            annotations.push(annotation);
        } else if !line.is_empty() {
            after_annotations = range_start;
            break;
        }
    }

    input.replace_range(0..after_annotations, "");

    Ok(annotations)
}

/// Returns the documentation of a header
pub(super) fn parse_documentation(input: &str) -> HeaderDocumentation {
    let mut name = None;
    let mut description: Option<String> = None;

    for line in input.lines() {
        if let Some(documentation) = line.strip_prefix("///") {
            if documentation.starts_with('/') { break }
            let documentation = documentation.trim();
            if documentation.is_empty() { continue }
            if name.is_none() {
                name = Some(documentation.to_string());
            } else if let Some(prior) = &mut description {
                prior.push('\n');
                prior.push_str(documentation);
            } else {
                description = Some(documentation.to_string());
            }
        } else { break }
    }

    HeaderDocumentation { name, description }
}

pub(super) struct HeaderContents {
    pub(super) parameter_documentation: FxHashMap<String, String>,
    pub(super) contents: Vec<HeaderContent>,
}
pub(super) fn parse_contents(input: String) -> Result<HeaderContents, String> {
    let mut parameter_documentation = FxHashMap::default();
    let mut contents = vec![];
    let mut last_documentation: Option<&str> = None;
    let mut skip_validation = false;

    for raw_line in input.lines() {
        let line = trim_comment(raw_line);
        if !line.is_empty() {
            let content = parse_line(line, skip_validation)?;

            if let HeaderContent::Command(HeaderCommand::Parameter { identifier, .. }) = &content {
                if let Some(documentation) = last_documentation {
                    parameter_documentation.insert(identifier.to_owned(), documentation.to_string());
                }
            }

            contents.push(content);
        }

        last_documentation = parse_parameter_documentation(raw_line);
        skip_validation = raw_line.trim_start().strip_prefix("//").map_or(false, |comment|
            comment.trim_start().starts_with("skip-validate"));
    }

    Ok(HeaderContents { parameter_documentation, contents })
}

fn parse_parameter_documentation(line: &str) -> Option<&str> {
    line.strip_prefix("////").and_then(|documentation| {
        if documentation.starts_with('/') {
            None
        } else {
            Some(documentation.trim())
        }
    })
}

fn parse_line(line: &str, skip_validation: bool) -> Result<HeaderContent, String> {
    if let Some(flags) = line.strip_prefix("Flags:") {
        parse_flags(flags.trim_start())
    } else if let Some(timer) = line.strip_prefix("timer:") {
        parse_timer(timer.trim_start())
    } else if let Some(command) = line.strip_prefix("!!") {
        parse_command(command)
    } else {
        parse_pickup(line, skip_validation)
    }
}

fn parse_flags(flags: &str) -> Result<HeaderContent, String> {
    if flags.is_empty() {
        return Err("empty flagline".to_string());
    }
    let flags = flags.split(',').map(|flag| VString(flag.trim().to_string())).collect();
    Ok(HeaderContent::Flags(flags))
}

fn parse_timer(timer: &str) -> Result<HeaderContent, String> {
    let mut parts = timer.split('|');
    let toggle = parse_uber_identifier(&mut parts).map_err(|err| format!("malformed timer declaration {timer}: {err}"))?;
    let timer = parse_uber_identifier(&mut parts).map_err(|err| format!("malformed timer declaration {timer}: {err}"))?;
    if parts.next().is_some() {
        return Err(format!("Too many parts in timer declaration {timer}"));
    }

    let timer_definition = TimerDefinition { toggle, timer };
    Ok(HeaderContent::Timer(timer_definition))
}

fn parse_command(command: &str) -> Result<HeaderContent, String> {
    HeaderCommand::parse(command).map(HeaderContent::Command)
}

fn parse_pickup(mut pickup: &str, skip_validation: bool) -> Result<HeaderContent, String> {
    let ignore = pickup.starts_with('!');
    if ignore {
        pickup = &pickup[1..];
    }

    let mut parts = pickup.splitn(3, '|');
    let trigger = parse_v_uber_state_trigger(&mut parts).map_err(|err| format!("malformed pickup {pickup}: {err}"))?;

    let item = parts.next().ok_or_else(|| format!("missing parts in pickup {pickup}"))?;
    let item = VItem::parse(item)?;

    let pickup = VPickup { trigger, item, ignore, skip_validation };
    Ok(HeaderContent::Pickup(pickup))
}

#[cfg(test)]
mod tests {
    use crate::item::*;
    use crate::util::*;

    #[test]
    fn item_parsing() {
        assert_eq!(Item::parse("0|5000"), Ok(Item::SpiritLight(5000)));
        assert_eq!(Item::parse("0|-5000"), Ok(Item::RemoveSpiritLight(5000)));
        assert_eq!(Item::parse("1|2"), Ok(Item::Resource(Resource::Ore)));
        assert!(Item::parse("1|-2").is_err());
        assert!(Item::parse("1|5").is_err());
        assert_eq!(Item::parse("2|8"), Ok(Item::Skill(Skill::Launch)));
        assert_eq!(Item::parse("2|120"), Ok(Item::Skill(Skill::AncestralLight1)));
        assert_eq!(Item::parse("2|121"), Ok(Item::Skill(Skill::AncestralLight2)));
        assert!(Item::parse("2|25").is_err());
        assert!(Item::parse("2|-9").is_err());
        assert_eq!(Item::parse("3|28"), Ok(Item::Shard(Shard::LastStand)));
        assert_eq!(Item::parse("5|16"), Ok(Item::Teleporter(Teleporter::Marsh)));
        assert_eq!(Item::parse("9|0"), Ok(Item::Water));
        assert_eq!(Item::parse("9|-0"), Ok(Item::RemoveWater));
        assert_eq!(Item::parse("11|0"), Ok(Item::BonusUpgrade(BonusUpgrade::RapidHammer)));
        assert_eq!(Item::parse("10|31"), Ok(Item::BonusItem(BonusItem::EnergyRegeneration)));
        assert!(Item::parse("8|5|3|6").is_err());
        assert!(Item::parse("8||||").is_err());
        assert!(Item::parse("8|5|3|in|3").is_err());
        assert!(Item::parse("8|5|3|bool|3").is_err());
        assert!(Item::parse("8|5|3|float|hm").is_err());
        assert_eq!(Item::parse("8|5|3|int|6"), Ok(UberState::from_parts("5", "3=6").unwrap().to_item(UberType::Int)));
        assert_eq!(Item::parse("4|0"), Ok(Item::Command(Command::Autosave)));
        assert!(Item::parse("12").is_err());
        assert!(Item::parse("").is_err());
        assert!(Item::parse("0|").is_err());
        assert!(Item::parse("0||400").is_err());
        assert!(Item::parse("7|3").is_err());
        assert!(Item::parse("-0|65").is_err());
    }
}
