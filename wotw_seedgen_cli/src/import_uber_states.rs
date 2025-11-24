use std::{fs::File, io::Write};

use arboard::Clipboard;
use wotw_seedgen_assets::{LocData, StateData, UberStateData, UberStateDump};

use crate::Error;

const OVERRIDES: [(&str, f32); 14] = [
    ("player.positionX", 0.),
    ("player.positionY", 0.),
    ("player.currentArea", 13.),
    ("mapSegments.segment12701", 1.),
    ("input.mouseWorldPositionX", 0.),
    ("input.mouseWorldPositionY", 0.),
    ("randomValueGenerator.randomInt", 0.),
    ("randomValueGenerator.randomFloat", 0.),
    ("randomValueGenerator.randomBoolean", 0.),
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

    let data = UberStateData::from_parts(dump.clone(), &LocData::default(), &StateData::default());

    for (name, value) in &OVERRIDES {
        let (group, member) = name.split_once('.').unwrap();
        let uber_identifier = data.name_lookup[group][member][0].uber_identifier;

        let dump_member = dump
            .groups
            .get_mut(&uber_identifier.group)
            .unwrap()
            .states
            .get_mut(&uber_identifier.member)
            .unwrap();

        dump_member.value = *value;
    }

    let mut out = File::create("uber_state_dump.json")?;

    serde_json::to_writer_pretty(&mut out, &dump)?;

    writeln!(&mut out)?;

    Ok(())
}
