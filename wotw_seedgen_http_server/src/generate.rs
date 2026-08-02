use std::io::Cursor;

use serde::Serialize;
use tokio::sync::RwLockReadGuard;
use wotw_seedgen::{data::UniverseSettings, log_capture::Record};

use crate::{
    api::GenerateQuery,
    assets::Cache,
    error::{Error, Result},
};

#[derive(Serialize)]
pub struct Universe {
    pub worlds: Vec<ciborium::value::Value>,
    pub json_spoiler: Option<String>,
    pub text_spoiler: Option<String>,
    pub logs: Vec<Record>,
}

pub fn generate(
    query: GenerateQuery,
    settings: &UniverseSettings,
    cache: RwLockReadGuard<Cache>,
) -> Result<Vec<u8>> {
    let max_log_level = query.max_log_level.unwrap_or_default().into();

    let (universe, logs) = cache
        .generate(settings, max_log_level)
        .map_err(Error::Generate)?;

    let worlds = universe
        .worlds
        .into_iter()
        .map(|seed| {
            let mut bytes = Cursor::new(vec![]);

            seed.package(&mut bytes, true)
                .map_err(|err| Error::Generate(err.to_string()))?;

            Ok(ciborium::value::Value::Bytes(bytes.into_inner()))
        })
        .collect::<Result<Vec<_>>>()?;

    let json_spoiler = query
        .json_spoiler
        .unwrap_or_default()
        .then(|| serde_json::to_string(&universe.spoiler).unwrap());
    let text_spoiler = query
        .text_spoiler
        .unwrap_or_default()
        .then(|| universe.spoiler.to_string());

    let universe = Universe {
        worlds,
        json_spoiler,
        text_spoiler,
        logs,
    };

    let mut bytes = vec![];
    ciborium::into_writer(&universe, &mut bytes).unwrap();

    Ok(bytes)
}
