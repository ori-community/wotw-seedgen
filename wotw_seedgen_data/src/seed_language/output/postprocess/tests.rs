use std::sync::LazyLock;

use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;

use crate::{
    assets::{AssetCacheValues, TEST_ASSETS},
    seed_language::{
        ast::ClientEvent,
        compile::{clean_water, store_boolean},
        output::{
            postprocess::{count_in_zone_message, postprocess},
            CommandString, CommandVoid, CommandsOutput, Event, IntermediateOutput, ItemMetadata,
            PlaceholderMap, StringOrPlaceholder, Trigger,
        },
    },
    UberIdentifier, Zone,
};

static MARSH_TRIGGER: LazyLock<Trigger> = LazyLock::new(|| {
    let uber_identifier = TEST_ASSETS
        .loc_data()
        .entries
        .iter()
        .find(|entry| entry.zone == Zone::Marsh && entry.value.is_none())
        .unwrap()
        .uber_identifier;

    Trigger::loc_data_trigger(uber_identifier, None)
});

macro_rules! test {
    ($commands:expr, [$($placeholders:expr),+ $(,)?] $(,)?) => {
        let actual = test_postprocess($commands);
        let expected = vec![$(PlaceholderMap {
            strings: $placeholders,
        }),+];

        assert_eq!(actual, expected);
    };
}

#[test]
fn zone_of() {
    for command in [clean_water(), test_call_function()] {
        let placeholder = StringOrPlaceholder::ZoneOfPlaceholder(Box::new(command.clone()));

        test!(
            [test_output(vec![
                placeholder_event(placeholder.clone()),
                on_marsh(command.clone()),
            ])],
            [FxHashMap::from_iter([(
                placeholder.clone(),
                Zone::Marsh.to_string().into()
            )])],
        );

        test!(
            [
                test_output(vec![
                    placeholder_event(placeholder.clone()),
                    on_multiworld(command),
                ]),
                test_output(vec![on_marsh_multiworld()]),
            ],
            [
                FxHashMap::from_iter([(
                    placeholder,
                    format!("<world>1</>'s {}", Zone::Marsh).into()
                )]),
                FxHashMap::default(),
            ],
        );
    }
}

#[test]
fn item_on() {
    let placeholder = StringOrPlaceholder::ItemOnPlaceholder(Box::new(MARSH_TRIGGER.clone()));

    test!(
        [test_output(vec![
            placeholder_event(placeholder.clone()),
            on_marsh(clean_water()),
        ])],
        [FxHashMap::from_iter([(
            placeholder.clone(),
            clean_water().contained_messages().next().unwrap().clone(),
        )])],
    );

    // item_on currently utilizes the item messages generated during placement,
    // it has no actual multiworld awareness that could be tested here
}

#[test]
fn count_in_zone() {
    for command in [clean_water(), test_call_function()] {
        let placeholder =
            StringOrPlaceholder::CountInZonePlaceholder(vec![command.clone()], Zone::Marsh);

        test!(
            [test_output(vec![
                placeholder_event(placeholder.clone()),
                on_marsh(command.clone()),
            ])],
            [FxHashMap::from_iter([(
                placeholder.clone(),
                count_in_zone_message(
                    vec![(
                        &on_marsh(command.clone()),
                        TEST_ASSETS.loc_data().entries.first().unwrap(),
                    )],
                    &ItemMetadata::new(),
                )
            )])],
        );

        test!(
            [
                test_output(vec![
                    placeholder_event(placeholder.clone()),
                    on_marsh_multiworld(),
                ]),
                test_output(vec![on_multiworld(command)]),
            ],
            [
                FxHashMap::from_iter([(
                    placeholder,
                    count_in_zone_message(
                        vec![(
                            &on_marsh_multiworld(),
                            TEST_ASSETS.loc_data().entries.first().unwrap(),
                        )],
                        &ItemMetadata::new(),
                    )
                )]),
                FxHashMap::default(),
            ],
        );
    }
}

fn test_postprocess<const N: usize>(commands: [CommandsOutput; N]) -> Vec<PlaceholderMap> {
    let mut output = commands.map(|commands| IntermediateOutput {
        commands,
        ..IntermediateOutput::default()
    });
    let mut output_iter_mut = output.iter_mut();
    let mut worlds = [(); N].map(|()| output_iter_mut.next().unwrap());

    postprocess(&mut worlds, TEST_ASSETS.loc_data(), &mut Pcg64Mcg::new(0))
}

fn test_output(events: Vec<Event>) -> CommandsOutput {
    CommandsOutput {
        events,
        lookup: vec![CommandVoid::Multi {
            commands: Vec::new(),
        }],
        ..CommandsOutput::NONE
    }
}

fn test_call_function() -> CommandVoid {
    CommandVoid::CallFunction {
        booleans: vec![],
        integers: vec![],
        floats: vec![],
        strings: vec![],
        index: 0,
    }
}

fn placeholder_event(placeholder: StringOrPlaceholder) -> Event {
    Event {
        trigger: Trigger::ClientEvent(ClientEvent::Spawn),
        command: debug_log(placeholder),
    }
}

fn debug_log<S: Into<CommandString>>(message: S) -> CommandVoid {
    CommandVoid::DebugLog {
        message: message.into(),
    }
}

fn on_marsh(command: CommandVoid) -> Event {
    Event {
        trigger: MARSH_TRIGGER.clone(),
        command,
    }
}

fn on_multiworld(command: CommandVoid) -> Event {
    Event {
        trigger: Trigger::multiworld(0),
        command,
    }
}

fn on_marsh_multiworld() -> Event {
    Event {
        trigger: MARSH_TRIGGER.clone(),
        command: store_boolean(UberIdentifier::multiworld(0), true),
    }
}
