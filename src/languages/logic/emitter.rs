use rustc_hash::{FxHashSet, FxHashMap};

use super::{parser::{self, AreaTree}, tokenizer::Metadata, Location, NamedState};
use crate::{world::{
    graph::{self, Graph, Node},
    requirements::Requirement,
}, settings::{Difficulty, Trick}, Settings};
use crate::item::Skill;
use crate::util::Zone;

struct EmitterContext<'a> {
    definitions: &'a FxHashMap<&'a str, parser::Group<'a>>,
    settings: &'a Settings,
    validate: bool,
    node_map: FxHashMap<&'a str, usize>,
    used_states: FxHashSet<&'a str>,
}

fn build_trick_requirement(trick: &Trick, out: Requirement, context: &mut EmitterContext) -> Requirement {
    if context.settings.any_contain_trick(trick) {
        if context.settings.all_contain_trick(trick) {
            out
        } else {
            build_and(vec![
                Requirement::Trick(*trick),
                out
            ])
        }
    } else {
        Requirement::Impossible
    }
}

fn build_difficulty_requirement(difficulty: Difficulty, out: Requirement, region: bool, context: &mut EmitterContext) -> Requirement {
    if region {
        if context.settings.any_have_difficulty(difficulty) {
            if context.settings.lowest_difficulty() >= difficulty {
                out
            } else {
                build_and(vec![
                    Requirement::Difficulty(difficulty),
                    out
                ])
            }
        } else {
            Requirement::Impossible
        }
    } else if context.settings.highest_difficulty() >= difficulty {
        if context.settings.lowest_difficulty() >= difficulty {
            out
        } else {
            build_and(vec![
                Requirement::Difficulty(difficulty),
                out
            ])
        }
    } else {
        Requirement::Impossible
    }
}

const HARD_BOSS_HEALTH_MULTIPLIER: f32 = 1.8;

fn build_boss_requirement(health: u16, context: &EmitterContext) -> Requirement {
    if context.settings.any_play_hard() {
        if context.settings.all_play_hard() {
            Requirement::Boss(f32::from(health) * HARD_BOSS_HEALTH_MULTIPLIER)
        } else {
            Requirement::Or(vec![
                Requirement::And(vec![
                    Requirement::NormalGameDifficulty,
                    Requirement::Boss(f32::from(health)),
                ]),
                Requirement::Boss(f32::from(health) * HARD_BOSS_HEALTH_MULTIPLIER),
            ])
        }
    } else {
        Requirement::Boss(f32::from(health))
    }
}

fn build_requirement<'a>(requirement: &parser::Requirement<'a>, region: bool, context: &mut EmitterContext<'a>) -> Requirement {
    match requirement {
        parser::Requirement::Free => Requirement::Free,
        parser::Requirement::Definition(identifier) => build_requirement_group(&context.definitions[identifier], region, context),
        parser::Requirement::Difficulty(difficulty) => build_difficulty_requirement(*difficulty, Requirement::Free, region, context),
        parser::Requirement::Trick(glitch) => build_trick_requirement(glitch, Requirement::Free, context),
        parser::Requirement::Skill(skill) => Requirement::Skill(*skill),
        parser::Requirement::EnergySkill(skill, amount) => Requirement::EnergySkill(*skill, (*amount).into()),
        parser::Requirement::SpiritLight(amount) => Requirement::SpiritLight(*amount),
        parser::Requirement::Resource(resource, amount) => Requirement::Resource(*resource, *amount),
        parser::Requirement::Shard(shard) => Requirement::Shard(*shard),
        parser::Requirement::Teleporter(teleporter) => Requirement::Teleporter(*teleporter),
        parser::Requirement::Water => Requirement::Water,
        parser::Requirement::State(state) => {
            if context.validate { context.used_states.insert(state); }
            Requirement::State(context.node_map[state])
        },
        parser::Requirement::Damage(amount) => Requirement::Damage(f32::from(*amount)),
        parser::Requirement::Danger(amount) => Requirement::Danger(f32::from(*amount)),
        parser::Requirement::Combat(enemies) => Requirement::Combat(enemies.clone()),
        parser::Requirement::Boss(health) => build_boss_requirement(*health, context),
        parser::Requirement::BreakWall(health) => Requirement::BreakWall(f32::from(*health)),
        parser::Requirement::BreakCrystal =>
            build_or(vec![
                Requirement::Skill(Skill::Sword),
                Requirement::Skill(Skill::Hammer),
                Requirement::EnergySkill(Skill::Bow, 1.0),
                build_difficulty_requirement(Difficulty::Gorlek, Requirement::EnergySkill(Skill::Shuriken, 1.0), false, context),
                build_difficulty_requirement(Difficulty::Gorlek, Requirement::EnergySkill(Skill::Grenade, 1.0), false, context),
                build_difficulty_requirement(Difficulty::Unsafe, Requirement::EnergySkill(Skill::Spear, 1.0), false, context),
            ]),
        parser::Requirement::ShurikenBreak(health) => build_trick_requirement(&Trick::ShurikenBreak, Requirement::ShurikenBreak(f32::from(*health)), context),
        parser::Requirement::SentryBreak(health) => build_trick_requirement(&Trick::SentryBreak, Requirement::SentryBreak(f32::from(*health)), context),
        parser::Requirement::HammerBreak => build_trick_requirement(&Trick::HammerBreak, Requirement::Skill(Skill::Hammer), context),
        parser::Requirement::SpearBreak => build_trick_requirement(&Trick::SpearBreak, Requirement::EnergySkill(Skill::Spear, 1.0), context),
        parser::Requirement::SentryJump(amount) => 
            build_and(vec![
                Requirement::EnergySkill(Skill::Sentry, (*amount).into()),
                build_or(vec![
                    build_trick_requirement(&Trick::SwordSentryJump, Requirement::Skill(Skill::Sword), context),
                    build_trick_requirement(&Trick::HammerSentryJump, Requirement::Skill(Skill::Hammer), context),
                ]),
            ]),
        parser::Requirement::SwordSentryJump(amount) => build_trick_requirement(&Trick::SwordSentryJump,
            Requirement::And(vec![
                Requirement::EnergySkill(Skill::Sentry, (*amount).into()),
                Requirement::Skill(Skill::Sword),
            ]), context),
        parser::Requirement::HammerSentryJump(amount) => build_trick_requirement(&Trick::HammerSentryJump,
            Requirement::And(vec![
                Requirement::EnergySkill(Skill::Sentry, (*amount).into()),
                Requirement::Skill(Skill::Hammer),
            ]), context),
        parser::Requirement::SentryBurn(amount) => build_trick_requirement(&Trick::SentryBurn, Requirement::EnergySkill(Skill::Sentry, (*amount).into()), context),
        parser::Requirement::LaunchSwap => build_trick_requirement(&Trick::LaunchSwap, Requirement::Skill(Skill::Launch), context),
        parser::Requirement::SentrySwap(amount) => build_trick_requirement(&Trick::SentrySwap, Requirement::EnergySkill(Skill::Sentry, (*amount).into()), context),
        parser::Requirement::FlashSwap => build_trick_requirement(&Trick::FlashSwap, Requirement::NonConsumingEnergySkill(Skill::Flash), context),
        parser::Requirement::BlazeSwap(amount) => build_trick_requirement(&Trick::BlazeSwap, Requirement::EnergySkill(Skill::Blaze, (*amount).into()), context),
        parser::Requirement::WaveDash => build_trick_requirement(&Trick::WaveDash, Requirement::And(vec![Requirement::Skill(Skill::Dash), Requirement::NonConsumingEnergySkill(Skill::Regenerate)]), context),
        parser::Requirement::GrenadeJump => build_trick_requirement(&Trick::GrenadeJump, Requirement::NonConsumingEnergySkill(Skill::Grenade), context),
        parser::Requirement::GrenadeCancel => Requirement::NonConsumingEnergySkill(Skill::Grenade),
        parser::Requirement::HammerJump => build_trick_requirement(&Trick::HammerJump, Requirement::And(vec![Requirement::Skill(Skill::Hammer), Requirement::Skill(Skill::DoubleJump)]), context),
        parser::Requirement::SwordJump => build_trick_requirement(&Trick::SwordJump, Requirement::And(vec![Requirement::Skill(Skill::Sword), Requirement::Skill(Skill::DoubleJump)]), context),
        parser::Requirement::GrenadeRedirect(amount) => build_trick_requirement(&Trick::GrenadeRedirect, Requirement::EnergySkill(Skill::Grenade, (*amount).into()), context),
        parser::Requirement::SentryRedirect(amount) => build_trick_requirement(&Trick::SentryRedirect, Requirement::EnergySkill(Skill::Sentry, (*amount).into()), context),
        parser::Requirement::GlideJump => build_trick_requirement(&Trick::GlideJump, Requirement::Skill(Skill::Glide), context),
        parser::Requirement::GlideHammerJump => build_trick_requirement(&Trick::GlideHammerJump, Requirement::And(vec![Requirement::Skill(Skill::Glide), Requirement::Skill(Skill::Hammer)]), context),
        parser::Requirement::SpearJump(amount) => build_trick_requirement(&Trick::SpearJump, Requirement::EnergySkill(Skill::Spear, (*amount).into()), context),
    }
}

fn build_and(mut ands: Vec<Requirement>) -> Requirement {
    if ands.iter().any(|and| matches!(and, Requirement::Impossible)) {
        return Requirement::Impossible;
    }
    ands.retain(|and| !matches!(and, Requirement::Free));
    if ands.len() == 1 {
        return ands.pop().unwrap();
    }
    if ands.is_empty() {
        return Requirement::Free;
    }
    Requirement::And(ands)
}
fn build_or(mut ors: Vec<Requirement>) -> Requirement {
    if ors.iter().any(|or| matches!(or, Requirement::Free)) {
        return Requirement::Free;
    }
    ors.retain(|or| !matches!(or, Requirement::Impossible));
    if ors.len() == 1 {
        return ors.pop().unwrap();
    }
    if ors.is_empty() {
        return Requirement::Impossible;
    }
    Requirement::Or(ors)
}

fn build_requirement_group<'a>(group: &parser::Group<'a>, region: bool, context: &mut EmitterContext<'a>) -> Requirement {
    let lines = group.lines.iter().map(|line| {
        let mut parts = vec![];
        if !line.ands.is_empty() {
            let ands = line.ands.iter().map(|and| build_requirement(and, region, context)).collect::<Vec<_>>();
            parts.push(build_and(ands));
        }
        if !line.ors.is_empty() {
            let ors = line.ors.iter().map(|or| build_requirement(or, region, context)).collect::<Vec<_>>();
            parts.push(build_or(ors));
        }
        if let Some(subgroup) = &line.group {
            parts.push(build_requirement_group(subgroup, region, context));
        }
        build_and(parts)
    }).collect::<Vec<_>>();
    build_or(lines)
}

fn add_entry<'a>(graph: &mut FxHashMap<&'a str, usize>, key: &'a str, index: usize) -> Result<(), String> {
    if graph.insert(key, index).is_some() {
        return Err(format!("Name {} was used multiple times ambiguously", key));
    }
    Ok(())
}

pub fn emit(areas: &AreaTree, metadata: &Metadata, locations: &[Location], state_map: &[NamedState], settings: &Settings, validate: bool) -> Result<Graph, String> {
    let node_count = areas.anchors.len() + locations.len() + metadata.states.len();
    let mut graph = Vec::with_capacity(node_count);
    let mut used_states = FxHashSet::default();
    used_states.reserve(metadata.states.len());
    let mut node_map = FxHashMap::default();
    node_map.reserve(node_count);

    for location in locations {
        let name = &location.name[..];
        let zone = match &location.zone[..] {
            "Inkwater Marsh" => Zone::Marsh,
            "Midnight Burrows" => Zone::Burrows,
            "Kwoloks Hollow" => Zone::Hollow,
            "Wellspring Glades" => Zone::Glades,
            "The Wellspring" => Zone::Wellspring,
            "Luma Pools" => Zone::Pools,
            "Silent Woods" => Zone::Woods,
            "Baurs Reach" => Zone::Reach,
            "Mouldwood Depths" => Zone::Depths,
            "Windswept Wastes" => Zone::Wastes,
            "Windtorn Ruins" => Zone::Ruins,
            "Willows End" => Zone::Willow,
            "Shop" => Zone::Shop,
            "Void" => Zone::Void,
            _ => return Err(format!("invalid zone {} in loc_data", location.zone)),
        };

        let index = graph.len();
        add_entry(&mut node_map, &location.name, index)?;

        if metadata.quests.contains(name) {
            graph.push(Node::Quest(graph::Quest {
                identifier: location.name.clone(),
                zone,
                index,
                uber_state: location.uber_state.clone(),
                position: location.position.clone(),
            }));
        } else {
            graph.push(Node::Pickup(graph::Pickup {
                identifier: location.name.clone(),
                zone,
                index,
                uber_state: location.uber_state.clone(),
                position: location.position.clone(),
            }));
        }
    }
    for &state in &metadata.states {
        let index = graph.len();
        add_entry(&mut node_map, state, index)?;

        let mut uber_state = None;
        if let Some(named_state) = state_map.iter().find(|&named_state| named_state.name == state) {
            uber_state = Some(named_state.uber_state.clone());
        } else if validate {
            log::trace!("Couldn't find an entry for {} in the state table", state);
        }

        graph.push(Node::State(graph::State {
            identifier: state.to_owned(),
            index,
            uber_state,
        }));
    }

    let length = graph.len();
    for (index, anchor) in areas.anchors.iter().enumerate() {
        add_entry(&mut node_map, anchor.identifier, length + index)?;
    }

    let mut context = EmitterContext {
        definitions: &areas.definitions,
        settings,
        validate,
        node_map,
        used_states,
    };

    for anchor in &areas.anchors {
        let region = anchor.region();
        let region = areas.regions.get(region);
        let mut region_requirement = None;
        if let Some(group) = region {
            region_requirement = Some(build_requirement_group(group, true, &mut context));
        }

        let refills = anchor.refills.iter().map(|refill| {
            let mut requirement = Requirement::Free;
            if let Some(group) = &refill.requirements {
                requirement = build_requirement_group(group, false, &mut context);
            }
            graph::Refill {
                name: refill.name,
                requirement,
            }
        }).collect::<Vec<_>>();

        let mut connections = Vec::with_capacity(anchor.connections.len());
        for connection in &anchor.connections {
            let mut requirement = build_requirement_group(&connection.requirements, false, &mut context);
            if let Some(region_requirement) = &region_requirement {
                requirement = build_and(vec![region_requirement.clone(), requirement]);
            }

            let to = *context.node_map.get(connection.identifier).ok_or_else(|| format!("Anchor {} connects to {:?} {} which doesn't actually exist", anchor.identifier, connection.name, connection.identifier))?;

            connections.push(graph::Connection {
                to,
                requirement,
            });
        }

        graph.push(Node::Anchor(graph::Anchor {
            identifier: anchor.identifier.to_owned(),
            position: anchor.position.clone(),
            can_spawn: anchor.can_spawn,
            index: graph.len(),
            refills,
            connections,
        }));
    }

    if validate {
        for anchor in &areas.anchors {
            for connection in &anchor.connections {
                let expected_type = graph[context.node_map[connection.identifier]].node_type();
                if connection.name != expected_type {
                    return Err(format!("Anchor {} connects to {:?} {} which is actually a {:?}", anchor.identifier, connection.name, connection.identifier, expected_type));
                }
            }
        }

        for &region in areas.regions.keys() {
            if !areas.anchors.iter().any(|anchor| anchor.region() == region) {
                log::warn!("Region {} has no anchors with a matching name.", region);
            }
        }
        for state in &metadata.states {
            if !context.used_states.contains(state) {
                log::trace!("State {} was never used as a requirement.", state);
            }
        }
    }

    Ok(Graph {
        nodes: graph,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_scaling() {
        let context = EmitterContext {
            definitions: &FxHashMap::default(),
            settings: &Settings::default(),
            validate: false,
            node_map: FxHashMap::default(),
            used_states: FxHashSet::default(),
        };

        let requirement = build_boss_requirement(100, &context);
        match requirement {
            Requirement::Boss(health) if health == 100.0 => {},
            _ => panic!(),
        }

        let mut settings = Settings::default();
        settings.world_settings[0].hard = true;
        let context = EmitterContext {
            definitions: &FxHashMap::default(),
            settings: &settings,
            validate: false,
            node_map: FxHashMap::default(),
            used_states: FxHashSet::default(),
        };

        let requirement = build_boss_requirement(100, &context);
        match requirement {
            Requirement::Boss(health) if health == 100.0 * 1.7999999523162841796875 => {},
            _ => panic!(),
        }
    }
}
