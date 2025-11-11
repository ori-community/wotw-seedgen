use std::fs::File;

use crate::{files, Error};

pub fn sort_uber_states() -> Result<(), Error> {
    let logic_access = files::logic_access("")?;

    let dump = logic_access.uber_state_dump()?;

    let out = File::create("uber_state_dump.json")?;

    serde_json::to_writer_pretty(out, &dump)?;

    Ok(())
}
