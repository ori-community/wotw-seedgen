use rustc_hash::{FxHashSet, FxHashMap};

use super::{parser::{self, Areas, AreaContent}, locations::Location, states::NamedState};

use crate::{world::{
    graph::{self, Graph, Node},
    requirements::Requirement,
}, settings::{Difficulty, Trick}, Settings, util::NodeKind};
use crate::item::Skill;

struct EmitterContext<'a> {
    macros: &'a FxHashMap<&'a str, parser::Group<'a>>,
    settings: &'a Settings,
    node_map: FxHashMap<String, usize>,
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

fn build_boss_requirement(health: f32, context: &EmitterContext) -> Requirement {
    if context.settings.any_play_hard() {
        if context.settings.all_play_hard() {
            Requirement::Boss(health * HARD_BOSS_HEALTH_MULTIPLIER)
        } else {
            Requirement::Or(vec![
                Requirement::And(vec![
                    Requirement::NormalGameDifficulty,
                    Requirement::Boss(health),
                ]),
                Requirement::Boss(health * HARD_BOSS_HEALTH_MULTIPLIER),
            ])
        }
    } else {
        Requirement::Boss(health)
    }
}

fn build_requirement<'a>(requirement: &parser::Requirement<'a>, region: bool, context: &mut EmitterContext<'a>) -> Requirement {
    match &requirement.value {
        parser::RequirementValue::Free => Requirement::Free,
        parser::RequirementValue::Impossible => Requirement::Impossible,
        parser::RequirementValue::Macro(identifier) => build_requirement_group(&context.macros[identifier], region, context),
        parser::RequirementValue::Difficulty(difficulty) => build_difficulty_requirement(*difficulty, Requirement::Free, region, context),
        parser::RequirementValue::Trick(glitch) => build_trick_requirement(glitch, Requirement::Free, context),
        parser::RequirementValue::Skill(skill) => Requirement::Skill((*skill).into()),
        parser::RequirementValue::UseSkill(skill, amount) => Requirement::EnergySkill((*skill).into(), *amount as f32),
        parser::RequirementValue::SpiritLight(amount) => Requirement::SpiritLight(*amount),
        parser::RequirementValue::Resource(resource, amount) => Requirement::Resource((*resource).into(), *amount),
        parser::RequirementValue::Shard(shard) => Requirement::Shard((*shard).into()),
        parser::RequirementValue::Teleporter(teleporter) => Requirement::Teleporter((*teleporter).into()),
        parser::RequirementValue::Water => Requirement::Water,
        parser::RequirementValue::State(state) => Requirement::State(context.node_map[*state]),
        parser::RequirementValue::Damage(amount) => Requirement::Damage(*amount as f32),
        parser::RequirementValue::Danger(amount) => Requirement::Danger(*amount as f32),
        parser::RequirementValue::Combat(enemies) => Requirement::Combat(enemies.clone()),
        parser::RequirementValue::Boss(health) => build_boss_requirement(*health as f32, context),
        parser::RequirementValue::BreakWall(health) => Requirement::BreakWall(*health as f32),
        parser::RequirementValue::BreakCrystal =>
            build_or(vec![
                Requirement::Skill(Skill::Sword),
                Requirement::Skill(Skill::Hammer),
                Requirement::EnergySkill(Skill::Bow, 1.0),
                build_difficulty_requirement(Difficulty::Gorlek, Requirement::EnergySkill(Skill::Shuriken, 1.0), false, context),
                build_difficulty_requirement(Difficulty::Gorlek, Requirement::EnergySkill(Skill::Grenade, 1.0), false, context),
                build_difficulty_requirement(Difficulty::Unsafe, Requirement::EnergySkill(Skill::Spear, 1.0), false, context),
            ]),
        parser::RequirementValue::ShurikenBreak(health) => build_trick_requirement(&Trick::ShurikenBreak, Requirement::ShurikenBreak(*health as f32), context),
        parser::RequirementValue::SentryBreak(health) => build_trick_requirement(&Trick::SentryBreak, Requirement::SentryBreak(*health as f32), context),
        parser::RequirementValue::HammerBreak => build_trick_requirement(&Trick::HammerBreak, Requirement::Skill(Skill::Hammer), context),
        parser::RequirementValue::SpearBreak => build_trick_requirement(&Trick::SpearBreak, Requirement::EnergySkill(Skill::Spear, 1.0), context),
        parser::RequirementValue::SentryJump(amount) => 
            build_and(vec![
                Requirement::EnergySkill(Skill::Sentry, *amount as f32),
                build_or(vec![
                    build_trick_requirement(&Trick::SwordSentryJump, Requirement::Skill(Skill::Sword), context),
                    build_trick_requirement(&Trick::HammerSentryJump, Requirement::Skill(Skill::Hammer), context),
                ]),
            ]),
        parser::RequirementValue::SwordSentryJump(amount) => build_trick_requirement(&Trick::SwordSentryJump,
            Requirement::And(vec![
                Requirement::EnergySkill(Skill::Sentry, *amount as f32),
                Requirement::Skill(Skill::Sword),
            ]), context),
        parser::RequirementValue::HammerSentryJump(amount) => build_trick_requirement(&Trick::HammerSentryJump,
            Requirement::And(vec![
                Requirement::EnergySkill(Skill::Sentry, *amount as f32),
                Requirement::Skill(Skill::Hammer),
            ]), context),
        parser::RequirementValue::SentryBurn(amount) => build_trick_requirement(&Trick::SentryBurn, Requirement::EnergySkill(Skill::Sentry, *amount as f32), context),
        parser::RequirementValue::LaunchSwap => build_trick_requirement(&Trick::LaunchSwap, Requirement::Skill(Skill::Launch), context),
        parser::RequirementValue::SentrySwap(amount) => build_trick_requirement(&Trick::SentrySwap, Requirement::EnergySkill(Skill::Sentry, *amount as f32), context),
        parser::RequirementValue::FlashSwap => build_trick_requirement(&Trick::FlashSwap, Requirement::NonConsumingEnergySkill(Skill::Flash), context),
        parser::RequirementValue::BlazeSwap(amount) => build_trick_requirement(&Trick::BlazeSwap, Requirement::EnergySkill(Skill::Blaze, *amount as f32), context),
        parser::RequirementValue::WaveDash => build_trick_requirement(&Trick::WaveDash, Requirement::And(vec![Requirement::Skill(Skill::Dash), Requirement::NonConsumingEnergySkill(Skill::Regenerate)]), context),
        parser::RequirementValue::GrenadeJump => build_trick_requirement(&Trick::GrenadeJump, Requirement::NonConsumingEnergySkill(Skill::Grenade), context),
        parser::RequirementValue::GrenadeCancel => Requirement::NonConsumingEnergySkill(Skill::Grenade),
        parser::RequirementValue::HammerJump => build_trick_requirement(&Trick::HammerJump, Requirement::And(vec![Requirement::Skill(Skill::Hammer), Requirement::Skill(Skill::DoubleJump)]), context),
        parser::RequirementValue::SwordJump => build_trick_requirement(&Trick::SwordJump, Requirement::And(vec![Requirement::Skill(Skill::Sword), Requirement::Skill(Skill::DoubleJump)]), context),
        parser::RequirementValue::GrenadeRedirect(amount) => build_trick_requirement(&Trick::GrenadeRedirect, Requirement::EnergySkill(Skill::Grenade, *amount as f32), context),
        parser::RequirementValue::SentryRedirect(amount) => build_trick_requirement(&Trick::SentryRedirect, Requirement::EnergySkill(Skill::Sentry, *amount as f32), context),
        parser::RequirementValue::GlideJump => build_trick_requirement(&Trick::GlideJump, Requirement::Skill(Skill::Glide), context),
        parser::RequirementValue::GlideHammerJump => build_trick_requirement(&Trick::GlideHammerJump, Requirement::And(vec![Requirement::Skill(Skill::Glide), Requirement::Skill(Skill::Hammer)]), context),
        parser::RequirementValue::SpearJump(amount) => build_trick_requirement(&Trick::SpearJump, Requirement::EnergySkill(Skill::Spear, *amount as f32), context),
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

fn add_entry(node_map: &mut FxHashMap<String, usize>, key: &str, index: usize) -> Result<(), String> {
    match node_map.insert(key.to_string(), index) {
        Some(_) => Err(format!("Name \"{key}\" was used multiple times ambiguously")),
        None => Ok(()),
    }
}

/// Builds the [`Graph`] from parsed data
/// 
/// The given [`Settings`] will be used to optimize the [`Graph`], changing them afterwards may invalidate the result
pub fn build(areas: Areas, locations: Vec<Location>, named_states: Vec<NamedState>, settings: &Settings, validate: bool) -> Result<Graph, String> {
    let mut macros = FxHashMap::default();
    let mut regions = FxHashMap::default();
    regions.reserve(20);
    let mut states = FxHashSet::default();
    states.reserve(named_states.len());
    let mut quests = FxHashSet::default();
    quests.reserve(named_states.len() / 5);
    let mut anchors = Vec::with_capacity(250);
    let node_count = areas.contents.len();
    for content in areas.contents {
        match content {
            AreaContent::Requirement(named_group) => { macros.insert(named_group.name, named_group.group); },
            AreaContent::Region(named_group) => { regions.insert(named_group.name, named_group.group); },
            AreaContent::Anchor(anchor) => {
                for connection in &anchor.connections {
                    match connection.kind {
                        NodeKind::State => { states.insert(connection.identifier); },
                        NodeKind::Quest => { quests.insert(connection.identifier); },
                        _ => {},
                    }
                }
                anchors.push(anchor)
            },
        }
    }

    if validate {
        for &region in regions.keys() {
            if !anchors.iter().any(|anchor| anchor.region() == region) {
                log::warn!("Region {} has no anchors with a matching name.", region);
            }
        }
    }

    let mut index = 0;
    let mut nodes = Vec::with_capacity(node_count);
    let mut node_map = FxHashMap::default();
    node_map.reserve(node_count);

    for location in locations {
        let Location { name, zone, uber_state, position, map_position } = location;
        let identifier = name;
        add_entry(&mut node_map, &identifier, index)?;
        let node = match quests.contains(&identifier[..]) {
            true => Node::Quest(graph::Quest { identifier, zone, index, uber_state, position, map_position }),
            false => Node::Pickup(graph::Pickup { identifier, zone, index, uber_state, position, map_position }),
        };
        nodes.push(node);
        index += 1;
    }
    for state in &states {
        let identifier = state.to_string();
        let uber_state = named_states.iter()
            .find(|named_state| named_state.name == identifier)
            .map(|named_state| &named_state.uber_state)
            .cloned();
        if uber_state.is_none() {
            log::trace!("Couldn't find an entry for {} in the state table", state);
        }
        add_entry(&mut node_map, &identifier, index)?;
        let node = Node::State(graph::State { identifier, index, uber_state });
        nodes.push(node);
        index += 1;
    }

    for (anchor_index, anchor) in anchors.iter().enumerate() {
        add_entry(&mut node_map, &anchor.identifier, index + anchor_index)?;
    }

    let mut context = EmitterContext {
        macros: &macros,
        settings,
        node_map,
    };
    for anchor in anchors {
        let region = regions.get(anchor.region());
        let region_requirement = region.map(|group| build_requirement_group(group, true, &mut context));

        let parser::Anchor { identifier, position, can_spawn, refills, connections } = anchor;
        let identifier = identifier.to_owned();

        let refills = refills.into_iter().map(|refill| {
            let value = refill.value;
            let requirement = refill.requirements.map_or(Requirement::Free, |group| build_requirement_group(&group, false, &mut context));
            graph::Refill { value, requirement }
        }).collect();

        let connections = connections.into_iter().map(|connection| {
            let mut requirement = build_requirement_group(&connection.requirements, false, &mut context);
            if let Some(region_requirement) = &region_requirement {
                requirement = build_and(vec![region_requirement.clone(), requirement]);
            }
            let to = *context.node_map.get(connection.identifier).ok_or_else(|| format!("Anchor {} connects to {} {} which doesn't actually exist", identifier, connection.kind, connection.identifier))?;

            if validate {
                let expected_kind = nodes.get(to).map_or(NodeKind::Anchor, |node| node.node_kind());
                if connection.kind != expected_kind {
                    return Err(format!("Anchor {} connects to {} {} which is actually a {}", identifier, connection.kind, connection.identifier, expected_kind));
                }
            }

            Ok(graph::Connection { to, requirement })
        }).collect::<Result<Vec<_>, String>>()?;

        let node = Node::Anchor(graph::Anchor { identifier, position, can_spawn, index, refills, connections });
        index += 1;
        nodes.push(node);
    };

    if validate {
        check_unused_states(&nodes, &states);
    }

    Ok(Graph { nodes })
}

fn check_unused_states(nodes: &[Node], states: &FxHashSet<&str>) {
    let mut used_states = FxHashSet::default();
    used_states.reserve(states.len());
    nodes.iter()
        .filter_map(|node| if let Node::Anchor(anchor) = node { Some(anchor) } else { None })
        .flat_map(|anchor| &anchor.connections)
        .map(|connection| &nodes[connection.to])
        .filter(|node| node.node_kind() == NodeKind::State)
        .for_each(|node| { used_states.insert(node.identifier()); });

    let unused_states = states.iter().filter(|state| !used_states.contains(*state));
    for state in unused_states {
        log::trace!("State {} was never used as a requirement", state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_scaling() {
        let context = EmitterContext {
            macros: &FxHashMap::default(),
            settings: &Settings::default(),
            node_map: FxHashMap::default(),
        };

        let requirement = build_boss_requirement(100.0, &context);
        match requirement {
            Requirement::Boss(health) if health == 100.0 => {},
            _ => panic!(),
        }

        let mut settings = Settings::default();
        settings.world_settings[0].hard = true;
        let context = EmitterContext {
            macros: &FxHashMap::default(),
            settings: &settings,
            node_map: FxHashMap::default(),
        };

        let requirement = build_boss_requirement(100.0, &context);
        match requirement {
            Requirement::Boss(health) if health == 100.0 * 1.7999999523162841796875 => {},
            _ => panic!(),
        }
    }
}
