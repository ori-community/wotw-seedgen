use itertools::Itertools;
use wotw_seedgen_data::{assets::UberStateData, seed_language::ast, UberIdentifier};

pub fn uber_identifier_info(
    uber_identifier: &ast::UberIdentifier,
    uber_state_data: &UberStateData,
) -> Option<String> {
    let info = match uber_identifier {
        ast::UberIdentifier::Numeric(numeric) => {
            let member = numeric.member.value.as_option()?.data;
            let identifier = UberIdentifier::new(numeric.group.data, member);

            let entry = uber_state_data.id_lookup.get(&identifier)?;

            match &entry.rando_name {
                None => entry.name.clone(),
                Some(rando_name) => {
                    format!("{rando_name} ({})", entry.name)
                }
            }
        }
        ast::UberIdentifier::Name(name) => {
            let group_lookup = uber_state_data.name_lookup.get(name.group.data.0)?;

            let member = &name.member.value.as_option()?;

            let member_lookup = group_lookup.get(member.data.0)?;

            match member_lookup.as_slice() {
                [single_element] => single_element.to_string(),
                elements => elements
                    .iter()
                    .format_with("\n", |alias, f| f(&format_args!("- {alias}")))
                    .to_string(),
            }
        }
    };

    Some(info)
}
