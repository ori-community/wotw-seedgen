use std::io::Cursor;

use serde::Serialize;
use tokio::sync::RwLockReadGuard;
use wotw_seedgen::{data::UniverseSettings, generate_seed};

use crate::{
    api::GenerateQuery,
    assets::Cache,
    error::{Error, Result},
};

#[derive(Serialize)]
pub struct Universe {
    pub worlds: Vec<Vec<u8>>,
    pub json_spoiler: Option<String>,
    pub text_spoiler: Option<String>,
}

pub fn generate(
    query: GenerateQuery,
    settings: &UniverseSettings,
    cache: RwLockReadGuard<Cache>,
) -> Result<Vec<u8>> {
    let universe = generate_seed(
        &cache.graph,
        &cache.base.loc_data,
        &cache.base.uber_state_data,
        &*cache,
        settings,
        false,
    )
    .map_err(Error::Generate)?;

    let worlds = universe
        .worlds
        .into_iter()
        .map(|seed| {
            let mut bytes = Cursor::new(vec![]);

            seed.package(&mut bytes, true)
                .map_err(|err| Error::Generate(err.to_string()))?;

            Ok(bytes.into_inner())
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
    };

    let mut bytes = vec![];
    ciborium::into_writer(&universe, &mut bytes).unwrap();

    Ok(bytes)
}
