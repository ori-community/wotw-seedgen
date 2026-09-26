use itertools::Itertools;
use wotw_seedgen_data::{
    assets::{UberStateData, UberStateNameEntry},
    parse::SpannedOption,
    seed_language::ast,
    UberIdentifier,
};

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

            match group_lookup.get(member.data.0)? {
                UberStateNameEntry::Vanilla(uber_identifiers) => {
                    match uber_identifiers.as_slice() {
                        [single_element] => single_element.to_string(),
                        elements => elements
                            .iter()
                            .format_with("\n", |identifier, f| f(&format_args!("- {identifier}")))
                            .to_string(),
                    }
                }
                UberStateNameEntry::Rando(rando_group) => match &name.pickup {
                    SpannedOption::Some(pickup) => {
                        let pickup = pickup.identifier.value.as_option()?.data.0;
                        rando_group.members.get(pickup)?.to_string()
                    }
                    SpannedOption::None(_) => rando_group.root_member.as_ref()?.to_string(),
                },
            }
        }
    };

    Some(info)
}
