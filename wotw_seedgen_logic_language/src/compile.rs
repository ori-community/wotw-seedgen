use std::{borrow::Cow, iter, mem, ops::Range, str::FromStr};

use crate::{
    ast,
    output::{
        Anchor, Connection, Door, DoorId, Enemy, Graph, Node, Refill, RefillValue, Requirement,
    },
    token::Tokenizer,
};
use rustc_hash::FxHashMap;
use wotw_seedgen_assets::{LocData, StateData, StateDataEntry};
use wotw_seedgen_data::{Position, Shard, Skill, Teleporter, UberIdentifier};
use wotw_seedgen_parse::{
    Error, ErrorKind, ParseResult, Recover, Recoverable, Result, Separated, SeparatedNonEmpty,
    Span, Spanned, SpannedOption,
};
use wotw_seedgen_settings::{Difficulty, Trick, WorldSettings, WorldSettingsHelpers};

// TODO not really part of compilation but some kind of lints would be nice, like:
// verify all states were used
// verify nothing was used mixed as either quest or pickup
// verify all anchors are connected
// check for implicit moki

impl Graph {
    /// `settings` is used for optimization, you may pass an empty slice if you don't know the settings this `Graph` will be used for
    pub fn compile(
        mut areas: ast::Areas,
        loc_data: LocData,
        state_data: StateData,
        settings: &[WorldSettings],
    ) -> ParseResult<Graph> {
        let loc_data_nodes = loc_data
            .entries
            .into_iter()
            .map(Node::Pickup)
            .collect::<Vec<_>>();

        let state_data_nodes = state_data
            .entries
            .into_iter()
            .map(Node::State)
            .collect::<Vec<_>>();

        let mut compiler = Compiler::new(&mut areas, &loc_data_nodes, &state_data_nodes, settings);

        areas.contents.compile(&mut compiler);
        compiler.generate_door_connections();

        let Compiler {
            nodes: compiled_nodes,
            default_door_connections,
            errors,
            ..
        } = compiler;

        let mut nodes: Vec<Node> = loc_data_nodes;
        nodes.reserve_exact(state_data_nodes.len() + compiled_nodes.len());
        nodes.extend(state_data_nodes);
        nodes.extend(compiled_nodes);

        ParseResult {
            parsed: Some(Graph {
                nodes,
                default_door_connections,
            }),
            errors,
        }
    }
}

struct Compiler<'source> {
    nodes: Vec<Node>,
    index_offset: usize,
    door_nodes: Vec<usize>,
    default_door_connections: FxHashMap<DoorId, DoorId>,
    difficulty_requirements: DifficultyRequirements,
    trick_requirements: TrickRequirements,
    state_map: FxHashMap<Cow<'source, str>, usize>,
    pickup_map: FxHashMap<&'source str, usize>,
    anchor_map: FxHashMap<String, usize>,
    macros: FxHashMap<String, Requirement>,
    // TODO apply region requirements
    regions: FxHashMap<String, Requirement>,
    errors: Vec<Error>,
}

impl<'source> Compiler<'source> {
    fn new(
        areas: &mut ast::Areas,
        loc_data_nodes: &'source [Node],
        state_data_nodes: &'source [Node],
        settings: &[WorldSettings],
    ) -> Self {
        let mut errors = vec![];
        let mut nodes = vec![]; // TODO capacity
        let mut state_map = FxHashMap::default();
        let mut pickup_map = FxHashMap::default();
        let mut anchor_map = FxHashMap::default();
        let mut index_iter = 0..;

        // index_iter has to be the second iterator so it doesn't get incremented when the first one runs out
        for (node, index) in iter::zip(loc_data_nodes, &mut index_iter) {
            let identifier = node.identifier();
            if pickup_map.insert(identifier, index).is_some() {
                errors.push(Error::custom(
                    format!("duplicate identifier \"{identifier}\" in loc_data"),
                    0..0,
                ));
            }
        }
        for (node, index) in iter::zip(state_data_nodes, &mut index_iter) {
            let identifier = node.identifier();
            if state_map.insert(Cow::Borrowed(identifier), index).is_some() {
                errors.push(Error::custom(
                    format!("duplicate identifier \"{identifier}\" in state_data"),
                    0..0,
                ));
            }
        }

        // partition
        {
            let mut iter = areas.contents.iter_mut();

            while let Some(anchor) = iter.find(|content| {
                matches!(content.value, SpannedOption::Some(ast::Content::Anchor(..)))
            }) {
                match iter.rfind(|content| {
                    matches!(
                        content.value,
                        SpannedOption::Some(
                            ast::Content::Requirement(..) | ast::Content::Region(..)
                        )
                    )
                }) {
                    None => break,
                    Some(non_anchor) => mem::swap(anchor, non_anchor),
                }
            }
        }

        let mut anchors = Vec::with_capacity(300); // We know more about the needed capacity than collect would

        for anchor in areas
            .contents
            .iter()
            .filter_map(|content| content.value.as_option())
            .filter_map(|content| {
                if let ast::Content::Anchor(_, anchor) = content {
                    Some(anchor)
                } else {
                    None
                }
            })
            .filter_map(|anchor| anchor.value.as_option())
        {
            for connection in anchor
                .content
                .content
                .value
                .as_option()
                .into_iter()
                .flat_map(|group| &group.content)
                .filter_map(|content| match content {
                    ast::AnchorContent::Connection(
                        Spanned {
                            data: ast::ConnectionKeyword::State,
                            ..
                        },
                        connection,
                    ) => connection.value.as_option(),
                    _ => None,
                })
            {
                if !state_map.contains_key(connection.identifier.data.0) {
                    nodes.push(Node::LogicalState(connection.identifier.data.0.to_string()));
                    state_map.insert(
                        Cow::Owned(connection.identifier.data.0.to_string()),
                        index_iter.next().unwrap(),
                    );
                }
            }
            anchors.push(anchor);
        }

        for (index, anchor) in index_iter.zip(anchors) {
            if anchor_map.contains_key(anchor.identifier.data.0) {
                errors.push(Error::custom(
                    format!("duplicate identifier \"{}\"", anchor.identifier.data.0),
                    anchor.identifier.span.clone(),
                ));
            } else {
                anchor_map.insert(anchor.identifier.data.0.to_string(), index);
            }
        }

        Self {
            nodes,
            index_offset: loc_data_nodes.len() + state_data_nodes.len(),
            door_nodes: vec![],
            default_door_connections: FxHashMap::default(),
            difficulty_requirements: DifficultyRequirements::new(settings),
            trick_requirements: TrickRequirements::new(settings),
            state_map,
            pickup_map,
            anchor_map,
            macros: Default::default(),
            regions: Default::default(),
            errors,
        }
    }

    fn error(&mut self, message: String, span: Range<usize>) {
        self.errors.push(Error::custom(message, span));
    }

    fn consume_result<T>(&mut self, result: Result<T>) -> Option<T> {
        result.map_err(|err| self.errors.push(err)).ok()
    }

    fn generate_door_connections(&mut self) {
        for index in self.door_nodes.iter().copied() {
            let anchor = self.nodes[index].expect_anchor();
            let anchor_identifier = anchor.identifier.clone();
            let door = anchor.door.as_ref().unwrap();
            let door_id = door.id;
            let door_requirement = door.requirement.clone();

            if let Some(target) = self.anchor_map.get(&door.target) {
                let target_door_id = self.nodes[*target - self.index_offset]
                    .expect_anchor()
                    .door
                    .as_ref()
                    .unwrap()
                    .id;

                self.default_door_connections
                    .insert(door_id, target_door_id);
            }

            let visited_state_index = self.nodes.len();
            // TODO during seedgen this should be 1, but start as 0 in the true seed
            self.nodes.push(Node::State(StateDataEntry {
                identifier: format!("{}Visited", anchor_identifier),
                uber_identifier: UberIdentifier::new(28, door_id),
                value: None,
            }));

            for target_index in self.door_nodes.iter().copied() {
                if index == target_index {
                    continue;
                }

                let state_index = self.nodes.len();
                self.nodes[index]
                    .expect_anchor_mut()
                    .connections
                    .push(Connection {
                        to: self.index_offset + target_index,
                        requirement: Requirement::And(vec![
                            Requirement::State(self.index_offset + state_index),
                            Requirement::State(self.index_offset + visited_state_index),
                            door_requirement.clone(),
                        ]),
                    });

                let target_anchor = self.nodes[target_index].expect_anchor(); // TODO verify while parsing

                self.nodes.push(Node::State(StateDataEntry {
                    identifier: format!("{} to {}", anchor_identifier, target_anchor.identifier),
                    uber_identifier: UberIdentifier::new(27, door_id),
                    value: Some(target_anchor.door.as_ref().unwrap().id), // TODO this should not be treated as strictly incremental
                }));
            }
        }
    }
}

struct DifficultyRequirements {
    moki: Requirement,
    gorlek: Requirement,
    kii: Requirement,
    notsafe: Requirement,
}

impl DifficultyRequirements {
    // TODO could propagate this all the way up to use WorldSettingsHelpers and improve the interface
    fn new(settings: &[WorldSettings]) -> Self {
        let lowest_difficulty = settings.lowest_difficulty();
        let highest_difficulty = settings.highest_difficulty();

        let build_difficulty = move |difficulty| {
            if highest_difficulty < difficulty {
                Requirement::Impossible
            } else if lowest_difficulty >= difficulty {
                Requirement::Free
            } else {
                Requirement::Difficulty(difficulty)
            }
        };

        Self {
            moki: build_difficulty(Difficulty::Moki),
            gorlek: build_difficulty(Difficulty::Gorlek),
            kii: build_difficulty(Difficulty::Kii),
            notsafe: build_difficulty(Difficulty::Unsafe),
        }
    }

    fn get(&self, difficulty: Difficulty) -> Requirement {
        match difficulty {
            Difficulty::Moki => self.moki.clone(),
            Difficulty::Gorlek => self.gorlek.clone(),
            Difficulty::Kii => self.kii.clone(),
            Difficulty::Unsafe => self.notsafe.clone(),
        }
    }
}

// could reduce the memory footprint by using a smaller type than Requirement, but there's just one of these around oriShrug
struct TrickRequirements {
    sword_sentry_jump: Requirement,
    hammer_sentry_jump: Requirement,
    shuriken_break: Requirement,
    sentry_break: Requirement,
    hammer_break: Requirement,
    spear_break: Requirement,
    sentry_burn: Requirement,
    remove_kill_plane: Requirement,
    launch_swap: Requirement,
    sentry_swap: Requirement,
    flash_swap: Requirement,
    blaze_swap: Requirement,
    wave_dash: Requirement,
    grenade_jump: Requirement,
    sword_jump: Requirement,
    hammer_jump: Requirement,
    glide_jump: Requirement,
    glide_hammer_jump: Requirement,
    coyote_hammer_jump: Requirement,
    wall_hammer_jump: Requirement,
    grounded_hammer_jump: Requirement,
    extended_hammer: Requirement,
    grenade_redirect: Requirement,
    sentry_redirect: Requirement,
    pause_hover: Requirement,
    spear_jump: Requirement,
}

impl TrickRequirements {
    fn new(settings: &[WorldSettings]) -> Self {
        let build_trick = move |trick| {
            if settings.is_empty() {
                Requirement::Trick(trick)
            } else if settings.none_contain_trick(trick) {
                Requirement::Impossible
            } else if settings.all_contain_trick(trick) {
                Requirement::Free
            } else {
                Requirement::Trick(trick)
            }
        };

        Self {
            sword_sentry_jump: build_trick(Trick::SwordSentryJump),
            hammer_sentry_jump: build_trick(Trick::HammerSentryJump),
            shuriken_break: build_trick(Trick::ShurikenBreak),
            sentry_break: build_trick(Trick::SentryBreak),
            hammer_break: build_trick(Trick::HammerBreak),
            spear_break: build_trick(Trick::SpearBreak),
            sentry_burn: build_trick(Trick::SentryBurn),
            remove_kill_plane: build_trick(Trick::RemoveKillPlane),
            launch_swap: build_trick(Trick::LaunchSwap),
            sentry_swap: build_trick(Trick::SentrySwap),
            flash_swap: build_trick(Trick::FlashSwap),
            blaze_swap: build_trick(Trick::BlazeSwap),
            wave_dash: build_trick(Trick::WaveDash),
            grenade_jump: build_trick(Trick::GrenadeJump),
            sword_jump: build_trick(Trick::SwordJump),
            hammer_jump: build_trick(Trick::HammerJump),
            glide_jump: build_trick(Trick::GlideJump),
            glide_hammer_jump: build_trick(Trick::GlideHammerJump),
            coyote_hammer_jump: build_trick(Trick::CoyoteHammerJump),
            wall_hammer_jump: build_trick(Trick::WallHammerJump),
            grounded_hammer_jump: build_trick(Trick::GroundedHammerJump),
            extended_hammer: build_trick(Trick::ExtendedHammer),
            grenade_redirect: build_trick(Trick::GrenadeRedirect),
            sentry_redirect: build_trick(Trick::SentryRedirect),
            pause_hover: build_trick(Trick::PauseHover),
            spear_jump: build_trick(Trick::SpearJump),
        }
    }

    fn get(&self, trick: Trick, amount: &mut Option<usize>) -> Option<Requirement> {
        let requirement = match trick {
            Trick::SwordSentryJump => build_and([
                self.sword_sentry_jump.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
                Requirement::Skill(Skill::Sword),
            ]),
            Trick::HammerSentryJump => build_and([
                self.hammer_sentry_jump.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::ShurikenBreak => build_and([
                self.shuriken_break.clone(),
                Requirement::ShurikenBreak(amount.take()? as f32),
            ]),
            Trick::SentryBreak => build_and([
                self.sentry_break.clone(),
                Requirement::SentryBreak(amount.take()? as f32),
            ]),
            Trick::HammerBreak => {
                build_and([self.hammer_break.clone(), Requirement::Skill(Skill::Hammer)])
            }
            Trick::SpearBreak => build_and([
                self.spear_break.clone(),
                Requirement::EnergySkill(Skill::Spear, 1.),
            ]),
            Trick::SentryBurn => build_and([
                self.sentry_burn.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
            ]),
            Trick::RemoveKillPlane => self.remove_kill_plane.clone(),
            Trick::LaunchSwap => {
                build_and([self.launch_swap.clone(), Requirement::Skill(Skill::Launch)])
            }
            Trick::SentrySwap => build_and([
                self.sentry_swap.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
            ]),
            Trick::FlashSwap => build_and([
                self.flash_swap.clone(),
                Requirement::NonConsumingEnergySkill(Skill::Flash),
            ]),
            Trick::BlazeSwap => build_and([
                self.blaze_swap.clone(),
                Requirement::EnergySkill(Skill::Blaze, amount.take()? as f32),
            ]),
            Trick::WaveDash => build_and([
                self.wave_dash.clone(),
                Requirement::Skill(Skill::Dash),
                Requirement::NonConsumingEnergySkill(Skill::Regenerate),
            ]),
            Trick::GrenadeJump => build_and([
                self.grenade_jump.clone(),
                Requirement::NonConsumingEnergySkill(Skill::Grenade),
            ]),
            Trick::SwordJump => build_and([
                self.sword_jump.clone(),
                Requirement::Skill(Skill::Sword),
                Requirement::Skill(Skill::DoubleJump),
            ]),
            Trick::HammerJump => {
                build_and([self.hammer_jump.clone(), Requirement::Skill(Skill::Hammer)])
            }
            Trick::AerialHammerJump => build_and([
                self.hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
                Requirement::Skill(Skill::DoubleJump),
            ]),
            Trick::GlideJump => {
                build_and([self.glide_jump.clone(), Requirement::Skill(Skill::Glide)])
            }
            Trick::GlideHammerJump => build_and([
                self.glide_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
                Requirement::Skill(Skill::Glide),
            ]),
            Trick::CoyoteHammerJump => build_and([
                self.coyote_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::WallHammerJump => build_and([
                self.wall_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::GroundedHammerJump => build_and([
                self.grounded_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::ExtendedHammer => build_and([
                self.extended_hammer.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::GrenadeRedirect => build_and([
                self.grenade_redirect.clone(),
                Requirement::EnergySkill(Skill::Grenade, amount.take()? as f32),
            ]),
            Trick::SentryRedirect => build_and([
                self.sentry_redirect.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
            ]),
            Trick::PauseHover => self.pause_hover.clone(),
            Trick::SpearJump => build_and([
                self.spear_jump.clone(),
                Requirement::EnergySkill(Skill::Spear, amount.take()? as f32),
            ]),
        };

        Some(requirement)
    }
}

trait Compile {
    type Output;

    fn compile(self, compiler: &mut Compiler) -> Self::Output;
}

impl<T: Compile> Compile for Vec<T> {
    type Output = Vec<T::Output>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}

impl<'source, T: Compile, R: Recover<'source, Tokenizer>> Compile for Recoverable<T, R> {
    type Output = Option<T::Output>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.value.into_option().map(|t| t.compile(compiler))
    }
}

impl<T: Compile> Compile for Spanned<T> {
    type Output = T::Output;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.data.compile(compiler)
    }
}

impl<T: Compile, Separator> Compile for Separated<T, Separator> {
    // TODO is there a merit to returning an iterator here?
    type Output = Vec<T::Output>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}

impl<T: Compile, Separator> Compile for SeparatedNonEmpty<T, Separator> {
    type Output = Vec<T::Output>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}

impl<T: Compile> Compile for ast::Group<T> {
    type Output = Option<T::Output>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.content.compile(compiler)
    }
}

impl<T: Compile> Compile for ast::GroupContent<T> {
    type Output = T::Output;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.content.compile(compiler)
    }
}

impl<'source> Compile for ast::Content<'source> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        match self {
            ast::Content::Requirement(_, content) => content.compile(compiler),
            ast::Content::Region(_, content) => content.compile(compiler),
            ast::Content::Anchor(_, content) => content.compile(compiler),
        };
    }
}

impl<'source> Compile for ast::Macro<'source> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        if compiler.macros.contains_key(self.identifier.data.0) {
            compiler.error("duplicate identifier".to_string(), self.identifier.span);
        } else {
            let requirement = self
                .requirements
                .compile(compiler)
                .map_or(Requirement::Impossible, build_or);

            compiler
                .macros
                .insert(self.identifier.data.0.to_string(), requirement);
        }
    }
}

impl<'source> Compile for ast::Region<'source> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        if compiler.regions.contains_key(self.identifier.data.0) {
            compiler.error("duplicate identifier".to_string(), self.identifier.span);
        } else {
            let requirement = self
                .requirements
                .compile(compiler)
                .map_or(Requirement::Impossible, build_or);

            compiler
                .regions
                .insert(self.identifier.data.0.to_string(), requirement);
        }
    }
}

impl<'source> Compile for ast::Anchor<'source> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let position = self
            .position
            .and_then(|position| position.compile(compiler));

        let mut door = None;
        let mut can_spawn = true;
        let mut teleport_restriction = None;
        let mut refills = vec![];
        let mut connections = vec![];

        if let SpannedOption::Some(content) = self.content.content.value {
            for content in content.content {
                match content {
                    ast::AnchorContent::Door(keyword, anchor_door) => {
                        if let Some(anchor_door) = anchor_door.compile(compiler) {
                            compiler.door_nodes.push(compiler.nodes.len());

                            if door.replace(anchor_door).is_some() {
                                compiler.error("duplicate door".to_string(), keyword.span);
                            }
                        }
                    }
                    ast::AnchorContent::NoSpawn(keyword) => {
                        if !mem::take(&mut can_spawn) {
                            compiler.error("duplicate nospawn".to_string(), keyword.span);
                        }
                    }
                    ast::AnchorContent::TpRestriction(keyword, requirements) => {
                        let requirement = requirements
                            .data
                            .map_or(Requirement::Impossible, |group| group.compile(compiler));

                        if teleport_restriction.replace(requirement).is_some() {
                            compiler.error("duplicate tprestriction".to_string(), keyword.span);
                        }
                    }
                    ast::AnchorContent::Refill(_, refill) => {
                        refills.extend(refill.compile(compiler))
                    }
                    ast::AnchorContent::Connection(keyword, connection) => {
                        if let SpannedOption::Some(connection) = connection.value {
                            let to = match keyword.data {
                                ast::ConnectionKeyword::State => compiler
                                    .state_map
                                    .get(&Cow::Borrowed(connection.identifier.data.0)),
                                ast::ConnectionKeyword::Pickup | ast::ConnectionKeyword::Quest => {
                                    compiler.pickup_map.get(connection.identifier.data.0)
                                }
                                ast::ConnectionKeyword::Anchor => {
                                    compiler.anchor_map.get(connection.identifier.data.0)
                                }
                            };

                            match to {
                                None => {
                                    compiler.error(
                                        "unknown identifier".to_string(),
                                        connection.identifier.span,
                                    );
                                }
                                Some(&to) => {
                                    let mut requirement = connection.requirements.compile(compiler);

                                    if let Some(region_requirement) = self
                                        .identifier
                                        .data
                                        .region()
                                        .and_then(|region| compiler.regions.get(region))
                                    {
                                        requirement =
                                            build_and([requirement, region_requirement.clone()])
                                    }

                                    connections.push(Connection { to, requirement })
                                }
                            }
                        }
                    }
                }
            }
        }

        compiler.nodes.push(Node::Anchor(Anchor {
            identifier: self.identifier.data.0.to_string(),
            position,
            door,
            can_spawn,
            teleport_restriction: teleport_restriction.unwrap_or(Requirement::Free),
            refills,
            connections,
        }));
    }
}

impl Compile for ast::AnchorPosition {
    type Output = Option<Position>;

    fn compile(self, _compiler: &mut Compiler) -> Self::Output {
        self.position.value.into_option().map(|position| Position {
            x: position.x.data,
            y: position.y.data,
        })
    }
}

impl<'source> Compile for ast::Door<'source> {
    type Output = Option<Door>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let mut id = None;
        let mut target = None;
        let mut requirement = None;

        if let SpannedOption::Some(content) = self.content.value {
            for content in content.content {
                match content {
                    ast::DoorContent::Id(keyword, door_id) => {
                        if let SpannedOption::Some(door_id) = door_id.value {
                            if id.replace(door_id.id.data).is_some() {
                                compiler.error("duplicate id".to_string(), keyword.span);
                            }
                        }
                    }
                    ast::DoorContent::Target(keyword, door_target) => {
                        if let SpannedOption::Some(door_target) = door_target.value {
                            if !compiler.anchor_map.contains_key(door_target.target.data.0) {
                                compiler
                                    .error("unknown anchor".to_string(), door_target.target.span);
                            }

                            if target
                                .replace(door_target.target.data.0.to_string())
                                .is_some()
                            {
                                compiler.error("duplicate target".to_string(), keyword.span);
                            }
                        }
                    }
                    ast::DoorContent::Enter(keyword, door_requirement) => {
                        if let Some(door_requirement) = door_requirement.compile(compiler) {
                            if requirement.replace(door_requirement).is_some() {
                                compiler.error("duplicate enter".to_string(), keyword.span);
                            }
                        }
                    }
                }
            }
        }

        if let (Some(id), Some(target), Some(requirement)) = (id, target, requirement) {
            Some(Door {
                id,
                target,
                requirement,
            })
        } else {
            None
        }
    }
}

impl<'source> Compile for ast::Refill<'source> {
    type Output = Refill;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        Refill {
            value: self.value.compile(compiler),
            requirement: self
                .requirements
                .map_or(Requirement::Free, |group| group.compile(compiler)),
        }
    }
}

impl Compile for ast::RefillValue {
    type Output = RefillValue;

    fn compile(self, _compiler: &mut Compiler) -> Self::Output {
        match self {
            ast::RefillValue::Full => RefillValue::Full,
            ast::RefillValue::Checkpoint => RefillValue::Checkpoint,
            ast::RefillValue::Health(amount) => RefillValue::Health(
                amount
                    .amount
                    .and_then(|amount| amount.value.value.into_option())
                    .map_or(1, |amount| amount.data) as f32,
            ),
            ast::RefillValue::Energy(amount) => RefillValue::Energy(
                amount
                    .amount
                    .and_then(|amount| amount.value.value.into_option())
                    .map_or(1, |amount| amount.data) as f32,
            ),
        }
    }
}

impl<'source> Compile for ast::RequirementLineOrGroup<'source> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.requirement
            .compile(compiler)
            .unwrap_or(Requirement::Impossible)
    }
}

impl<'source> Compile for ast::InlineRequirementOrGroup<'source> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        match self {
            ast::InlineRequirementOrGroup::Inline(_) => Requirement::Free,
            ast::InlineRequirementOrGroup::Group(group) => build_or(group.compile(compiler)),
        }
    }
}

impl<'source> Compile for ast::RequirementLine<'source> {
    type Output = Requirement;

    fn compile(mut self, compiler: &mut Compiler) -> Self::Output {
        // iter::chain doesn't consider it valid to mutably borrow the compiler in both parts
        // a vec would lose out on the short-circuiting behaviour of build_and
        // so we build a custom iterator
        let mut ands = self.ands.into_iter();
        let mut ors = Some(self.ors);

        build_and(iter::from_fn(|| {
            ands.next()
                .map(|(and, _)| and.compile(compiler))
                .or_else(|| ors.take().map(|ors| build_or(ors.compile(compiler))))
                .or_else(|| {
                    self.group.take().map(|group| {
                        group
                            .compile(compiler)
                            .map_or(Requirement::Impossible, build_or)
                    })
                })
        }))
    }
}

impl<'source> Compile for ast::Requirement<'source> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        match self {
            ast::Requirement::Combat(requirement) => requirement.compile(compiler),
            ast::Requirement::State(identifier) => {
                compile_state(compiler, identifier.data.0, identifier.span)
            }
            ast::Requirement::Plain(requirement) => requirement.compile(compiler),
        }
    }
}

fn compile_state(compiler: &mut Compiler, identifier: &str, span: Range<usize>) -> Requirement {
    match compiler
        .state_map
        .get(identifier)
        .or_else(|| compiler.pickup_map.get(identifier))
    {
        None => {
            compiler.error(format!("Unknown requirement \"{}\"", identifier), span);
            Requirement::Impossible
        }
        Some(index) => Requirement::State(*index),
    }
}

impl<'source> Compile for ast::CombatRequirement<'source> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.enemies
            .compile(compiler)
            .and_then(|enemies| enemies.into_iter().collect())
            .map_or(Requirement::Impossible, Requirement::Combat)
    }
}

impl<'source> Compile for ast::Enemy<'source> {
    type Output = Option<(Enemy, u8)>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        compiler
            .consume_result(
                self.identifier
                    .data
                    .0
                    .parse()
                    .map_err(|_| Error::custom("unknown enemy".to_string(), self.identifier.span)),
            )
            .map(|enemy| {
                let amount = self.amount.map_or(1, |amount| amount.value.data);
                (enemy, amount)
            })
    }
}

impl<'source> Compile for ast::PlainRequirement<'source> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let identifier = self.identifier.data.0;
        let amount_span = self.amount.as_ref().map_or(
            self.identifier.span.end..self.identifier.span.end,
            |amount| amount.span(),
        );

        let mut amount = self
            .amount
            .and_then(|amount| amount.value.value.into_option().map(|amount| amount.data));

        let get_amount = || {
            amount.ok_or_else(|| {
                Error::new(
                    ErrorKind::ExpectedToken("'='".to_string()),
                    self.identifier.span.end..self.identifier.span.end,
                )
            })
        };

        let no_amount = || {
            if amount.is_none() {
                Ok(())
            } else {
                Err(Error::custom(
                    "this requirement accepts no amount".to_string(),
                    amount_span.clone(),
                ))
            }
        };

        let result = if let Some(requirement) = compiler.macros.get(identifier) {
            Ok(requirement.clone())
        } else if let Ok(skill) = Skill::from_str(identifier) {
            match amount {
                None => Ok(Requirement::Skill(skill)),
                Some(amount) => {
                    if skill.energy_cost() == 0. {
                        Err(Error::custom(
                            "this skill accepts no amount".to_string(),
                            amount_span,
                        ))
                    } else {
                        Ok(Requirement::EnergySkill(skill, amount as f32))
                    }
                }
            }
        } else if let Ok(difficulty) = Difficulty::from_str(identifier) {
            no_amount().map(|()| compiler.difficulty_requirements.get(difficulty))
        } else if let Ok(trick) = Trick::from_str(identifier) {
            let option = compiler.trick_requirements.get(trick, &mut amount);

            if amount.is_some() {
                Err(Error::custom(
                    "requirement accepts no amount".to_string(),
                    amount_span,
                ))
            } else {
                option.ok_or_else(|| {
                    Error::new(
                        ErrorKind::ExpectedToken("'='".to_string()),
                        self.identifier.span.end..self.identifier.span.end,
                    )
                })
            }
        } else if let Ok(shard) = Shard::from_str(identifier) {
            no_amount().map(|()| Requirement::Shard(shard))
        } else if let Some(teleporter) = identifier
            .strip_suffix("TP")
            .and_then(|identifier| Teleporter::from_str(identifier).ok())
        {
            no_amount().map(|()| Requirement::Teleporter(teleporter))
        } else {
            match identifier {
                "free" => no_amount().map(|()| Requirement::Free),
                "Impossible" => no_amount().map(|()| Requirement::Impossible),
                "SpiritLight" => get_amount().map(Requirement::SpiritLight),
                // TODO remove Ore
                "Ore" | "GorlekOre" => get_amount().map(Requirement::GorlekOre),
                "Keystone" => get_amount().map(Requirement::Keystone),
                "Water" => no_amount().map(|()| Requirement::Water),
                "Damage" => get_amount().map(|amount| Requirement::Damage(amount as f32)),
                "Danger" => get_amount().map(|amount| Requirement::Danger(amount as f32)),
                "Boss" => get_amount().map(|amount| Requirement::Boss(amount as f32)),
                "BreakWall" => get_amount().map(|amount| Requirement::BreakWall(amount as f32)),
                "BreakCrystal" => no_amount().map(|()| {
                    build_or(vec![
                        Requirement::Skill(Skill::Sword),
                        Requirement::Skill(Skill::Hammer),
                        Requirement::EnergySkill(Skill::Bow, 1.0),
                        build_and([
                            compiler.difficulty_requirements.get(Difficulty::Gorlek),
                            build_or(vec![
                                Requirement::EnergySkill(Skill::Shuriken, 1.0),
                                Requirement::EnergySkill(Skill::Grenade, 1.0),
                            ]),
                        ]),
                        build_and([
                            compiler.difficulty_requirements.get(Difficulty::Unsafe),
                            Requirement::EnergySkill(Skill::Spear, 1.0),
                        ]),
                    ])
                }),
                "SentryJump" => get_amount().map(|amount| {
                    build_and([
                        Requirement::EnergySkill(Skill::Sentry, amount as f32),
                        build_or(vec![
                            build_and([
                                compiler
                                    .trick_requirements
                                    .get(Trick::SwordSentryJump, &mut Some(amount))
                                    .unwrap(),
                                Requirement::Skill(Skill::Sword),
                            ]),
                            build_and([
                                compiler
                                    .trick_requirements
                                    .get(Trick::HammerSentryJump, &mut Some(amount))
                                    .unwrap(),
                                Requirement::Skill(Skill::Hammer),
                            ]),
                        ]),
                    ])
                }),
                // TODO remove?
                "SwordSJump" => compiler
                    .trick_requirements
                    .get(Trick::SwordSentryJump, &mut amount)
                    .ok_or_else(|| todo!()),
                // TODO remove?
                "HammerSJump" => compiler
                    .trick_requirements
                    .get(Trick::HammerSentryJump, &mut amount)
                    .ok_or_else(|| todo!()),
                "GrenadeCancel" => Ok(Requirement::NonConsumingEnergySkill(Skill::Grenade)),
                "BowCancel" => Ok(Requirement::NonConsumingEnergySkill(Skill::Bow)),
                other => match compile_state(compiler, other, self.identifier.span) {
                    Requirement::Impossible => Ok(Requirement::Impossible),
                    state => no_amount().map(|()| state),
                },
            }
        };

        compiler
            .consume_result(result)
            .unwrap_or(Requirement::Impossible)
    }
}

fn build_and<I: IntoIterator<Item = Requirement>>(requirements: I) -> Requirement {
    let mut filtered = vec![];

    for requirement in requirements {
        match requirement {
            Requirement::Free => {}
            Requirement::Impossible => return Requirement::Impossible,
            Requirement::And(nested) => filtered.extend(nested),
            other => filtered.push(other),
        }
    }

    match filtered.len() {
        0 => Requirement::Free,
        1 => filtered.pop().unwrap(),
        _ => Requirement::And(filtered),
    }
}

fn build_or<I: IntoIterator<Item = Requirement>>(requirements: I) -> Requirement {
    let mut filtered = vec![];

    for requirement in requirements {
        match requirement {
            Requirement::Free => return Requirement::Free,
            Requirement::Impossible => {}
            Requirement::Or(nested) => filtered.extend(nested),
            other => filtered.push(other),
        }
    }

    // TODO on higher difficulties there can be a lot of redundancy, could optimize those away now?

    match filtered.len() {
        0 => Requirement::Impossible,
        1 => filtered.pop().unwrap(),
        _ => Requirement::Or(filtered),
    }
}
