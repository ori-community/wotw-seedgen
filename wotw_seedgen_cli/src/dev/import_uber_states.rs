use std::io::Write;

use arboard::Clipboard;
use wotw_seedgen::data::assets::{
    self, Pickups, States, UberStateData, UberStateDump, UberStateNameEntry,
};

use crate::Error;

const OVERRIDES: [(&str, f32); 15] = [
    ("player.shardSlots", 3.),
    ("player.positionX", 0.),
    ("player.positionY", 0.),
    ("player.area", 13.),
    ("map.filter", 0.),
    ("player.mouseWorldPositionX", 0.),
    ("player.mouseWorldPositionY", 0.),
    ("rng.randomInteger", 0.),
    ("rng.randomFloat", 0.),
    ("rng.randomBoolean", 0.),
    (
        "statsUberStateGroup.timeAirborneStatSettingSerializedUberState",
        0.,
    ),
    (
        "statsUberStateGroup.timeLongestSingleAirborneStatSettingSerializedUberState",
        0.,
    ),
    (
        "statsUberStateGroup.distanceTravelledStatSettingSerializedFloatUberState",
        0.,
    ),
    ("swampStateGroup.savePedestalSwampIntroTop", 0.),
    ("swampStateGroup.finishedIntroTop", 0.),
];

pub fn import_uber_states() -> Result<(), Error> {
    let mut clipboard = Clipboard::new()?;
    let text = clipboard.get_text()?;

    if !text.starts_with('{') {
        Err("clipboard doesn't contain an UberState dump")?;
    }

    let mut dump: UberStateDump =
        serde_json::from_str(&text).map_err(|err| format!("failed to parse clipboard: {err}"))?;

    let map_segments = dump.groups.get_mut(&22).unwrap();
    assert_eq!(map_segments.name, "mapSegments");

    for map_segment in map_segments.states.values_mut() {
        map_segment.value = 1.;
    }

    let data = UberStateData::from_parts(dump.clone(), &Pickups::default(), &States::default());

    for (name, value) in &OVERRIDES {
        let (group, member) = name.split_once('.').unwrap();

        let UberStateNameEntry::Vanilla(uber_identifiers) = &data.name_lookup[group][member] else {
            panic!("overrides should reference the vanilla names");
        };

        assert_eq!(uber_identifiers.len(), 1);

        let uber_identifier = uber_identifiers[0];

        let dump_member = dump
            .groups
            .get_mut(&uber_identifier.group)
            .unwrap()
            .states
            .get_mut(&uber_identifier.member)
            .unwrap();

        dump_member.value = *value;
    }

    let mut out = assets::file_create("uber_state_dump.json")?;

    serde_json::to_writer_pretty(&mut out, &dump)?;

    writeln!(&mut out)?;

    Ok(())
}
