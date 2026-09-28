use std::fmt::Write;

use itertools::Itertools;
use wotw_seedgen_data::{
    assets::{UberStateData, UberStateDataEntry, UberStateNameEntry},
    parse::SpannedOption,
    seed_language::ast,
    UberIdentifier,
};

pub fn uber_identifier_info(
    uber_identifier: &ast::UberIdentifier,
    uber_state_data: &UberStateData,
) -> Option<String> {
    let mut info = String::new();

    match uber_identifier {
        ast::UberIdentifier::Numeric(numeric) => {
            let member = numeric.member.value.as_option()?.data;
            let identifier = UberIdentifier::new(numeric.group.data, member);

            let entry = uber_state_data.id_lookup.get(&identifier)?;

            let _ = match &entry.rando_name {
                None => write!(info, "{}", entry.name),
                Some(rando_name) => {
                    write!(info, "{rando_name} ({})", entry.name)
                }
            };

            write_expression_aliases(&mut info, entry);
        }
        ast::UberIdentifier::Name(name) => {
            let group_lookup = uber_state_data.name_lookup.get(name.group.data.0)?;

            let member = name.member.value.as_option()?;

            match group_lookup.get(member.data.0)? {
                UberStateNameEntry::Vanilla(uber_identifiers) => {
                    match uber_identifiers.as_slice() {
                        [uber_identifier] => {
                            let entry = uber_state_data.id_lookup.get(uber_identifier)?;

                            let _ = match &entry.rando_name {
                                None => write!(info, "{uber_identifier}"),
                                Some(rando_name) => {
                                    write!(info, "{rando_name} ({uber_identifier})")
                                }
                            };

                            write_expression_aliases(&mut info, entry);
                        }
                        ambiguous => {
                            let _ = write!(
                                &mut info,
                                "ambiguous: could be {}",
                                ambiguous.iter().format(" or ")
                            );
                        }
                    }
                }
                UberStateNameEntry::Rando(rando_group) => {
                    let alias = match &name.pickup {
                        SpannedOption::Some(pickup) => {
                            let pickup = pickup.identifier.value.as_option()?.data.0;
                            rando_group.members.get(pickup)?
                        }
                        SpannedOption::None(_) => rando_group.root_member.as_ref()?,
                    };

                    let entry = uber_state_data.id_lookup.get(&alias.uber_identifier)?;

                    let _ = write!(&mut info, "{} ({})", entry.name, alias.uber_identifier);

                    if let Some(value) = alias.value {
                        let _ = write!(&mut info, " >= {value}");
                    }

                    write_expression_aliases(&mut info, entry);
                }
            }
        }
    }

    Some(info)
}

fn write_expression_aliases(info: &mut String, entry: &UberStateDataEntry) {
    if entry.expression_aliases.is_empty() {
        return;
    }

    let _ = write!(info, "\n\nExpression aliases:\n");

    for alias in &entry.expression_aliases {
        let _ = write!(
            info,
            "\n- {rando_name} ({name} >= {value})",
            rando_name = alias.name,
            name = entry.name,
            value = alias.value
        );
    }
}
