use std::{borrow::Cow, iter, mem, ops::Range, str::FromStr};

use crate::{
    assets::{LocData, StateData, StateDataEntry},
    logic_language::{
        ast,
        optimize::Optimize,
        output::{
            Anchor, Connection, Enemy, Entrance, EntranceId, Graph, Node, Refill, RefillValue,
            Requirement,
        },
        token::Tokenizer,
    },
    Difficulty, Position, Shard, Skill, Teleporter, Trick, UberIdentifier, WorldSettings,
    WorldSettingsHelpers,
};
use rustc_hash::FxHashMap;
use wotw_seedgen_log_capture::{LogCapture, NO_LOG_CAPTURE};
use wotw_seedgen_parse::{
    AsItem, Error, ErrorKind, ParseResult, Recover, Recoverable, Result, Separated,
    SeparatedNonEmptyGeneric, Span, Spanned, SpannedOption,
};

// TODO not really part of compilation but some kind of lints would be nice, like:
// verify all states were used
// verify nothing was used mixed as either quest or pickup
// verify all anchors are connected
// check for implicit moki

impl Graph {
    pub fn compiler() -> CompilerConfig<'static, 'static> {
        CompilerConfig::new()
    }
}

pub struct CompilerConfig<'settings, 'log> {
    settings: &'settings [WorldSettings],
    log_capture: &'log LogCapture,
}

impl CompilerConfig<'static, 'static> {
    pub const fn new() -> Self {
        Self {
            settings: &[],
            log_capture: &NO_LOG_CAPTURE,
        }
    }
}

impl<'settings, 'log> CompilerConfig<'settings, 'log> {
    /// `settings` used for optimization
    pub const fn with_settings(mut self, settings: &'settings [WorldSettings]) -> Self {
        self.settings = settings;
        self
    }

    pub const fn with_log_capture(mut self, log_capture: &'log LogCapture) -> Self {
        self.log_capture = log_capture;
        self
    }

    pub fn compile(
        self,
        mut paths: ast::Paths,
        loc_data: LocData,
        state_data: StateData,
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

        let mut compiler = Compiler::new(
            &mut paths,
            &loc_data_nodes,
            &state_data_nodes,
            self.settings,
            self.log_capture,
        );

        paths.contents.compile(&mut compiler);
        compiler.generate_entrance_connections();

        let Compiler {
            nodes: compiled_nodes,
            mut extern_requirements,
            mut default_entrance_connections,
            errors,
            ..
        } = compiler;

        let mut nodes: Vec<Node> = loc_data_nodes;
        nodes.reserve_exact(state_data_nodes.len() + compiled_nodes.len());
        nodes.extend(state_data_nodes);
        nodes.extend(compiled_nodes);

        nodes.shrink_to_fit();
        extern_requirements.shrink_to_fit();
        default_entrance_connections.shrink_to_fit();

        let mut graph = Graph {
            nodes,
            extern_requirements,
            default_entrance_connections,
        };

        graph.optimize(self.log_capture);

        ParseResult {
            parsed: Some(graph),
            errors,
        }
    }
}

struct Compiler<'source, 'log> {
    nodes: Vec<Node>,
    extern_requirements: Vec<Requirement>,
    index_offset: usize,
    entrance_nodes: Vec<usize>,
    default_entrance_connections: FxHashMap<EntranceId, EntranceId>,
    requirements: RequirementRepository,
    state_map: FxHashMap<Cow<'source, str>, usize>,
    pickup_map: FxHashMap<&'source str, usize>,
    anchor_map: FxHashMap<String, usize>,
    extern_requirement_map: FxHashMap<String, usize>,
    regions: FxHashMap<String, Requirement>,
    errors: Vec<Error>,
    log_capture: &'log LogCapture,
}

impl<'source, 'log> Compiler<'source, 'log> {
    fn new(
        paths: &mut ast::Paths,
        loc_data_nodes: &'source [Node],
        state_data_nodes: &'source [Node],
        settings: &[WorldSettings],
        log_capture: &'log LogCapture,
    ) -> Self {
        let mut errors = vec![];
        let mut node_index_iter = 0..;

        // index_iter has to be the second iterator so it doesn't get incremented when the first one runs out
        let mut pickup_map = FxHashMap::default();
        for (node, index) in iter::zip(loc_data_nodes, &mut node_index_iter) {
            let identifier = node.identifier();
            if pickup_map.insert(identifier, index).is_some() {
                errors.push(Error::error(
                    format!("duplicate identifier \"{identifier}\" in loc_data"),
                    0..0,
                ));
            }
        }

        let mut state_map = FxHashMap::default();
        for (node, index) in iter::zip(state_data_nodes, &mut node_index_iter) {
            let identifier = node.identifier();
            if state_map.insert(Cow::Borrowed(identifier), index).is_some() {
                errors.push(Error::error(
                    format!("duplicate identifier \"{identifier}\" in state_data"),
                    0..0,
                ));
            }
        }

        let mut nodes = vec![]; // TODO capacity
        let mut anchors = Vec::with_capacity(300); // We know more about the needed capacity than collect would
        let mut anchor_map = FxHashMap::default();
        let mut extern_requirement_map = FxHashMap::default();
        let mut extern_requirement_index_iter = 0..;

        for content in &paths.contents {
            let SpannedOption::Some(content) = &content.value else {
                continue;
            };

            match content {
                ast::Content::Requirement(_, requirement) => {
                    let SpannedOption::Some(requirement) = &requirement.value else {
                        continue;
                    };

                    let previous = extern_requirement_map.insert(
                        requirement.identifier.data.0.to_string(),
                        extern_requirement_index_iter.next().unwrap(),
                    );

                    if previous.is_some() {
                        errors.push(Error::error(
                            "duplicate identifier".to_string(),
                            requirement.identifier.span.clone(),
                        ));
                    }
                }
                ast::Content::Region(..) => {}
                ast::Content::Anchor(_, anchor) => {
                    let SpannedOption::Some(anchor) = &anchor.value else {
                        continue;
                    };

                    anchors.push(anchor);

                    let SpannedOption::Some(group) = &anchor.content.content.value else {
                        continue;
                    };

                    for connection in &group.content {
                        let ast::AnchorContent::Connection(
                            Spanned {
                                data: ast::ConnectionKeyword::State,
                                ..
                            },
                            Recoverable {
                                value: SpannedOption::Some(state_connection),
                                ..
                            },
                        ) = connection
                        else {
                            continue;
                        };

                        if !state_map.contains_key(state_connection.identifier.data.0) {
                            nodes.push(Node::LogicalState(
                                state_connection.identifier.data.0.to_string(),
                            ));

                            state_map.insert(
                                Cow::Owned(state_connection.identifier.data.0.to_string()),
                                node_index_iter.next().unwrap(),
                            );
                        }
                    }
                }
            }
        }

        for (index, anchor) in node_index_iter.zip(anchors) {
            if anchor_map.contains_key(anchor.identifier.data.0) {
                errors.push(Error::error(
                    format!("duplicate identifier \"{}\"", anchor.identifier.data.0),
                    anchor.identifier.span.clone(),
                ));
            } else {
                anchor_map.insert(anchor.identifier.data.0.to_string(), index);
            }
        }

        Self {
            nodes,
            extern_requirements: vec![],
            index_offset: loc_data_nodes.len() + state_data_nodes.len(),
            entrance_nodes: vec![],
            default_entrance_connections: FxHashMap::default(),
            requirements: RequirementRepository::new(settings),
            state_map,
            pickup_map,
            anchor_map,
            extern_requirement_map,
            regions: FxHashMap::default(),
            errors,
            log_capture,
        }
    }

    fn error(&mut self, message: String, span: Range<usize>) {
        self.errors.push(Error::error(message, span));
    }

    fn consume_result<T>(&mut self, result: Result<T>) -> Option<T> {
        result.map_err(|err| self.errors.push(err)).ok()
    }

    fn generate_entrance_connections(&mut self) {
        for index in self.entrance_nodes.iter().copied() {
            let anchor = self.nodes[index].expect_anchor();
            let anchor_identifier = anchor.identifier.clone();
            let entrance = anchor.entrance.as_ref().unwrap();
            let entrance_id = entrance.id;
            let entrance_requirement = entrance.requirement.clone();

            if let Some(target) = self.anchor_map.get(&entrance.target) {
                let target_entrance_id = self.nodes[*target - self.index_offset]
                    .expect_anchor()
                    .entrance
                    .as_ref()
                    .unwrap()
                    .id;

                self.default_entrance_connections
                    .insert(entrance_id, target_entrance_id);
            }

            let visited_state_index = self.nodes.len();
            // TODO during seedgen this should be 1, but start as 0 in the true seed
            self.nodes.push(Node::State(StateDataEntry {
                identifier: format!("{anchor_identifier}Visited"),
                uber_identifier: UberIdentifier::known_entrance_connections(entrance_id),
                value: None,
            }));

            for target_index in self.entrance_nodes.iter().copied() {
                if index == target_index {
                    continue;
                }

                let state_index = self.nodes.len();
                self.nodes[index]
                    .expect_anchor_mut()
                    .connections
                    .push(Connection {
                        to: self.index_offset + target_index,
                        requirement: Requirement::and([
                            Requirement::State(self.index_offset + state_index),
                            Requirement::State(self.index_offset + visited_state_index),
                            entrance_requirement.clone(),
                        ]),
                        implicitly_generated: true,
                    });

                let target_anchor = self.nodes[target_index].expect_anchor(); // TODO verify while parsing

                self.nodes.push(Node::State(StateDataEntry {
                    identifier: format!("{} to {}", anchor_identifier, target_anchor.identifier),
                    uber_identifier: UberIdentifier::entrances(entrance_id),
                    value: Some(target_anchor.entrance.as_ref().unwrap().id), // TODO this should not be treated as strictly incremental
                }));
            }
        }
    }
}

struct RequirementRepository {
    difficulty: DifficultyRequirements,
    trick: TrickRequirements,
    normal: Requirement,
}

impl RequirementRepository {
    fn new(settings: &[WorldSettings]) -> Self {
        Self {
            difficulty: DifficultyRequirements::new(settings),
            trick: TrickRequirements::new(settings),
            normal: setting_requirement(
                settings,
                WorldSettingsHelpers::all_play_hard,
                WorldSettingsHelpers::none_play_hard,
                Requirement::NormalGameDifficulty,
            ),
        }
    }

    fn difficulty(&self, difficulty: Difficulty) -> Requirement {
        match difficulty {
            Difficulty::Moki => self.difficulty.moki.clone(),
            Difficulty::Gorlek => self.difficulty.gorlek.clone(),
            Difficulty::Kii => self.difficulty.kii.clone(),
            Difficulty::Unsafe => self.difficulty.r#unsafe.clone(),
        }
    }

    fn trick(&self, trick: Trick, amount: &mut Option<usize>) -> Option<Requirement> {
        let requirement = match trick {
            Trick::SwordSentryJump => Requirement::and([
                self.trick.sword_sentry_jump.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
                Requirement::Skill(Skill::Sword),
            ]),
            Trick::HammerSentryJump => Requirement::and([
                self.trick.hammer_sentry_jump.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::ShurikenBreak => Requirement::and([
                self.trick.shuriken_break.clone(),
                Requirement::ShurikenBreak(amount.take()? as f32),
            ]),
            Trick::SentryBreak => Requirement::and([
                self.trick.sentry_break.clone(),
                Requirement::SentryBreak(amount.take()? as f32),
            ]),
            Trick::HammerBreak => Requirement::and([
                self.trick.hammer_break.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::SpearBreak => Requirement::and([
                self.trick.spear_break.clone(),
                Requirement::EnergySkill(Skill::Spear, 1.),
            ]),
            Trick::SentryBurn => Requirement::and([
                self.trick.sentry_burn.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
            ]),
            Trick::RemoveKillPlane => self.trick.remove_kill_plane.clone(),
            Trick::LaunchSwap => Requirement::and([
                self.trick.launch_swap.clone(),
                Requirement::Skill(Skill::Launch),
            ]),
            Trick::SentrySwap => Requirement::and([
                self.trick.sentry_swap.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
            ]),
            Trick::FlashSwap => Requirement::and([
                self.trick.flash_swap.clone(),
                Requirement::NonConsumingEnergySkill(Skill::Flash),
            ]),
            Trick::BlazeSwap => Requirement::and([
                self.trick.blaze_swap.clone(),
                Requirement::EnergySkill(Skill::Blaze, amount.take()? as f32),
            ]),
            Trick::WaveDash => Requirement::and([
                self.trick.wave_dash.clone(),
                Requirement::Skill(Skill::Dash),
                Requirement::NonConsumingEnergySkill(Skill::Regenerate),
            ]),
            Trick::GrenadeJump => Requirement::and([
                self.trick.grenade_jump.clone(),
                Requirement::or([
                    Requirement::and([
                        self.difficulty.r#unsafe.clone(),
                        Requirement::NonConsumingEnergySkill(Skill::Grenade),
                    ]),
                    match amount.take() {
                        None => Requirement::Impossible,
                        Some(amount) => Requirement::EnergySkill(Skill::Grenade, amount as f32),
                    },
                ]),
            ]),
            Trick::SwordJump => Requirement::and([
                self.trick.sword_jump.clone(),
                Requirement::Skill(Skill::Sword),
                Requirement::Skill(Skill::DoubleJump),
            ]),
            Trick::GlideJump => Requirement::and([
                self.trick.glide_jump.clone(),
                Requirement::Skill(Skill::Glide),
            ]),
            Trick::AerialHammerJump => Requirement::and([
                self.trick.aerial_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
                Requirement::Skill(Skill::DoubleJump),
            ]),
            Trick::GlideHammerJump => Requirement::and([
                self.trick.glide_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
                Requirement::Skill(Skill::Glide),
            ]),
            Trick::CoyoteHammerJump => Requirement::and([
                self.trick.coyote_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::WallHammerJump => Requirement::and([
                self.trick.wall_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::GroundedHammerJump => Requirement::and([
                self.trick.grounded_hammer_jump.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::HammerExtension => Requirement::and([
                self.trick.hammer_extension.clone(),
                Requirement::Skill(Skill::Hammer),
            ]),
            Trick::GrenadeRedirect => Requirement::and([
                self.trick.grenade_redirect.clone(),
                Requirement::EnergySkill(Skill::Grenade, amount.take()? as f32),
            ]),
            Trick::SentryRedirect => Requirement::and([
                self.trick.sentry_redirect.clone(),
                Requirement::EnergySkill(Skill::Sentry, amount.take()? as f32),
            ]),
            Trick::PauseFloat => self.trick.pause_float.clone(),
            Trick::SpearJump => Requirement::and([
                self.trick.spear_jump.clone(),
                Requirement::EnergySkill(Skill::Spear, amount.take()? as f32),
            ]),
            Trick::GlideBashChain => Requirement::and([
                self.trick.glide_bash_chain.clone(),
                Requirement::Skill(Skill::Glide),
                Requirement::Skill(Skill::Bash),
            ]),
            Trick::DoubleJumpBashChain => Requirement::and([
                self.trick.double_jump_bash_chain.clone(),
                Requirement::Skill(Skill::DoubleJump),
                Requirement::Skill(Skill::Bash),
            ]),
            Trick::DashBashChain => Requirement::and([
                self.trick.dash_bash_chain.clone(),
                Requirement::Skill(Skill::Dash),
                Requirement::Skill(Skill::Bash),
            ]),
            Trick::LaunchBashChain => Requirement::and([
                self.trick.launch_bash_chain.clone(),
                Requirement::Skill(Skill::Launch),
                Requirement::Skill(Skill::Bash),
            ]),
            Trick::Unpopular => self.trick.unpopular.clone(),
        };

        Some(requirement)
    }
}

struct DifficultyRequirements {
    moki: Requirement,
    gorlek: Requirement,
    kii: Requirement,
    r#unsafe: Requirement,
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
            r#unsafe: build_difficulty(Difficulty::Unsafe),
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
    glide_jump: Requirement,
    aerial_hammer_jump: Requirement,
    glide_hammer_jump: Requirement,
    coyote_hammer_jump: Requirement,
    wall_hammer_jump: Requirement,
    grounded_hammer_jump: Requirement,
    hammer_extension: Requirement,
    grenade_redirect: Requirement,
    sentry_redirect: Requirement,
    pause_float: Requirement,
    spear_jump: Requirement,
    glide_bash_chain: Requirement,
    double_jump_bash_chain: Requirement,
    dash_bash_chain: Requirement,
    launch_bash_chain: Requirement,
    unpopular: Requirement,
}

impl TrickRequirements {
    fn new(settings: &[WorldSettings]) -> Self {
        let build_trick = move |trick| {
            setting_requirement(
                settings,
                |settings| settings.none_contain_trick(trick),
                |settings| settings.all_contain_trick(trick),
                Requirement::Trick(trick),
            )
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
            glide_jump: build_trick(Trick::GlideJump),
            aerial_hammer_jump: build_trick(Trick::AerialHammerJump),
            glide_hammer_jump: build_trick(Trick::GlideHammerJump),
            coyote_hammer_jump: build_trick(Trick::CoyoteHammerJump),
            wall_hammer_jump: build_trick(Trick::WallHammerJump),
            grounded_hammer_jump: build_trick(Trick::GroundedHammerJump),
            hammer_extension: build_trick(Trick::HammerExtension),
            grenade_redirect: build_trick(Trick::GrenadeRedirect),
            sentry_redirect: build_trick(Trick::SentryRedirect),
            pause_float: build_trick(Trick::PauseFloat),
            spear_jump: build_trick(Trick::SpearJump),
            glide_bash_chain: build_trick(Trick::GlideBashChain),
            double_jump_bash_chain: build_trick(Trick::DoubleJumpBashChain),
            dash_bash_chain: build_trick(Trick::DashBashChain),
            launch_bash_chain: build_trick(Trick::LaunchBashChain),
            unpopular: build_trick(Trick::Unpopular),
        }
    }
}

fn setting_requirement<S, FN, FA>(settings: &S, none: FN, all: FA, req: Requirement) -> Requirement
where
    S: WorldSettingsHelpers + ?Sized,
    FN: FnOnce(&S) -> bool,
    FA: FnOnce(&S) -> bool,
{
    if settings.is_empty() {
        req
    } else if none(settings) {
        Requirement::Impossible
    } else if all(settings) {
        Requirement::Free
    } else {
        req
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

impl<T, First, Separator> Compile for SeparatedNonEmptyGeneric<First, T, Separator>
where
    T: Compile,
    First: AsItem<T>,
{
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

impl Compile for ast::Content<'_> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        match self {
            ast::Content::Requirement(_, content) => content.compile(compiler),
            ast::Content::Region(_, content) => content.compile(compiler),
            ast::Content::Anchor(_, content) => content.compile(compiler),
        };
    }
}

impl Compile for ast::Macro<'_> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let requirement = self
            .requirements
            .compile(compiler)
            .map_or(Requirement::Impossible, Requirement::or);

        compiler.extern_requirements.push(requirement);
    }
}

impl Compile for ast::Region<'_> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        if compiler.regions.contains_key(self.identifier.data.0) {
            compiler.error("duplicate identifier".to_string(), self.identifier.span);
        } else {
            let mut requirement = self
                .requirements
                .compile(compiler)
                .map_or(Requirement::Impossible, Requirement::or);

            // Region requirements are instantiated often, we'd rather optimize it once here
            requirement.optimize(compiler.log_capture);

            compiler
                .regions
                .insert(self.identifier.data.0.to_string(), requirement);
        }
    }
}

impl Compile for ast::Anchor<'_> {
    type Output = ();

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let position = self
            .position
            .and_then(|position| position.compile(compiler));

        let mut entrance = None;
        let mut can_spawn = true;
        let mut teleport_restriction = None;
        let mut refills = vec![];
        let mut connections = vec![];

        if let SpannedOption::Some(content) = self.content.content.value {
            for content in content.content {
                match content {
                    ast::AnchorContent::Entrance(keyword, anchor_entrance) => {
                        if let Some(anchor_entrance) = anchor_entrance.compile(compiler) {
                            compiler.entrance_nodes.push(compiler.nodes.len());

                            if entrance.replace(anchor_entrance).is_some() {
                                compiler.error("duplicate entrance".to_string(), keyword.span);
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
                            .into_option()
                            .map_or(Requirement::Impossible, |group| group.compile(compiler));

                        if teleport_restriction.replace(requirement).is_some() {
                            compiler.error("duplicate tprestriction".to_string(), keyword.span);
                        }
                    }
                    ast::AnchorContent::Refill(_, refill) => {
                        refills.extend(refill.compile(compiler));
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
                                        requirement = Requirement::and([
                                            // TODO the placement of the region requirement has some complex trade-offs
                                            // For instance, solving the region requirement earlier before there's many branches is nice,
                                            // but checking whether it's met and then running into an unmet requirement later is bad.
                                            region_requirement.clone(),
                                            requirement,
                                        ]);
                                    }

                                    connections.push(Connection {
                                        to,
                                        requirement,
                                        implicitly_generated: false,
                                    });
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
            entrance,
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

impl Compile for ast::Entrance<'_> {
    type Output = Option<Entrance>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let mut id = None;
        let mut target = None;
        let mut requirement = None;

        if let SpannedOption::Some(content) = self.content.value {
            for content in content.content {
                match content {
                    ast::EntranceContent::Id(keyword, entrance_id) => {
                        if let SpannedOption::Some(entrance_id) = entrance_id.value {
                            if id.replace(entrance_id.id.data).is_some() {
                                compiler.error("duplicate id".to_string(), keyword.span);
                            }
                        }
                    }
                    ast::EntranceContent::Target(keyword, entrance_target) => {
                        if let SpannedOption::Some(entrance_target) = entrance_target.value {
                            if !compiler
                                .anchor_map
                                .contains_key(entrance_target.target.data.0)
                            {
                                compiler.error(
                                    "unknown anchor".to_string(),
                                    entrance_target.target.span,
                                );
                            }

                            if target
                                .replace(entrance_target.target.data.0.to_string())
                                .is_some()
                            {
                                compiler.error("duplicate target".to_string(), keyword.span);
                            }
                        }
                    }
                    ast::EntranceContent::Enter(keyword, entrance_requirement) => {
                        if let Some(entrance_requirement) = entrance_requirement.compile(compiler) {
                            if requirement.replace(entrance_requirement).is_some() {
                                compiler.error("duplicate enter".to_string(), keyword.span);
                            }
                        }
                    }
                }
            }
        }

        if let (Some(id), Some(target), Some(requirement)) = (id, target, requirement) {
            Some(Entrance {
                id,
                target,
                requirement,
            })
        } else {
            None
        }
    }
}

impl Compile for ast::Refill<'_> {
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

impl Compile for ast::RequirementLineOrGroup<'_> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.requirement
            .compile(compiler)
            .unwrap_or(Requirement::Impossible)
    }
}

impl Compile for ast::InlineRequirementOrGroup<'_> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        match self {
            ast::InlineRequirementOrGroup::Inline(_) => Requirement::Free,
            ast::InlineRequirementOrGroup::Group(group) => Requirement::or(group.compile(compiler)),
        }
    }
}

impl Compile for ast::RequirementLine<'_> {
    type Output = Requirement;

    fn compile(mut self, compiler: &mut Compiler) -> Self::Output {
        // iter::chain doesn't consider it valid to mutably borrow the compiler in both parts
        // a vec would lose out on the short-circuiting behaviour of Requirement::and
        // so we build a custom iterator
        let mut ands = self.ands.into_iter();
        let mut ors = Some(self.ors);

        Requirement::and(iter::from_fn(|| {
            ands.next()
                .map(|(and, _)| and.compile(compiler))
                .or_else(|| ors.take().map(|ors| Requirement::or(ors.compile(compiler))))
                .or_else(|| {
                    self.group.take().map(|group| {
                        group
                            .compile(compiler)
                            .map_or(Requirement::Impossible, Requirement::or)
                    })
                })
        }))
    }
}

impl Compile for ast::Requirement<'_> {
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
            compiler.error(format!("Unknown requirement \"{identifier}\""), span);
            Requirement::Impossible
        }
        Some(index) => Requirement::State(*index),
    }
}

impl Compile for ast::CombatRequirement<'_> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        self.enemies
            .compile(compiler)
            .and_then(|enemies| enemies.into_iter().collect())
            .map_or(Requirement::Impossible, Requirement::Combat)
    }
}

impl Compile for ast::Enemy<'_> {
    type Output = Option<(Enemy, u8)>;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        compiler
            .consume_result(
                self.identifier
                    .data
                    .0
                    .parse()
                    .map_err(|_| Error::error("unknown enemy".to_string(), self.identifier.span)),
            )
            .map(|enemy| {
                let amount = self.amount.map_or(1, |amount| amount.value.data);
                (enemy, amount)
            })
    }
}

impl Compile for ast::PlainRequirement<'_> {
    type Output = Requirement;

    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let identifier = self.identifier.data.0;
        let amount_span = self.amount.as_ref().map_or(
            self.identifier.span.end..self.identifier.span.end,
            ast::Amount::span,
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
                Err(Error::error(
                    "this requirement accepts no amount".to_string(),
                    amount_span.clone(),
                ))
            }
        };

        let result = if let Some(extern_index) = compiler.extern_requirement_map.get(identifier) {
            Ok(Requirement::Extern(*extern_index))
        } else if let Ok(skill) = Skill::from_str(identifier) {
            match amount {
                None => Ok(Requirement::Skill(skill)),
                Some(amount) => {
                    if skill.energy_cost() == 0. {
                        Err(Error::error(
                            "this skill accepts no amount".to_string(),
                            amount_span,
                        ))
                    } else {
                        Ok(Requirement::EnergySkill(skill, amount as f32))
                    }
                }
            }
        } else if let Ok(difficulty) = Difficulty::from_str(identifier) {
            no_amount().map(|()| compiler.requirements.difficulty(difficulty))
        } else if let Ok(trick) = Trick::from_str(identifier) {
            let option = compiler.requirements.trick(trick, &mut amount);

            if amount.is_some() {
                Err(Error::error(
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
                // TODO free is lowercase but impossible is uppercase
                "free" => no_amount().map(|()| Requirement::Free),
                "Impossible" => no_amount().map(|()| Requirement::Impossible),
                "NormalGameDifficulty" => {
                    no_amount().map(|()| compiler.requirements.normal.clone())
                }
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
                    Requirement::or([
                        Requirement::Skill(Skill::Sword),
                        Requirement::Skill(Skill::Hammer),
                        Requirement::EnergySkill(Skill::Bow, 1.0),
                        Requirement::and([
                            compiler.requirements.difficulty(Difficulty::Gorlek),
                            Requirement::or([
                                Requirement::EnergySkill(Skill::Shuriken, 1.0),
                                Requirement::EnergySkill(Skill::Grenade, 1.0),
                            ]),
                        ]),
                        Requirement::and([
                            compiler.requirements.difficulty(Difficulty::Unsafe),
                            Requirement::EnergySkill(Skill::Spear, 1.0),
                        ]),
                    ])
                }),
                // TODO this doubles the energy and skill reqs
                "SentryJump" => get_amount().map(|amount| {
                    Requirement::and([
                        Requirement::EnergySkill(Skill::Sentry, amount as f32),
                        Requirement::or([
                            Requirement::and([
                                compiler
                                    .requirements
                                    .trick(Trick::SwordSentryJump, &mut Some(amount))
                                    .unwrap(),
                                Requirement::Skill(Skill::Sword),
                            ]),
                            Requirement::and([
                                compiler
                                    .requirements
                                    .trick(Trick::HammerSentryJump, &mut Some(amount))
                                    .unwrap(),
                                Requirement::Skill(Skill::Hammer),
                            ]),
                        ]),
                    ])
                }),
                // TODO remove?
                "SwordSJump" => compiler
                    .requirements
                    .trick(Trick::SwordSentryJump, &mut amount)
                    .ok_or_else(|| todo!()),
                // TODO remove?
                "HammerSJump" => compiler
                    .requirements
                    .trick(Trick::HammerSentryJump, &mut amount)
                    .ok_or_else(|| todo!()),
                "AbilitySwap" => get_amount().map(|amount| {
                    Requirement::or([
                        compiler
                            .requirements
                            .trick(Trick::SentrySwap, &mut Some(amount))
                            .unwrap(),
                        compiler
                            .requirements
                            .trick(Trick::FlashSwap, &mut Some(amount))
                            .unwrap(),
                        compiler
                            .requirements
                            .trick(Trick::BlazeSwap, &mut Some(amount))
                            .unwrap(),
                    ])
                }),
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
