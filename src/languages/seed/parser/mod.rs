pub mod parse_item;
pub mod parse_icon;

use crate::util::{UberIdentifier, UberState};

pub(crate) fn parse_uber_identifier<'a, I>(parts: &mut I) -> Result<UberIdentifier, String>
where
    I: Iterator<Item = &'a str>,
{
    let uber_group = parts.next().ok_or_else(|| String::from("missing uber group"))?;
    let uber_id = parts.next().ok_or_else(|| String::from("missing uber id"))?;

    UberIdentifier::from_parts(uber_group, uber_id)
}

pub(crate) fn parse_uber_state<'a, I>(parts: &mut I) -> Result<UberState, String>
where
    I: Iterator<Item = &'a str>,
{
    let uber_group = parts.next().ok_or_else(|| String::from("missing uber group"))?;
    let uber_id = parts.next().ok_or_else(|| String::from("missing uber id"))?;

    UberState::from_parts(uber_group, uber_id)
}
