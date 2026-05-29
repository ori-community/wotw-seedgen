use wotw_seedgen::{
    LogicalDifficulty,
    data::{
        Difficulty,
        logic_language::output::{Graph, Node},
    },
};

use crate::api::logic::SpawnAnchors;

impl SpawnAnchors {
    pub fn new(graph: &Graph) -> Self {
        let moki_spawn_locations = Difficulty::Moki.spawn_locations();
        let spawn_locations = Difficulty::Gorlek.spawn_locations();

        let mut identifiers = vec![];
        let mut moki_teleporters = Vec::with_capacity(moki_spawn_locations.len());
        let mut teleporters = Vec::with_capacity(spawn_locations.len());

        for anchor in graph
            .nodes
            .iter()
            .filter_map(Node::try_as_anchor_ref)
            .filter(|anchor| anchor.can_spawn())
        {
            eprintln!("anchor {}", anchor.identifier);

            if spawn_locations.contains(&anchor.identifier.as_str()) {
                teleporters.push(identifiers.len());

                if moki_spawn_locations.contains(&anchor.identifier.as_str()) {
                    moki_teleporters.push(identifiers.len());
                }
            }

            identifiers.push(anchor.identifier.clone());
        }

        SpawnAnchors {
            identifiers,
            moki_teleporters,
            teleporters,
        }
    }
}
