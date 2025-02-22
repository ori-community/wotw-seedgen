use super::*;
use crate::{
    item_pool::ItemPool,
    tests::{test_logger, AREAS},
};
use wotw_seedgen_settings::{Difficulty, UniverseSettings, DEFAULT_SPAWN};
use wotw_seedgen_static_assets::{LOC_DATA, STATE_DATA, UBER_STATE_DATA};

#[test]
fn reach_check() {
    test_logger();

    let mut universe_settings = UniverseSettings::new(String::default());
    universe_settings.world_settings[0].difficulty = Difficulty::Gorlek;

    let graph = Graph::compile(
        AREAS.clone(),
        LOC_DATA.clone(),
        STATE_DATA.clone(),
        &universe_settings.world_settings,
    )
    .into_result()
    .unwrap();

    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    let uber_states = UberStates::new(&UBER_STATE_DATA);
    let mut world = World::new(
        &graph,
        spawn,
        &universe_settings.world_settings[0],
        uber_states,
    );
    let output = IntermediateOutput::default();

    // TODO remove if unnecessary; door connections should already be default, visited might be in the future
    // enable default door connections
    for (from, to) in &world.graph.default_door_connections {
        world.set_integer(UberIdentifier::new(27, *from), *to, &output);
        world.set_boolean(UberIdentifier::new(28, *from), true, &output);
    }

    let mut pool = ItemPool::default();
    for item in pool.drain() {
        // TODO this isn't really how a reach check should be set up, the input represents what the world already has, and all the consequences have happened already
        // it would be better to set the inventory directly rather than simulate all the changes
        world.simulate(&item, &output);
    }
    world.modify_spirit_light(10000, &output);

    let reached = world
        .reached()
        .iter()
        .filter_map(|node| match node {
            Node::State(_) | Node::LogicalState(_) => None,
            _ => Some(node.identifier()),
        })
        .collect();

    let all_locations = LOC_DATA
        .entries
        .iter()
        .map(|location| location.identifier.as_str())
        .collect::<FxHashSet<_>>();

    if !(reached == all_locations) {
        let mut diff = all_locations.difference(&reached).collect::<Vec<_>>();
        diff.sort_unstable();
        eprintln!(
            "difference (reached {} / {} items): {:?}",
            reached.len(),
            all_locations.len(),
            diff
        );
    }

    assert_eq!(reached, all_locations);

    let spawn = graph.find_node("GladesTown.Teleporter").unwrap();
    let uber_states = UberStates::new(&UBER_STATE_DATA);
    let mut world = World::new_spawn(
        &graph,
        spawn,
        &universe_settings.world_settings[0],
        uber_states,
    );

    for _ in 0..7 {
        world.modify_max_health(5, &output); // TODO how do this
    }
    for _ in 0..6 {
        world.modify_max_energy(0.5.into(), &output);
    }
    world.set_skill(Skill::DoubleJump, true, &output);
    world.set_shard(Shard::TripleJump, true, &output);

    let reached = world
        .reached()
        .iter()
        .map(|node| node.identifier())
        .collect::<FxHashSet<_>>();
    assert_eq!(
        reached,
        [
            "GladesTown.UpdraftCeilingEX",
            "GladesTown.AboveTpEX",
            "GladesTown.BountyShard",
            "GladesTown.BelowHoleHutEX"
        ]
        .into_iter()
        .collect()
    );
}
