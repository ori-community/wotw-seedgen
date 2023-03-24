mod is_met;
mod solutions;
#[cfg(test)]
mod tests;

pub(crate) use solutions::filter_redundancies;

use std::slice;

use smallvec::SmallVec;

use crate::item::{Resource, Shard, Skill, Teleporter};
use crate::settings::{Difficulty, Trick, WorldSettings};
use crate::util::Enemy;

#[derive(Debug, Clone)]
pub enum Requirement {
    Free,
    Impossible,
    Difficulty(Difficulty),
    NormalGameDifficulty,
    Trick(Trick),
    Skill(Skill),
    EnergySkill(Skill, f32),
    NonConsumingEnergySkill(Skill),
    SpiritLight(u32),
    Resource(Resource, u32),
    Shard(Shard),
    Teleporter(Teleporter),
    Water,
    State(usize),
    Damage(f32),
    Danger(f32),
    Combat(SmallVec<[(Enemy, u8); 12]>),
    Boss(f32),
    BreakWall(f32),
    ShurikenBreak(f32),
    SentryBreak(f32),
    And(Vec<Requirement>),
    Or(Vec<Requirement>),
}

impl Requirement {
    /// Checks whether this [`Requirement`] is possible to meet with the given settings
    pub(crate) fn is_possible_for(&self, settings: &WorldSettings) -> bool {
        match self {
            Requirement::Impossible => false,
            Requirement::Difficulty(difficulty) => settings.difficulty >= *difficulty,
            Requirement::NormalGameDifficulty => !settings.hard,
            Requirement::Trick(trick) => settings.tricks.contains(trick),
            Requirement::And(nested) => nested
                .iter()
                .all(|requirement| requirement.is_possible_for(settings)),
            Requirement::Or(nested) => nested
                .iter()
                .any(|requirement| requirement.is_possible_for(settings)),
            _ => true,
        }
    }

    pub(crate) fn contained_requirements<'a, 'b>(
        &'a self,
        settings: &'b WorldSettings,
    ) -> ContainedRequirements<'a, 'b> {
        ContainedRequirements::new(self, settings)
    }
}

pub(crate) struct ContainedRequirements<'a, 'b> {
    nested: Vec<slice::Iter<'a, Requirement>>,
    settings: &'b WorldSettings,
}
impl<'a, 'b> ContainedRequirements<'a, 'b> {
    pub(crate) fn new(
        requirement: &'a Requirement,
        settings: &'b WorldSettings,
    ) -> ContainedRequirements<'a, 'b> {
        ContainedRequirements {
            nested: vec![slice::from_ref(requirement).iter()],
            settings,
        }
    }
}
impl<'a> Iterator for ContainedRequirements<'a, '_> {
    type Item = &'a Requirement;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            let current = self.nested.last_mut()?;
            loop {
                match current.next() {
                    Some(requirement) => {
                        if requirement.is_possible_for(self.settings) {
                            match requirement {
                                Requirement::And(nested) | Requirement::Or(nested) => {
                                    self.nested.push(nested.iter())
                                }
                                _ => return Some(requirement),
                            }
                            continue 'outer;
                        }
                    }
                    None => {
                        self.nested.pop();
                        continue 'outer;
                    }
                }
            }
        }
    }
}
