use std::str::FromStr;

use crate::{VItem, util::Icon};

use crate::header::{HeaderCommand, ParameterDefault, V};

fn parse_amount(item: &mut &str) -> V<i32> {
    if let Some(index) = item.find("x ") {
        let amount = item[..index].trim();

        if let Ok(amount) = V::try_wrap(amount) {
            *item = &item[index + 1..];
            return amount;
        }
    }
    V::Literal(1)
}
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    Some(s.to_string())
}

fn include_command(include: &str) -> Result<HeaderCommand, String> {
    let name = non_empty(include).ok_or_else(|| "missing name in include command".to_string())?;
    Ok(HeaderCommand::Include { name })
}
fn exclude_command(exclude: &str) -> Result<HeaderCommand, String> {
    let name = non_empty(exclude).ok_or_else(|| "missing name in exclude command".to_string())?;
    Ok(HeaderCommand::Exclude { name })
}
fn add_command(mut item: &str) -> Result<HeaderCommand, String> {
    let amount = parse_amount(&mut item);
    let item = VItem::parse(item)?;

    Ok(HeaderCommand::Add { item, amount })
}
fn remove_command(mut item: &str) -> Result<HeaderCommand, String> {
    let amount = parse_amount(&mut item);
    let item = VItem::parse(item)?;

    Ok(HeaderCommand::Remove { item, amount })
}
fn name_command(naming: &str) -> Result<HeaderCommand, String> {
    let (item, name) = naming.split_once(' ').ok_or_else(|| "missing name".to_string())?;
    let item = VItem::parse(item)?;
    let name = non_empty(name).ok_or_else(|| "missing display name".to_string())?;
    let name = V::wrap(&name);

    Ok(HeaderCommand::Name { item, name })
}
fn display_command(display: &str) -> Result<HeaderCommand, String> {
    let (item, name) = display.split_once(' ').ok_or_else(|| "missing display name".to_string())?;
    let item = VItem::parse(item)?;
    let name = non_empty(name).ok_or_else(|| "missing display name".to_string())?;
    let name = V::wrap(&name);

    Ok(HeaderCommand::Display { item, name })
}
fn description_command(description: &str) -> Result<HeaderCommand, String> {
    let (item, description) = description.split_once(' ').ok_or_else(|| "missing description".to_string())?;
    let item = VItem::parse(item)?;
    let description = non_empty(description).ok_or_else(|| "missing description".to_string())?;
    let description = V::wrap(&description);

    Ok(HeaderCommand::Description { item, description })
}
fn price_command(price: &str) -> Result<HeaderCommand, String> {
    let (item, price) = price.split_once(' ').ok_or_else(|| "missing price".to_string())?;
    let item = VItem::parse(item)?;
    let price = V::try_wrap(price).map_err(|_| format!("invalid price {price}"))?;

    Ok(HeaderCommand::Price { item, price })
}
fn icon_command(icon: &str) -> Result<HeaderCommand, String> {
    let (item, icon) = icon.split_once(' ').ok_or_else(|| "missing icon".to_string())?;
    let item = VItem::parse(item)?;
    let icon = Icon::parse(icon)?;

    Ok(HeaderCommand::Icon { item, icon })
}
fn parameter_command(parameter: &str) -> Result<HeaderCommand, String> {
    let (identifier, default) = parameter.split_once(' ').ok_or_else(|| "missing default value".to_string())?;
    let identifier = non_empty(identifier).ok_or_else(|| "missing identifier".to_string())?;

    let default = ParameterDefault::from_str(default)?;

    Ok(HeaderCommand::Parameter { identifier, default })
}
fn set_command(identifier: &str) -> Result<HeaderCommand, String> {
    let state = non_empty(identifier).ok_or_else(|| "missing identifier".to_string())?;

    Ok(HeaderCommand::Set { state })
}
fn if_command(comparison: &str) -> Result<HeaderCommand, String> {
    let (parameter, value) = comparison.split_once(' ').ok_or_else(|| "missing default value".to_string())?;
    let parameter = non_empty(parameter).ok_or_else(|| "missing parameter".to_string())?;
    let value = non_empty(value).ok_or_else(|| "missing value".to_string())?;

    Ok(HeaderCommand::If { parameter, value })
}
fn endif_command() -> HeaderCommand {
    HeaderCommand::EndIf
}

impl HeaderCommand {
    pub fn parse(command: &str) -> Result<HeaderCommand, String> {
        let command = if let Some(include) = command.strip_prefix("include ") {
            include_command(include.trim())?
        } else if let Some(exclude) = command.strip_prefix("exclude ") {
            exclude_command(exclude.trim())?
        } else if let Some(item) = command.strip_prefix("add ") {
            add_command(item.trim()).map_err(|err| format!("{err} in add command {command}"))?
        } else if let Some(item) = command.strip_prefix("remove ") {
            remove_command(item.trim()).map_err(|err| format!("{err} in remove command {command}"))?
        } else if let Some(naming) = command.strip_prefix("name ") {
            name_command(naming.trim()).map_err(|err| format!("{err} in name command {command}"))?
        } else if let Some(display) = command.strip_prefix("display ") {
            display_command(display.trim()).map_err(|err| format!("{err} in display command {command}"))?
        } else if let Some(description) = command.strip_prefix("description ") {
            description_command(description.trim()).map_err(|err| format!("{err} in description command {command}"))?
        } else if let Some(price) = command.strip_prefix("price ") {
            price_command(price.trim()).map_err(|err| format!("{err} in price command {command}"))?
        } else if let Some(icon) = command.strip_prefix("icon ") {
            icon_command(icon.trim()).map_err(|err| format!("{err} in icon command {command}"))?
        } else if let Some(parameter) = command.strip_prefix("parameter ") {
            parameter_command(parameter.trim()).map_err(|err| format!("{err} in parameter command {command}"))?
        } else if let Some(identifier) = command.strip_prefix("set ") {
            set_command(identifier.trim()).map_err(|err| format!("{err} in set command {command}"))?
        } else if let Some(comparison) = command.strip_prefix("if ") {
            if_command(comparison.trim()).map_err(|err| format!("{err} in if command {command}"))?
        } else if command.trim_end() == "endif" {
            endif_command()
        } else {
            return Err(format!("Unknown command {command}"));
        };
        Ok(command)
    }
}
