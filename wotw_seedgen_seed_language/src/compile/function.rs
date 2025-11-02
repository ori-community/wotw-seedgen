use super::{expression::CompileInto, Compile, SnippetCompiler, PRIVATE_MEMORY};
use crate::{
    ast::{self, UberStateType},
    compile::{add_float, add_integer, store_boolean, store_float, store_integer},
    output::{
        ArithmeticOperator, AsConstant, Command, CommandBoolean, CommandFloat, CommandInteger,
        CommandString, CommandVoid, CommandZone, Comparator, Concatenator, EqualityComparator,
        Operation,
    },
};
use arrayvec::ArrayVec;
use heck::ToTitleCase;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;
use regex::Regex;
use rustc_hash::FxHashMap;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    ops::Range,
};
use strum::{Display, EnumString, VariantArray};
use wotw_seedgen_assets::UberStateValue;
use wotw_seedgen_data::{
    Alignment, HorizontalAnchor, ScreenPosition, Shard, Skill, Teleporter, UberIdentifier,
    VerticalAnchor, WeaponUpgrade, WheelBind,
};
use wotw_seedgen_parse::{Error, Punctuated, Span, SpanEnd, SpanStart, Symbol};

// TODO could we make these helper functions const if multis used smallvecs?

pub fn spirit_light(amount: CommandInteger, rng: &mut Pcg64Mcg) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(spirit_light_string(amount.clone(), rng, false)),
            super::add_integer(UberIdentifier::SPIRIT_LIGHT, amount),
        ],
    }
}

pub fn gorlek_ore() -> CommandVoid {
    resource(gorlek_ore_string, UberIdentifier::GORLEK_ORE)
}

pub fn keystone() -> CommandVoid {
    resource(keystone_string, UberIdentifier::KEYSTONES)
}

pub fn shard_slot() -> CommandVoid {
    resource(shard_slot_string, UberIdentifier::SHARD_SLOTS)
}

pub fn health_fragment() -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(health_fragment_string(false)),
            add_integer(UberIdentifier::MAX_HEALTH, 5),
            // TODO reimplement fragment overflow bug?
            // TODO but MAX_HEALTH is just the base max health!
            store_integer(
                UberIdentifier::HEALTH,
                CommandInteger::FetchInteger {
                    uber_identifier: UberIdentifier::MAX_HEALTH,
                },
            ),
        ],
    }
}

pub fn energy_fragment() -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(energy_fragment_string(false)),
            add_float(UberIdentifier::MAX_ENERGY, 0.5),
            store_float(
                UberIdentifier::ENERGY,
                CommandFloat::FetchFloat {
                    uber_identifier: UberIdentifier::MAX_ENERGY,
                }, // TODO reimplement fragment overflow bug?
            ),
        ],
    }
}

pub fn skill(skill: Skill) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(skill_string(skill, false)),
            store_boolean(skill.uber_identifier(), true),
        ],
    }
}

pub fn shard(shard: Shard) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(shard_string(shard, false)),
            store_boolean(shard.uber_identifier(), true),
        ],
    }
}

pub fn teleporter(teleporter: Teleporter) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(teleporter_string(teleporter, false)),
            store_boolean(teleporter.uber_identifier(), true),
        ],
    }
}

pub fn clean_water() -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(clean_water_string(false)),
            store_boolean(UberIdentifier::CLEAN_WATER, true),
        ],
    }
}

pub fn weapon_upgrade(weapon_upgrade: WeaponUpgrade) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(weapon_upgrade_string(weapon_upgrade, false)),
            store_integer(weapon_upgrade.uber_identifier(), 1),
        ],
    }
}

struct ArgContext<'a, 'compiler, 'source, 'snippets, 'uberstates> {
    parameters: <Punctuated<ast::Expression<'source>, Symbol<','>> as IntoIterator>::IntoIter,
    compiler: &'a mut SnippetCompiler<'compiler, 'source, 'snippets, 'uberstates>,
}

fn arg<T: CompileInto>(context: &mut ArgContext) -> Option<T> {
    context.parameters.next()?.compile_into(context.compiler)
}

fn spanned_arg<T: CompileInto>(context: &mut ArgContext) -> Option<(T, Range<usize>)> {
    let next = context.parameters.next()?;
    let span = next.span();
    let next = next.compile_into(context.compiler)?;

    Some((next, span))
}

fn boxed_arg<T: CompileInto>(context: &mut ArgContext) -> Option<Box<T>> {
    arg(context).map(Box::new)
}

fn spanned_string_literal(context: &mut ArgContext) -> Option<(String, Range<usize>)> {
    let (arg, span) = spanned_arg::<CommandString>(context)?;

    match arg.as_constant() {
        Some(value) => Some((value.clone(), span)),
        _ => {
            context.compiler.errors.push(Error::custom(
                "Only literals are allowed in this position".to_string(),
                span,
            ));

            None
        }
    }
}

fn string_literal(context: &mut ArgContext) -> Option<String> {
    spanned_string_literal(context).map(|(value, _)| value)
}

fn boolean_id(context: &mut ArgContext) -> Option<usize> {
    string_literal(context).map(|id| context.compiler.global.boolean_ids.id(id))
}

fn integer_id(context: &mut ArgContext) -> Option<usize> {
    string_literal(context).map(|id| context.compiler.global.integer_ids.id(id))
}

fn float_id(context: &mut ArgContext) -> Option<usize> {
    string_literal(context).map(|id| context.compiler.global.float_ids.id(id))
}

fn string_id(context: &mut ArgContext) -> Option<usize> {
    string_literal(context).map(|id| context.compiler.global.string_ids.id(id))
}

fn message_id(context: &mut ArgContext) -> Option<usize> {
    string_literal(context).map(|id| context.compiler.global.message_ids.id(id))
}

fn wheel_id(context: &mut ArgContext) -> Option<usize> {
    string_literal(context).map(|id| context.compiler.global.wheel_ids.id(id))
}

fn warp_icon_id(context: &mut ArgContext) -> Option<usize> {
    string_literal(context).map(|id| context.compiler.global.warp_icon_ids.id(id))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum FunctionIdentifier {
    Fetch,
    IsInHitbox,
    GetBoolean,
    GetInteger,
    ToInteger,
    GetFloat,
    ToFloat,
    GetString,
    ToString,
    SpiritLightString,
    RemoveSpiritLightString,
    GorlekOreString,
    RemoveGorlekOreString,
    KeystoneString,
    RemoveKeystoneString,
    ShardSlotString,
    RemoveShardSlotString,
    HealthFragmentString,
    RemoveHealthFragmentString,
    EnergyFragmentString,
    RemoveEnergyFragmentString,
    SkillString,
    RemoveSkillString,
    ShardString,
    RemoveShardString,
    TeleporterString,
    RemoveTeleporterString,
    CleanWaterString,
    RemoveCleanWaterString,
    WeaponUpgradeString,
    RemoveWeaponUpgradeString,
    CurrentZone,
    CurrentMapZone,
    SpiritLight,
    RemoveSpiritLight,
    GorlekOre,
    RemoveGorlekOre,
    Keystone,
    RemoveKeystone,
    ShardSlot,
    RemoveShardSlot,
    HealthFragment,
    RemoveHealthFragment,
    EnergyFragment,
    RemoveEnergyFragment,
    Skill,
    RemoveSkill,
    Shard,
    RemoveShard,
    Teleporter,
    RemoveTeleporter,
    CleanWater,
    RemoveCleanWater,
    WeaponUpgrade,
    RemoveWeaponUpgrade,
    ItemMessage,
    ItemMessageWithTimeout,
    PriorityMessage,
    ControlledPriorityMessage,
    FreeMessage,
    DestroyMessage,
    SetMessageText,
    SetMessageTimeout,
    SetMessageBackground,
    SetMessagePosition,
    SetMessageAlignment,
    SetMessageHorizontalAnchor,
    SetMessageVerticalAnchor,
    SetMessageScreenPosition,
    SetMessageBoxWidth,
    SetMessageCoordinateSystem,
    SetMapMessage,
    Store,
    StoreWithoutTriggers,
    StoreDefaults,
    StoreDefaultsExclude,
    SetBoolean,
    SetInteger,
    SetFloat,
    SetString,
    Save,
    SaveToMemory,
    Warp,
    Equip,
    Unequip,
    TriggerKeybind,
    EnableServerSync,
    DisableServerSync,
    CreateWarpIcon,
    SetWarpIconLabel,
    DestroyWarpIcon,
    SetShopItemData,
    SetShopItemPrice,
    SetShopItemName,
    SetShopItemDescription,
    SetShopItemIcon,
    SetShopItemHidden,
    SetShopItemLocked,
    SetWheelItemData,
    SetWheelItemName,
    SetWheelItemDescription,
    SetWheelItemIcon,
    SetWheelItemColor,
    SetWheelItemAction,
    DestroyWheelItem,
    SwitchWheel,
    SetWheelPinned,
    ResetAllWheels,
    DebugLog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub args: ArrayVec<FunctionArg, 6>,
    pub return_ty: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionArg {
    pub name: &'static str,
    pub ty: &'static str,
}

macro_rules! function_signatures {
    (@acc $self:ident [$identifier:ident($($name:ident: $ty:tt),*) -> $return_ty:tt, $($more:tt)*] -> [$($acc:tt)*]) => {
        function_signatures!(@acc $self [$($more)*] -> [$($acc)*
            $identifier => FunctionSignature {
                args: ArrayVec::from_iter([
                    $(FunctionArg {
                        name: stringify!($name),
                        ty: stringify!($ty),
                    }),*
                ]),
                return_ty: Some(stringify!($return_ty)),
            },
        ])
    };

    (@acc $self:ident [$identifier:ident($($name:ident: $ty:tt),*), $($more:tt)*] -> [$($acc:tt)*]) => {
        function_signatures!(@acc $self [$($more)*] -> [$($acc)*
            $identifier => FunctionSignature {
                args: ArrayVec::from_iter([
                    $(FunctionArg {
                        name: stringify!($name),
                        ty: stringify!($ty),
                    }),*
                ]),
                return_ty: None,
            },
        ])
    };

    (@acc $self:ident [] -> [$($acc:tt)*]) => {
        {
            use FunctionIdentifier::*;

            match $self {
                $($acc)*
            }
        }
    };

    ($self:ident, $($items:tt)*) => {
        function_signatures!(@acc $self [$($items)*] -> [])
    };
}

impl FunctionIdentifier {
    pub fn signature(self) -> FunctionSignature {
        function_signatures! {
            self,
            Fetch(uber_identifier: UberIdentifier) -> ?,
            IsInHitbox(x1: Float, y1: Float, x2: Float, y2: Float) -> Boolean,
            GetBoolean(id: String) -> Boolean,
            GetInteger(id: String) -> Integer,
            ToInteger(float: Float) -> Integer,
            GetFloat(id: String) -> Float,
            ToFloat(integer: Integer) -> Float,
            GetString(id: String) -> String,
            ToString(value: ?) -> String,
            SpiritLightString(amount: Integer) -> String,
            RemoveSpiritLightString(amount: Integer) -> String,
            GorlekOreString() -> String,
            RemoveGorlekOreString() -> String,
            KeystoneString() -> String,
            RemoveKeystoneString() -> String,
            ShardSlotString() -> String,
            RemoveShardSlotString() -> String,
            HealthFragmentString() -> String,
            RemoveHealthFragmentString() -> String,
            EnergyFragmentString() -> String,
            RemoveEnergyFragmentString() -> String,
            SkillString(skill: Skill) -> String,
            RemoveSkillString(skill: Skill) -> String,
            ShardString(shard: Shard) -> String,
            RemoveShardString(shard: Shard) -> String,
            TeleporterString(teleporter: Teleporter) -> String,
            RemoveTeleporterString(teleporter: Teleporter) -> String,
            CleanWaterString() -> String,
            RemoveCleanWaterString() -> String,
            WeaponUpgradeString(weapon_upgrade: WeaponUpgrade) -> String,
            RemoveWeaponUpgradeString(weapon_upgrade: WeaponUpgrade) -> String,
            CurrentZone() -> Zone,
            CurrentMapZone() -> Zone,
            SpiritLight(amount: Integer),
            RemoveSpiritLight(amount: Integer),
            GorlekOre(),
            RemoveGorlekOre(),
            Keystone(),
            RemoveKeystone(),
            ShardSlot(),
            RemoveShardSlot(),
            HealthFragment(),
            RemoveHealthFragment(),
            EnergyFragment(),
            RemoveEnergyFragment(),
            Skill(skill: Skill),
            RemoveSkill(skill: Skill),
            Shard(shard: Shard),
            RemoveShard(shard: Shard),
            Teleporter(teleporter: Teleporter),
            RemoveTeleporter(teleporter: Teleporter),
            CleanWater(),
            RemoveCleanWater(),
            WeaponUpgrade(weapon_upgrade: WeaponUpgrade),
            RemoveWeaponUpgrade(weapon_upgrade: WeaponUpgrade),
            ItemMessage(message: String),
            ItemMessageWithTimeout(message: String, timeout: Float),
            PriorityMessage(message: String, timeout: Float),
            ControlledPriorityMessage(id: String, message: String, timeout: Float),
            FreeMessage(id: String, message: String),
            DestroyMessage(id: String),
            SetMessageText(id: String, message: String),
            SetMessageTimeout(id: String, timeout: Float),
            SetMessageBackground(id: String, background: Boolean),
            SetMessagePosition(id: String, x: Float, y: Float),
            SetMessageAlignment(id: String, alignment: Alignment),
            SetMessageHorizontalAnchor(id: String, horizontal_anchor: HorizontalAnchor),
            SetMessageVerticalAnchor(id: String, vertical_anchor: VerticalAnchor),
            SetMessageScreenPosition(id: String, screen_position: ScreenPosition),
            SetMessageBoxWidth(id: String, width: Float),
            SetMessageCoordinateSystem(id: String, coordinate_system: CoordinateSystem),
            SetMapMessage(message: String),
            Store(uber_identifier: UberIdentifier, value: ?),
            StoreWithoutTriggers(uber_identifier: UberIdentifier, value: ?),
            StoreDefaults(),
            StoreDefaultsExclude(regex: String),
            SetBoolean(id: String, value: Boolean),
            SetInteger(id: String, value: Integer),
            SetFloat(id: String, value: Float),
            SetString(id: String, value: String),
            Save(),
            SaveToMemory(),
            Warp(x: Float, y: Float),
            Equip(slot: EquipSlot, equipment: Equipment),
            Unequip(equipment: Equipment),
            TriggerKeybind(bind: String),
            EnableServerSync(uber_identifier: UberIdentifier),
            DisableServerSync(uber_identifier: UberIdentifier),
            CreateWarpIcon(id: String, x: Float, y: Float),
            SetWarpIconLabel(id: String, label: String),
            DestroyWarpIcon(id: String),
            SetShopItemData(uber_identifier: UberIdentifier, price: Integer, name: String, description: String, icon: Icon),
            SetShopItemPrice(uber_identifier: UberIdentifier, price: Integer),
            SetShopItemName(uber_identifier: UberIdentifier, name: String),
            SetShopItemDescription(uber_identifier: UberIdentifier, description: String),
            SetShopItemIcon(uber_identifier: UberIdentifier, icon: Icon),
            SetShopItemHidden(uber_identifier: UberIdentifier, hidden: Boolean),
            SetShopItemLocked(uber_identifier: UberIdentifier, locked: Boolean),
            SetWheelItemData(wheel: String, position: WheelItemPosition, name: String, description: String, icon: Icon, action: Action),
            SetWheelItemName(wheel: String, position: WheelItemPosition, name: String),
            SetWheelItemDescription(wheel: String, position: WheelItemPosition, description: String),
            SetWheelItemIcon(wheel: String, position: WheelItemPosition, icon: Icon),
            SetWheelItemColor(wheel: String, position: WheelItemPosition, red: Integer, green: Integer, blue: Integer, alpha: Integer),
            SetWheelItemAction(wheel: String, position: WheelItemPosition, bind: WheelBind, action: Action),
            DestroyWheelItem(wheel: String, position: WheelItemPosition),
            SwitchWheel(wheel: String),
            SetWheelPinned(wheel: String, pinned: Boolean),
            ResetAllWheels(),
            DebugLog(message: String),
        }
    }

    // This seems to get optimized cleanly
    fn arg_count(self) -> usize {
        self.signature().args.len()
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.args.iter().format(", "))?;

        if let Some(return_ty) = self.return_ty {
            write!(f, " -> {return_ty}")
        } else {
            Ok(())
        }
    }
}

impl Display for FunctionArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl<'source> ast::FunctionCall<'source> {
    fn compile_custom_function(
        self,
        index: usize,
        compiler: &mut SnippetCompiler,
    ) -> Option<Command> {
        if let Some(parameters) = compiler.consume_delimited(self.parameters) {
            if !parameters.is_empty() {
                compiler.errors.push(Error::custom(
                    "parameters for custom functions aren't (yet) supported".to_string(),
                    parameters[0].span().start..parameters.last.as_ref().unwrap().span().end,
                ).with_help("Use set functions for the values you want to pass and get them again in the function".to_string()))
            }
        }

        Some(Command::Void(CommandVoid::Lookup { index }))
    }

    fn compile_signature<'a, 'compiler, 'snippets, 'uberstates>(
        self,
        compiler: &'a mut SnippetCompiler<'compiler, 'source, 'snippets, 'uberstates>,
    ) -> Option<(
        FunctionIdentifier,
        ArgContext<'a, 'compiler, 'source, 'snippets, 'uberstates>,
    )> {
        let identifier = compiler.consume_result(
            self.identifier
                .data
                .0
                .parse::<FunctionIdentifier>()
                .map_err(|_| Error::custom("Unknown function".to_string(), self.identifier.span)),
        );
        let parameters = compiler.consume_result(self.parameters.content);

        let identifier = identifier?;
        let parameters = parameters?;
        let arg_count = identifier.arg_count();

        match parameters.len().cmp(&arg_count) {
            Ordering::Less => {
                let start = match &parameters.last {
                    None => self.parameters.open.span_start(),
                    Some(last) => last.span_end(),
                };
                let end = self.parameters.close.span_end();

                compiler
                    .errors
                    .push(Error::custom("Too few parameters".to_string(), start..end))
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                let start = parameters[arg_count].span_start();
                let end = self.parameters.close.span_end();

                compiler
                    .errors
                    .push(Error::custom("Too many parameters".to_string(), start..end))
            }
        }

        compiler.consume_result(self.parameters.close);

        let context = ArgContext {
            parameters: parameters.into_iter(),
            compiler,
        };

        Some((identifier, context))
    }
}

impl<'source> Compile<'source> for ast::FunctionCall<'source> {
    type Output = Option<Command>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(index) = compiler.function_indices.get(self.identifier.data.0) {
            return self.compile_custom_function(*index, compiler);
        }

        let (identifier, mut context) = self.compile_signature(compiler)?;

        let action = match identifier {
            FunctionIdentifier::Fetch => {
                let uber_identifier = context.parameters.next()?;

                let span = uber_identifier.span();
                let uber_identifier =
                    uber_identifier.compile_into::<UberIdentifier>(context.compiler)?;

                match context.compiler.uber_state_type(uber_identifier, &span)? {
                    UberStateType::Boolean => {
                        Command::Boolean(CommandBoolean::FetchBoolean { uber_identifier })
                    }
                    UberStateType::Integer => {
                        Command::Integer(CommandInteger::FetchInteger { uber_identifier })
                    }
                    UberStateType::Float => {
                        Command::Float(CommandFloat::FetchFloat { uber_identifier })
                    }
                }
            }
            FunctionIdentifier::IsInHitbox => Command::Boolean(CommandBoolean::IsInHitbox {
                x1: boxed_arg(&mut context)?,
                y1: boxed_arg(&mut context)?,
                x2: boxed_arg(&mut context)?,
                y2: boxed_arg(&mut context)?,
            }),
            FunctionIdentifier::GetBoolean => Command::Boolean(CommandBoolean::GetBoolean {
                id: boolean_id(&mut context)?,
            }),
            FunctionIdentifier::GetInteger => Command::Integer(CommandInteger::GetInteger {
                id: integer_id(&mut context)?,
            }),
            FunctionIdentifier::ToInteger => {
                let float = arg::<CommandFloat>(&mut context)?;

                let command = match float.as_constant() {
                    Some(value) => (value.round() as i32).into(),
                    None => CommandInteger::FromFloat {
                        float: Box::new(float),
                    },
                };

                Command::Integer(command)
            }
            FunctionIdentifier::GetFloat => Command::Float(CommandFloat::GetFloat {
                id: float_id(&mut context)?,
            }),
            FunctionIdentifier::ToFloat => {
                let integer = arg::<CommandInteger>(&mut context)?;

                let command = match integer.as_constant() {
                    Some(value) => (*value as f32).into(),
                    _ => CommandFloat::FromInteger {
                        integer: Box::new(integer),
                    },
                };

                Command::Float(command)
            }
            FunctionIdentifier::GetString => Command::String(CommandString::GetString {
                id: string_id(&mut context)?,
            }),
            FunctionIdentifier::ToString => {
                let (arg, span) = spanned_arg(&mut context)?;

                let command = match arg {
                    Command::Boolean(command) => match command.as_constant() {
                        Some(value) => value.to_string().into(),
                        None => CommandString::FromBoolean {
                            boolean: Box::new(command),
                        },
                    },
                    Command::Integer(command) => match command.as_constant() {
                        Some(value) => value.to_string().into(),
                        None => CommandString::FromInteger {
                            integer: Box::new(command),
                        },
                    },
                    Command::Float(command) => match command.as_constant() {
                        Some(value) => value.to_string().into(),
                        None => CommandString::FromFloat {
                            float: Box::new(command),
                        },
                    },
                    Command::String(command) => command,
                    _ => {
                        context
                            .compiler
                            .errors
                            .push(Error::custom("cannot convert to String".to_string(), span));

                        return None;
                    }
                };

                Command::String(command)
            }
            FunctionIdentifier::SpiritLightString => Command::String(spirit_light_string(
                arg(&mut context)?,
                &mut context.compiler.rng,
                false,
            )),
            FunctionIdentifier::RemoveSpiritLightString => Command::String(spirit_light_string(
                arg(&mut context)?,
                &mut context.compiler.rng,
                true,
            )),
            FunctionIdentifier::GorlekOreString => Command::String(gorlek_ore_string(false)),
            FunctionIdentifier::RemoveGorlekOreString => Command::String(gorlek_ore_string(true)),
            FunctionIdentifier::KeystoneString => Command::String(keystone_string(false)),
            FunctionIdentifier::RemoveKeystoneString => Command::String(keystone_string(true)),
            FunctionIdentifier::ShardSlotString => Command::String(shard_slot_string(false)),
            FunctionIdentifier::RemoveShardSlotString => Command::String(shard_slot_string(true)),
            FunctionIdentifier::HealthFragmentString => {
                Command::String(health_fragment_string(false))
            }
            FunctionIdentifier::RemoveHealthFragmentString => {
                Command::String(health_fragment_string(true))
            }
            FunctionIdentifier::EnergyFragmentString => {
                Command::String(energy_fragment_string(false))
            }
            FunctionIdentifier::RemoveEnergyFragmentString => {
                Command::String(energy_fragment_string(true))
            }
            FunctionIdentifier::SkillString => {
                Command::String(skill_string(arg(&mut context)?, false))
            }
            FunctionIdentifier::RemoveSkillString => {
                Command::String(skill_string(arg(&mut context)?, true))
            }
            FunctionIdentifier::ShardString => {
                Command::String(shard_string(arg(&mut context)?, false))
            }
            FunctionIdentifier::RemoveShardString => {
                Command::String(shard_string(arg(&mut context)?, true))
            }
            FunctionIdentifier::TeleporterString => {
                Command::String(teleporter_string(arg(&mut context)?, false))
            }
            FunctionIdentifier::RemoveTeleporterString => {
                Command::String(teleporter_string(arg(&mut context)?, true))
            }
            FunctionIdentifier::CleanWaterString => Command::String(clean_water_string(false)),
            FunctionIdentifier::RemoveCleanWaterString => Command::String(clean_water_string(true)),
            FunctionIdentifier::WeaponUpgradeString => {
                Command::String(weapon_upgrade_string(arg(&mut context)?, false))
            }
            FunctionIdentifier::RemoveWeaponUpgradeString => {
                Command::String(weapon_upgrade_string(arg(&mut context)?, true))
            }
            FunctionIdentifier::CurrentZone => Command::Zone(CommandZone::CurrentZone {}),
            FunctionIdentifier::CurrentMapZone => Command::Zone(CommandZone::CurrentMapZone {}),
            FunctionIdentifier::SpiritLight => {
                Command::Void(spirit_light(arg(&mut context)?, &mut context.compiler.rng))
            }
            FunctionIdentifier::RemoveSpiritLight => {
                let amount = arg::<CommandInteger>(&mut context)?;

                let negative = match amount.as_constant() {
                    Some(value) => (-value).into(),
                    None => CommandInteger::Arithmetic {
                        operation: Box::new(Operation {
                            left: amount.clone(),
                            operator: ArithmeticOperator::Multiply,
                            right: (-1).into(),
                        }),
                    },
                };

                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(spirit_light_string(amount, &mut context.compiler.rng, true)),
                        super::add_integer(UberIdentifier::SPIRIT_LIGHT, negative),
                    ],
                })
            }
            FunctionIdentifier::GorlekOre => Command::Void(gorlek_ore()),
            FunctionIdentifier::RemoveGorlekOre => {
                remove_resource(gorlek_ore_string, UberIdentifier::GORLEK_ORE)
            }
            FunctionIdentifier::Keystone => Command::Void(keystone()),
            FunctionIdentifier::RemoveKeystone => {
                remove_resource(keystone_string, UberIdentifier::KEYSTONES)
            }
            FunctionIdentifier::ShardSlot => Command::Void(shard_slot()),
            FunctionIdentifier::RemoveShardSlot => {
                remove_resource(shard_slot_string, UberIdentifier::SHARD_SLOTS)
            }
            FunctionIdentifier::HealthFragment => Command::Void(health_fragment()),
            FunctionIdentifier::RemoveHealthFragment => Command::Void(CommandVoid::Multi {
                commands: vec![
                    item_message(health_fragment_string(true)),
                    add_integer(UberIdentifier::MAX_HEALTH, -5),
                ],
            }),
            FunctionIdentifier::EnergyFragment => Command::Void(energy_fragment()),
            FunctionIdentifier::RemoveEnergyFragment => Command::Void(CommandVoid::Multi {
                commands: vec![
                    item_message(energy_fragment_string(true)),
                    add_float(UberIdentifier::MAX_ENERGY, -0.5),
                ],
            }),
            FunctionIdentifier::Skill => Command::Void(skill(arg(&mut context)?)),
            FunctionIdentifier::RemoveSkill => {
                let skill = arg(&mut context)?;

                let mut commands = vec![
                    item_message(skill_string(skill, true)),
                    store_boolean(skill.uber_identifier(), false),
                ];

                if let Some(equipment) = skill.equipment() {
                    commands.push(CommandVoid::Unequip { equipment });
                }

                Command::Void(CommandVoid::Multi { commands })
            }
            FunctionIdentifier::Shard => Command::Void(shard(arg(&mut context)?)),
            FunctionIdentifier::RemoveShard => {
                let shard = arg(&mut context)?;

                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(shard_string(shard, true)),
                        store_boolean(shard.uber_identifier(), false),
                    ],
                })
            }
            FunctionIdentifier::Teleporter => Command::Void(teleporter(arg(&mut context)?)),
            FunctionIdentifier::RemoveTeleporter => {
                let teleporter = arg(&mut context)?;

                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(teleporter_string(teleporter, true)),
                        // TODO remove map segment?
                        store_boolean(teleporter.uber_identifier(), false),
                    ],
                })
            }
            FunctionIdentifier::CleanWater => Command::Void(clean_water()),
            FunctionIdentifier::RemoveCleanWater => Command::Void(CommandVoid::Multi {
                commands: vec![
                    item_message(clean_water_string(true)),
                    store_boolean(UberIdentifier::CLEAN_WATER, false),
                ],
            }),
            FunctionIdentifier::WeaponUpgrade => Command::Void(weapon_upgrade(arg(&mut context)?)),
            FunctionIdentifier::RemoveWeaponUpgrade => {
                let weapon_upgrade = arg(&mut context)?;

                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(weapon_upgrade_string(weapon_upgrade, true)),
                        store_integer(weapon_upgrade.uber_identifier(), 0),
                    ],
                })
            }
            FunctionIdentifier::ItemMessage => Command::Void(item_message(arg(&mut context)?)),
            FunctionIdentifier::ItemMessageWithTimeout => {
                Command::Void(CommandVoid::QueuedMessage {
                    id: None,
                    priority: false,
                    message: arg(&mut context)?,
                    timeout: Some(arg(&mut context)?),
                })
            }
            FunctionIdentifier::PriorityMessage => Command::Void(CommandVoid::QueuedMessage {
                id: None,
                priority: true,
                message: arg(&mut context)?,
                timeout: Some(arg(&mut context)?),
            }),
            FunctionIdentifier::ControlledPriorityMessage => {
                Command::Void(CommandVoid::QueuedMessage {
                    id: Some(message_id(&mut context)?),
                    priority: true,
                    message: arg(&mut context)?,
                    timeout: Some(arg(&mut context)?),
                })
            }
            FunctionIdentifier::FreeMessage => Command::Void(CommandVoid::FreeMessage {
                id: message_id(&mut context)?,
                message: arg(&mut context)?,
            }),
            FunctionIdentifier::DestroyMessage => Command::Void(CommandVoid::MessageDestroy {
                id: message_id(&mut context)?,
            }),
            FunctionIdentifier::SetMessageText => Command::Void(CommandVoid::MessageText {
                id: message_id(&mut context)?,
                message: arg(&mut context)?,
            }),
            FunctionIdentifier::SetMessageTimeout => Command::Void(CommandVoid::MessageTimeout {
                id: message_id(&mut context)?,
                timeout: arg(&mut context)?,
            }),
            FunctionIdentifier::SetMessageBackground => {
                Command::Void(CommandVoid::MessageBackground {
                    id: message_id(&mut context)?,
                    background: arg(&mut context)?,
                })
            }
            // TODO should check on these whether the message is a type of message that you can set the position of
            // maybe also make
            FunctionIdentifier::SetMessagePosition => {
                Command::Void(CommandVoid::FreeMessagePosition {
                    id: message_id(&mut context)?,
                    x: arg(&mut context)?,
                    y: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetMessageAlignment => {
                Command::Void(CommandVoid::FreeMessageAlignment {
                    id: message_id(&mut context)?,
                    alignment: arg(&mut context)?,
                })
            }

            FunctionIdentifier::SetMessageHorizontalAnchor => {
                Command::Void(CommandVoid::FreeMessageHorizontalAnchor {
                    id: message_id(&mut context)?,
                    horizontal_anchor: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetMessageVerticalAnchor => {
                Command::Void(CommandVoid::FreeMessageVerticalAnchor {
                    id: message_id(&mut context)?,
                    vertical_anchor: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetMessageScreenPosition => {
                let id = message_id(&mut context)?;

                let commands = match arg(&mut context)? {
                    ScreenPosition::TopLeft => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Left,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Left,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Top,
                        },
                    ],
                    ScreenPosition::TopCenter => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Center,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Center,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Top,
                        },
                    ],
                    ScreenPosition::TopRight => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Right,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Right,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Top,
                        },
                    ],
                    ScreenPosition::MiddleLeft => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Left,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Left,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Middle,
                        },
                    ],
                    ScreenPosition::MiddleCenter => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Center,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Center,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Middle,
                        },
                    ],
                    ScreenPosition::MiddleRight => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Right,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Right,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Middle,
                        },
                    ],
                    ScreenPosition::BottomLeft => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Left,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Left,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Bottom,
                        },
                    ],
                    ScreenPosition::BottomCenter => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Center,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Center,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Bottom,
                        },
                    ],
                    ScreenPosition::BottomRight => vec![
                        CommandVoid::FreeMessageAlignment {
                            id,
                            alignment: Alignment::Right,
                        },
                        CommandVoid::FreeMessageHorizontalAnchor {
                            id,
                            horizontal_anchor: HorizontalAnchor::Right,
                        },
                        CommandVoid::FreeMessageVerticalAnchor {
                            id,
                            vertical_anchor: VerticalAnchor::Bottom,
                        },
                    ],
                };

                Command::Void(CommandVoid::Multi { commands })
            }
            FunctionIdentifier::SetMessageBoxWidth => {
                Command::Void(CommandVoid::FreeMessageBoxWidth {
                    id: message_id(&mut context)?,
                    width: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetMessageCoordinateSystem => {
                Command::Void(CommandVoid::FreeMessageCoordinateSystem {
                    id: message_id(&mut context)?,
                    coordinate_system: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetMapMessage => Command::Void(CommandVoid::SetMapMessage {
                value: arg(&mut context)?,
            }),
            FunctionIdentifier::Store => store(true, &mut context)?,
            FunctionIdentifier::StoreWithoutTriggers => store(false, &mut context)?,
            FunctionIdentifier::StoreDefaults => {
                store_defaults("*store_defaults".to_string(), &mut context, |_| true)
            }
            FunctionIdentifier::StoreDefaultsExclude => {
                let (regex, span) = spanned_string_literal(&mut context)?;

                let identifier = format!("*store_defaults_exclude{regex}");

                let regex = context.compiler.consume_result(
                    Regex::new(&regex)
                        .map_err(|err| Error::custom(format!("Invalid regex: {err}"), span)),
                )?;

                store_defaults(identifier, &mut context, |uber_identifier| {
                    !regex.is_match(&uber_identifier.to_string())
                })
            }
            FunctionIdentifier::SetBoolean => Command::Void(CommandVoid::SetBoolean {
                id: boolean_id(&mut context)?,
                value: arg(&mut context)?,
            }),
            FunctionIdentifier::SetInteger => Command::Void(CommandVoid::SetInteger {
                id: integer_id(&mut context)?,
                value: arg(&mut context)?,
            }),
            FunctionIdentifier::SetFloat => Command::Void(CommandVoid::SetFloat {
                id: float_id(&mut context)?,
                value: arg(&mut context)?,
            }),
            FunctionIdentifier::SetString => Command::Void(CommandVoid::SetString {
                id: string_id(&mut context)?,
                value: arg(&mut context)?,
            }),
            FunctionIdentifier::Save => Command::Void(CommandVoid::Save {}),
            FunctionIdentifier::SaveToMemory => Command::Void(CommandVoid::SaveToMemory {}),
            FunctionIdentifier::Warp => Command::Void(CommandVoid::Warp {
                x: arg(&mut context)?,
                y: arg(&mut context)?,
            }),
            FunctionIdentifier::Equip => Command::Void(CommandVoid::Equip {
                slot: arg(&mut context)?,
                equipment: arg(&mut context)?,
            }),
            FunctionIdentifier::Unequip => Command::Void(CommandVoid::Unequip {
                equipment: arg(&mut context)?,
            }),
            FunctionIdentifier::TriggerKeybind => Command::Void(CommandVoid::TriggerKeybind {
                bind: arg(&mut context)?,
            }),
            FunctionIdentifier::EnableServerSync => Command::Void(CommandVoid::EnableServerSync {
                uber_identifier: arg(&mut context)?,
            }),
            FunctionIdentifier::DisableServerSync => {
                Command::Void(CommandVoid::DisableServerSync {
                    uber_identifier: arg(&mut context)?,
                })
            }
            FunctionIdentifier::CreateWarpIcon => Command::Void(CommandVoid::CreateWarpIcon {
                id: warp_icon_id(&mut context)?,
                x: arg(&mut context)?,
                y: arg(&mut context)?,
            }),
            FunctionIdentifier::SetWarpIconLabel => Command::Void(CommandVoid::SetWarpIconLabel {
                id: warp_icon_id(&mut context)?,
                label: arg(&mut context)?,
            }),
            FunctionIdentifier::DestroyWarpIcon => Command::Void(CommandVoid::DestroyWarpIcon {
                id: warp_icon_id(&mut context)?,
            }),
            FunctionIdentifier::SetShopItemData => {
                let uber_identifier = arg::<UberIdentifier>(&mut context)?;

                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        CommandVoid::SetShopItemPrice {
                            uber_identifier,
                            price: arg(&mut context)?,
                        },
                        CommandVoid::SetShopItemName {
                            uber_identifier,
                            name: arg(&mut context)?,
                        },
                        CommandVoid::SetShopItemDescription {
                            uber_identifier,
                            description: arg(&mut context)?,
                        },
                        CommandVoid::SetShopItemIcon {
                            uber_identifier,
                            icon: arg(&mut context)?,
                        },
                    ],
                })
            }
            FunctionIdentifier::SetShopItemPrice => Command::Void(CommandVoid::SetShopItemPrice {
                uber_identifier: arg(&mut context)?,
                price: arg(&mut context)?,
            }),
            FunctionIdentifier::SetShopItemName => Command::Void(CommandVoid::SetShopItemName {
                uber_identifier: arg(&mut context)?,
                name: arg(&mut context)?,
            }),
            FunctionIdentifier::SetShopItemDescription => {
                Command::Void(CommandVoid::SetShopItemDescription {
                    uber_identifier: arg(&mut context)?,
                    description: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetShopItemIcon => Command::Void(CommandVoid::SetShopItemIcon {
                uber_identifier: arg(&mut context)?,
                icon: arg(&mut context)?,
            }),
            FunctionIdentifier::SetShopItemHidden => {
                Command::Void(CommandVoid::SetShopItemHidden {
                    uber_identifier: arg(&mut context)?,
                    hidden: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetShopItemLocked => {
                Command::Void(CommandVoid::SetShopItemLocked {
                    uber_identifier: arg(&mut context)?,
                    locked: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetWheelItemData => {
                let wheel = wheel_id(&mut context)?;
                let position = arg(&mut context)?;

                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        CommandVoid::SetWheelItemName {
                            wheel,
                            position,
                            name: arg(&mut context)?,
                        },
                        CommandVoid::SetWheelItemDescription {
                            wheel,
                            position,
                            description: arg(&mut context)?,
                        },
                        CommandVoid::SetWheelItemIcon {
                            wheel,
                            position,
                            icon: arg(&mut context)?,
                        },
                        CommandVoid::SetWheelItemAction {
                            wheel,
                            position,
                            bind: WheelBind::All,
                            action: arg(&mut context)?,
                        },
                    ],
                })
            }
            FunctionIdentifier::SetWheelItemName => Command::Void(CommandVoid::SetWheelItemName {
                wheel: wheel_id(&mut context)?,
                position: arg(&mut context)?,
                name: arg(&mut context)?,
            }),
            FunctionIdentifier::SetWheelItemDescription => {
                Command::Void(CommandVoid::SetWheelItemDescription {
                    wheel: wheel_id(&mut context)?,
                    position: arg(&mut context)?,
                    description: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetWheelItemIcon => Command::Void(CommandVoid::SetWheelItemIcon {
                wheel: wheel_id(&mut context)?,
                position: arg(&mut context)?,
                icon: arg(&mut context)?,
            }),
            FunctionIdentifier::SetWheelItemColor => {
                Command::Void(CommandVoid::SetWheelItemColor {
                    wheel: wheel_id(&mut context)?,
                    position: arg(&mut context)?,
                    red: arg(&mut context)?,
                    green: arg(&mut context)?,
                    blue: arg(&mut context)?,
                    alpha: arg(&mut context)?,
                })
            }
            FunctionIdentifier::SetWheelItemAction => {
                Command::Void(CommandVoid::SetWheelItemAction {
                    wheel: wheel_id(&mut context)?,
                    position: arg(&mut context)?,
                    bind: arg(&mut context)?,
                    action: arg(&mut context)?,
                })
            }
            FunctionIdentifier::DestroyWheelItem => Command::Void(CommandVoid::DestroyWheelItem {
                wheel: wheel_id(&mut context)?,
                position: arg(&mut context)?,
            }),
            FunctionIdentifier::SwitchWheel => Command::Void(CommandVoid::SwitchWheel {
                wheel: wheel_id(&mut context)?,
            }),
            FunctionIdentifier::SetWheelPinned => Command::Void(CommandVoid::SetWheelPinned {
                wheel: wheel_id(&mut context)?,
                pinned: arg(&mut context)?,
            }),
            FunctionIdentifier::ResetAllWheels => Command::Void(CommandVoid::ResetAllWheels {}),
            FunctionIdentifier::DebugLog => Command::Void(CommandVoid::DebugLog {
                message: arg(&mut context)?,
            }),
        };

        if let Some(excess) = context.parameters.next() {
            let span = excess.span();
            let end = context
                .parameters
                .last()
                .map_or(span.end, |last| last.span().end);

            context.compiler.errors.push(Error::custom(
                "Too many parameters".to_string(),
                span.start..end,
            ));

            return None;
        }

        Some(action)
    }
}

fn item_message(message: CommandString) -> CommandVoid {
    CommandVoid::QueuedMessage {
        id: None,
        priority: false,
        message,
        timeout: None,
    }
}

fn set_spirit_light_string(value: &str) -> Box<CommandVoid> {
    Box::new(CommandVoid::SetString {
        id: PRIVATE_MEMORY,
        value: value.into(),
    })
}

fn spirit_light_string(amount: CommandInteger, rng: &mut Pcg64Mcg, remove: bool) -> CommandString {
    CommandString::Multi {
        commands: vec![
            CommandVoid::If {
                condition: CommandBoolean::FetchBoolean {
                    uber_identifier: UberIdentifier {
                        group: 29,
                        member: 0,
                    },
                },
                command: set_random_spirit_light_string(&amount, rng),
            },
            CommandVoid::If {
                condition: CommandBoolean::CompareBoolean {
                    operation: Box::new(Operation {
                        left: CommandBoolean::FetchBoolean {
                            uber_identifier: UberIdentifier {
                                group: 29,
                                member: 0,
                            },
                        },
                        operator: EqualityComparator::Equal,
                        right: false.into(),
                    }),
                },
                command: set_spirit_light_string("Spirit Light"),
            },
        ],
        last: Box::new(if remove {
            CommandString::Concatenate {
                operation: Box::new(Operation {
                    left: match amount.as_constant() {
                        Some(value) => format!("@Remove {value} ").into(),
                        None => CommandString::Concatenate {
                            operation: Box::new(Operation {
                                left: "@Remove ".into(),
                                operator: Concatenator::Concat,
                                right: CommandString::Concatenate {
                                    operation: Box::new(Operation {
                                        left: CommandString::FromInteger {
                                            integer: Box::new(amount),
                                        },
                                        operator: Concatenator::Concat,
                                        right: " ".into(),
                                    }),
                                },
                            }),
                        },
                    },
                    operator: Concatenator::Concat,
                    right: CommandString::Concatenate {
                        operation: Box::new(Operation {
                            left: CommandString::GetString { id: PRIVATE_MEMORY },
                            operator: Concatenator::Concat,
                            right: "@".into(),
                        }),
                    },
                }),
            }
        } else {
            CommandString::Concatenate {
                operation: Box::new(Operation {
                    left: match amount.as_constant() {
                        Some(value) => format!("{value} ").into(),
                        None => CommandString::Concatenate {
                            operation: Box::new(Operation {
                                left: CommandString::FromInteger {
                                    integer: Box::new(amount),
                                },
                                operator: Concatenator::Concat,
                                right: " ".into(),
                            }),
                        },
                    },
                    operator: Concatenator::Concat,
                    right: CommandString::GetString { id: PRIVATE_MEMORY },
                }),
            }
        }),
    }
}

fn set_random_spirit_light_string(amount: &CommandInteger, rng: &mut Pcg64Mcg) -> Box<CommandVoid> {
    let name = SPIRIT_LIGHT_NAMES.choose(rng).unwrap();
    let constant_singular = amount.as_constant().map(|amount| matches!(amount, 1 | -1));

    match constant_singular {
        Some(true) => set_spirit_light_string(name.0),
        Some(false) => set_spirit_light_string(name.1),
        None => Box::new(CommandVoid::Multi {
            commands: vec![
                CommandVoid::SetBoolean {
                    id: PRIVATE_MEMORY,
                    value: CommandBoolean::CompareInteger {
                        operation: Box::new(Operation {
                            left: CommandInteger::Arithmetic {
                                operation: Box::new(Operation {
                                    left: amount.clone(),
                                    operator: ArithmeticOperator::Multiply,
                                    right: amount.clone(),
                                }),
                            },
                            operator: Comparator::Equal,
                            right: 1.into(),
                        }),
                    },
                },
                CommandVoid::If {
                    condition: CommandBoolean::GetBoolean { id: PRIVATE_MEMORY },
                    command: set_spirit_light_string(name.0),
                },
                CommandVoid::If {
                    condition: CommandBoolean::CompareBoolean {
                        operation: Box::new(Operation {
                            left: CommandBoolean::GetBoolean { id: PRIVATE_MEMORY },
                            operator: EqualityComparator::Equal,
                            right: false.into(),
                        }),
                    },
                    command: set_spirit_light_string(name.1),
                },
            ],
        }),
    }
}

fn resource_string(resource: &str, remove: bool) -> CommandString {
    if remove {
        format!("@Remove {resource}@").into()
    } else {
        resource.into()
    }
}

fn gorlek_ore_string(remove: bool) -> CommandString {
    resource_string("Gorlek Ore", remove)
}

fn keystone_string(remove: bool) -> CommandString {
    resource_string("Keystone", remove)
}

fn shard_slot_string(remove: bool) -> CommandString {
    resource_string("Shard Slot", remove)
}

fn health_fragment_string(remove: bool) -> CommandString {
    resource_string("Health Fragment", remove)
}

fn energy_fragment_string(remove: bool) -> CommandString {
    resource_string("Energy Fragment", remove)
}

fn skill_string(skill: Skill, remove: bool) -> CommandString {
    let skill_cased = skill.to_string().to_title_case();

    if remove {
        format!("@Remove {skill_cased}@")
    } else {
        match skill {
            Skill::GladesAncestralLight | Skill::MarshAncestralLight => {
                format!("#{skill_cased}#")
            }
            _ => format!("*{skill_cased}*"),
        }
    }
    .into()
}

fn shard_string(shard: Shard, remove: bool) -> CommandString {
    let shard_cased = shard.to_string().to_title_case();

    if remove {
        format!("@Remove {shard_cased}@")
    } else {
        format!("${shard_cased}$")
    }
    .into()
}

fn teleporter_string(teleporter: Teleporter, remove: bool) -> CommandString {
    let teleporter = teleporter.to_string();
    let teleporter = &teleporter[..teleporter.len() - 2];
    let teleporter_cased = teleporter.to_string().to_title_case();

    if remove {
        format!("@Remove {teleporter_cased} Teleporter@")
    } else {
        format!("#{teleporter_cased} Teleporter#")
    }
    .into()
}

fn clean_water_string(remove: bool) -> CommandString {
    if remove {
        "@Remove Clean Water@"
    } else {
        "*Clean Water*"
    }
    .into()
}

// TODO remove as const?
fn weapon_upgrade_string(weapon_upgrade: WeaponUpgrade, remove: bool) -> CommandString {
    let weapon_upgrade_cased = weapon_upgrade.to_string().to_title_case();

    if remove {
        format!("@Remove {weapon_upgrade_cased}@")
    } else {
        format!("#{weapon_upgrade_cased}#")
    }
    .into()
}

fn resource(string_fn: fn(bool) -> CommandString, uber_identifier: UberIdentifier) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(string_fn(false)),
            add_integer(uber_identifier, 1),
        ],
    }
}

fn remove_resource(
    string_fn: fn(bool) -> CommandString,
    uber_identifier: UberIdentifier,
) -> Command {
    Command::Void(CommandVoid::Multi {
        commands: vec![
            item_message(string_fn(true)),
            add_integer(uber_identifier, -1),
        ],
    })
}

fn store(trigger_events: bool, context: &mut ArgContext) -> Option<Command> {
    let (uber_identifier, span) = spanned_arg::<UberIdentifier>(context)?;

    let command = match context.compiler.uber_state_type(uber_identifier, &span)? {
        UberStateType::Boolean => CommandVoid::StoreBoolean {
            uber_identifier,
            value: arg(context)?,
            trigger_events,
        },
        UberStateType::Integer => CommandVoid::StoreInteger {
            uber_identifier,
            value: arg(context)?,
            trigger_events,
        },
        UberStateType::Float => CommandVoid::StoreFloat {
            uber_identifier,
            value: arg(context)?,
            trigger_events,
        },
    };

    if context
        .compiler
        .global
        .uber_state_data
        .id_lookup
        .get(&uber_identifier)
        .is_some_and(|entry| entry.readonly)
    {
        context.compiler.errors.push(Error::custom(
            "this uberState is readonly".to_string(),
            span,
        ));
    }

    Some(Command::Void(command))
}

fn store_defaults<F>(identifier: String, context: &mut ArgContext, mut condition: F) -> Command
where
    F: FnMut(UberIdentifier) -> bool,
{
    lookup_or_insert(
        identifier,
        &mut context.compiler.function_indices,
        &mut context.compiler.global.output.command_lookup,
        || CommandVoid::Multi {
            commands: context
                .compiler
                .global
                .uber_state_data
                .id_lookup
                .iter()
                .filter(|(uber_identifier, meta)| !meta.readonly && condition(**uber_identifier))
                .map(|(uber_identifier, meta)| match &meta.default_value {
                    UberStateValue::Boolean(value) => store_boolean(*uber_identifier, *value),
                    UberStateValue::Integer(value) => store_integer(*uber_identifier, *value),
                    UberStateValue::Float(value) => store_float(*uber_identifier, *value),
                })
                .collect(),
        },
    )
}

fn lookup_or_insert<F>(
    identifier: String,
    function_indices: &mut FxHashMap<String, usize>,
    command_lookup: &mut Vec<CommandVoid>,
    command: F,
) -> Command
where
    F: FnOnce() -> CommandVoid,
{
    let index = *function_indices.entry(identifier).or_insert_with(|| {
        let index = command_lookup.len();
        command_lookup.push(command());

        index
    });

    Command::Void(CommandVoid::Lookup { index })
}

const SPIRIT_LIGHT_NAMES: [(&str, &str); 87] = [
    ("Banana", "Bananas"),
    ("Bell", "Bells"),
    ("Bit", "Bits"),
    ("Bolt", "Bolts"),
    ("Boonbuck", "Boonbucks"),
    ("Boxing", "Boxings"),
    ("Brick", "Brick"),
    ("Brownie Point", "Brownie Points"),
    ("Byte", "Bytes"),
    ("Cash", "Cash"),
    ("Coin", "Coins"),
    ("Comment", "Comments"),
    ("Credit", "Credits"),
    ("Crown", "Crowns"),
    ("Diamond", "Diamonds"),
    ("Dollar", "Dollars"),
    ("Dollerydoo", "Dollerydoos"),
    ("Doubloon", "Doubloons"),
    ("Dram", "Drams"),
    ("Echoe", "Echoes"),
    ("Emerald", "Emeralds"),
    ("Euro", "Euros"),
    ("Exalted Orb", "Exalted Orbs"),
    ("EXP", "EXP"),
    ("Experience", "Experience"),
    ("Farthing", "Farthings"),
    ("Fish", "Fish"),
    ("Fun", "Fun"),
    ("Gallon", "Gallons"),
    ("Geo", "Geo"),
    ("Gil", "Gil"),
    ("Glod", "Glod"),
    ("Gold", "Gold"),
    ("GP", "GP"),
    ("Hryvnia", "Hryvnia"),
    ("Hug", "Hugs"),
    ("Kalganid", "Kalganids"),
    ("Leaf", "Leaves"),
    ("Like", "Likes"),
    ("Marble", "Marbles"),
    ("Mineral", "Minerals"),
    ("Money", "Money"),
    ("Munny", "Munny"),
    ("Noble", "Nobles"),
    ("Note", "Notes"),
    ("Nut", "Nuts"),
    ("Nuyen", "Nuyen"),
    ("Ori", "Ori Money"),
    ("Pesos", "Pesos"),
    ("Piece of Eight", "Pieces of Eight"),
    ("Point", "Points"),
    ("Poké", "Poké"),
    ("Pon", "Pons"),
    ("Pound Sterling", "Pounds Sterling"),
    ("Quatloo", "Quatloos"),
    ("Quill", "Quills"),
    ("Ring", "Rings"),
    ("Rosary", "Rosaries"),
    ("Ruby", "Rubies"),
    ("Rune", "Runes"),
    ("Rupee", "Rupees"),
    ("Sapphire", "Sapphires"),
    ("Sheep", "Sheep"),
    ("Shilling", "Shillings"),
    ("Silver", "Silver"),
    ("Sliver", "Slivers"),
    ("Smackerooni", "Smackeroonis"),
    ("Sock", "Socks"),
    ("Solari", "Solari"),
    ("Soul", "Souls"),
    ("Sovereign", "Sovereigns"),
    ("Sphere", "Spheres"),
    ("Spirit Buck", "Spirit Bucks"),
    ("Spirit", "Spirit Light"),
    ("Stamp", "Stamps"),
    ("Stonk", "Stonks"),
    ("Sub", "Subs"),
    ("Ticket", "Tickets"),
    ("Token", "Tokens"),
    ("Vespine", "Vespine Gas"),
    ("Wheat", "Wheat"),
    ("Widget", "Widgets"),
    ("Wood", "Wood"),
    ("XP", "XP"),
    ("Yen", "Yen"),
    ("Zenny", "Zenny"),
    ("Złoty", "Złoty"),
];
