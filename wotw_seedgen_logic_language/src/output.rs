use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use strum::EnumString;
use wotw_seedgen_assets::{LocDataEntry, StateDataEntry};
use wotw_seedgen_data::{Position, Shard, Skill, Teleporter, UberIdentifier, Zone};
use wotw_seedgen_settings::{Difficulty, Trick};

pub type DoorId = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub default_door_connections: FxHashMap<DoorId, DoorId>,
}
impl Graph {
    // TODO use more in tests
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            default_door_connections: FxHashMap::default(),
        }
    }

    // TODO could optimize based on the node kind we're looking for? The tag should be faster to compare than our long strings
    pub fn find_node(&self, node: &str) -> Result<usize, String> {
        self.nodes
            .iter()
            .position(|n| n.identifier() == node)
            .ok_or_else(|| format!("node \"{node}\" not found"))
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Anchor(Anchor),
    Pickup(LocDataEntry),
    State(StateDataEntry),
    LogicalState(String),
}
impl Node {
    pub fn identifier(&self) -> &str {
        match self {
            Node::Anchor(anchor) => &anchor.identifier,
            Node::Pickup(pickup) => &pickup.identifier,
            Node::State(state) => &state.identifier,
            Node::LogicalState(identifier) => identifier,
        }
    }
    pub fn zone(&self) -> Option<Zone> {
        match self {
            Node::Pickup(pickup) => Some(pickup.zone),
            _ => None,
        }
    }
    // TODO check which of these ended up being used
    pub fn uber_identifier(&self) -> Option<UberIdentifier> {
        match self {
            Node::Anchor(_) | Node::LogicalState(_) => None,
            Node::Pickup(pickup) => Some(pickup.uber_identifier),
            Node::State(state) => Some(state.uber_identifier),
        }
    }
    pub fn value(&self) -> Option<i32> {
        match self {
            Node::Anchor(_) | Node::LogicalState(_) => None,
            Node::Pickup(pickup) => pickup.value,
            Node::State(state) => state.value,
        }
    }
    pub fn position(&self) -> Option<&Position> {
        match self {
            Node::Anchor(anchor) => anchor.position.as_ref(),
            Node::Pickup(pickup) => pickup.position.as_ref(),
            Node::State(_) | Node::LogicalState(_) => None,
        }
    }
    pub fn map_position(&self) -> Option<&Position> {
        match self {
            Node::Anchor(anchor) => anchor.position.as_ref(),
            Node::Pickup(pickup) => pickup.map_position.as_ref(),
            Node::State(_) | Node::LogicalState(_) => None,
        }
    }
    pub fn can_place(&self) -> bool {
        matches!(self, Node::Pickup(_))
    }
    pub fn can_spawn(&self) -> bool {
        match self {
            Node::Anchor(anchor) => anchor.position.is_some() && anchor.can_spawn,
            _ => false,
        }
    }
    pub fn get_anchor(&self) -> Option<&Anchor> {
        match self {
            Node::Anchor(anchor) => Some(anchor),
            _ => None,
        }
    }
    pub fn get_anchor_mut(&mut self) -> Option<&mut Anchor> {
        match self {
            Node::Anchor(anchor) => Some(anchor),
            _ => None,
        }
    }
    pub fn expect_anchor(&self) -> &Anchor {
        match self {
            Node::Anchor(anchor) => anchor,
            _ => panic!("Called expect_anchor on {self:?}"),
        }
    }
    pub fn expect_anchor_mut(&mut self) -> &mut Anchor {
        match self {
            Node::Anchor(anchor) => anchor,
            _ => panic!("Called expect_anchor_mut on {self:?}"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Anchor {
    pub identifier: String,
    pub position: Option<Position>,
    pub door: Option<Door>,
    pub can_spawn: bool,
    pub teleport_restriction: Requirement,
    pub refills: Vec<Refill>,
    pub connections: Vec<Connection>,
}
impl PartialEq for Anchor {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Door {
    pub id: DoorId,
    pub target: String,
    pub requirement: Requirement,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    pub to: usize,
    pub requirement: Requirement,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Refill {
    pub value: RefillValue,
    pub requirement: Requirement,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefillValue {
    Full,
    Checkpoint,
    Health(f32),
    Energy(f32),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Requirement {
    Free,
    Impossible,
    Difficulty(Difficulty),
    NormalGameDifficulty,
    Trick(Trick),
    Skill(Skill),
    EnergySkill(Skill, f32),
    NonConsumingEnergySkill(Skill),
    // TODO resources as i32?
    SpiritLight(usize),
    GorlekOre(usize),
    Keystone(usize),
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumString)]
pub enum Enemy {
    Mantis,
    Slug,
    WeakSlug,
    BombSlug,
    CorruptSlug,
    SneezeSlug,
    ShieldSlug,
    Lizard,
    Bat,
    Hornbug,
    Skeeto,
    SmallSkeeto,
    Bee,
    Nest,
    Crab,
    SpinCrab,
    Tentacle,
    Balloon,
    Miner,
    MaceMiner,
    ShieldMiner,
    CrystalMiner,
    ShieldCrystalMiner,
    Sandworm,
    Spiderling,
    EnergyRefill,
}
impl Enemy {
    pub fn health(self) -> f32 {
        match self {
            Enemy::BombSlug | Enemy::CorruptSlug | Enemy::Balloon => 1.0,
            Enemy::SmallSkeeto => 8.0,
            Enemy::WeakSlug | Enemy::Spiderling => 12.0,
            Enemy::Slug => 13.0,
            Enemy::Skeeto | Enemy::Sandworm | Enemy::Tentacle => 20.0,
            Enemy::ShieldSlug | Enemy::Lizard | Enemy::Bee => 24.0,
            Enemy::Nest => 25.0,
            Enemy::Mantis | Enemy::SneezeSlug | Enemy::Bat | Enemy::Crab | Enemy::SpinCrab => 32.0,
            Enemy::Hornbug | Enemy::Miner => 40.0,
            Enemy::ShieldCrystalMiner => 50.0,
            Enemy::MaceMiner | Enemy::ShieldMiner => 60.0,
            Enemy::CrystalMiner => 80.0,
            Enemy::EnergyRefill => 0.0,
        }
    }
    pub fn shielded(self) -> bool {
        matches!(
            self,
            Enemy::Hornbug | Enemy::ShieldSlug | Enemy::ShieldMiner | Enemy::ShieldCrystalMiner
        )
    }
    pub fn armored(self) -> bool {
        matches!(self, Enemy::Tentacle)
    }
    pub fn aerial(self) -> bool {
        // whether we consider the enemy flying for movement restriction purposes
        matches!(
            self,
            Enemy::Bat
                | Enemy::Skeeto
                | Enemy::SmallSkeeto
                | Enemy::Bee
                | Enemy::Nest
                | Enemy::Tentacle
        )
    }
    pub fn flying(self) -> bool {
        // whether the game considers the enemy flying for wingclip
        matches!(self, Enemy::Skeeto | Enemy::SmallSkeeto | Enemy::Bee)
    }
    pub fn ranged(self) -> bool {
        // whether you need a ranged weapon
        matches!(
            self,
            Enemy::BombSlug | Enemy::CorruptSlug | Enemy::Balloon | Enemy::Bat
        )
    }
    pub fn dangerous(self) -> bool {
        matches!(
            self,
            Enemy::SneezeSlug
                | Enemy::Hornbug
                | Enemy::Crab
                | Enemy::SpinCrab
                | Enemy::Miner
                | Enemy::MaceMiner
                | Enemy::ShieldMiner
                | Enemy::CrystalMiner
                | Enemy::ShieldCrystalMiner
        )
    }
}
