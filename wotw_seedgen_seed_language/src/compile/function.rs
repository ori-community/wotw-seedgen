use super::{expression::CompileInto, Compile, SnippetCompiler, RESERVED_MEMORY};
use crate::{
    ast::{self, UberStateType},
    output::{
        ArithmeticOperator, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
        CommandVoid, CommandZone, EqualityComparator, Operation, StringOrPlaceholder,
    },
};
use convert_case::{Case, Casing};
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;
use std::ops::Range;
use strum::EnumString;
use wotw_seedgen_data::{
    uber_identifier, Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade, WheelBind,
};
use wotw_seedgen_parse::{Error, Punctuated, Span, Symbol};

pub fn spirit_light(amount: CommandInteger, rng: &mut Pcg64Mcg) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(spirit_light_string(amount.clone(), rng, false)),
            super::add_integer(uber_identifier::SPIRIT_LIGHT, amount),
        ],
    }
}
pub fn gorlek_ore() -> CommandVoid {
    resource(gorlek_ore_string, uber_identifier::GORLEK_ORE)
}
pub fn keystone() -> CommandVoid {
    resource(keystone_string, uber_identifier::KEYSTONES)
}
pub fn shard_slot() -> CommandVoid {
    resource(shard_slot_string, uber_identifier::SHARD_SLOTS)
}
pub fn health_fragment() -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(health_fragment_string(false)),
            super::add_integer_value(uber_identifier::MAX_HEALTH, 5),
            super::set_integer(
                uber_identifier::HEALTH,
                CommandInteger::FetchInteger {
                    uber_identifier: uber_identifier::MAX_HEALTH,
                }, // TODO reimplement fragment overflow bug?
            ),
        ],
    }
}
pub fn energy_fragment() -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(energy_fragment_string(false)),
            super::add_float_value(uber_identifier::MAX_ENERGY, 0.5.into()),
            super::set_float(
                uber_identifier::ENERGY,
                CommandFloat::FetchFloat {
                    uber_identifier: uber_identifier::MAX_ENERGY,
                }, // TODO reimplement fragment overflow bug?
            ),
        ],
    }
}
pub fn skill(skill: Skill) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(skill_string(skill, false)),
            super::set_boolean_value(skill.uber_identifier(), true),
        ],
    }
}
pub fn shard(shard: Shard) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(shard_string(shard, false)),
            super::set_boolean_value(shard.uber_identifier(), true),
        ],
    }
}
pub fn teleporter(teleporter: Teleporter) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(teleporter_string(teleporter, false)),
            super::set_boolean_value(teleporter.uber_identifier(), true),
        ],
    }
}
pub fn clean_water() -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(clean_water_string(false)),
            super::set_boolean_value(uber_identifier::CLEAN_WATER, true),
        ],
    }
}
pub fn weapon_upgrade(weapon_upgrade: WeaponUpgrade) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(weapon_upgrade_string(weapon_upgrade, false)),
            super::set_boolean_value(weapon_upgrade.uber_identifier(), true),
        ],
    }
}

struct ArgContext<'a, 'compiler, 'source, 'snippets, 'uberstates> {
    span: Range<usize>,
    parameters: <Punctuated<ast::Expression<'source>, Symbol<','>> as IntoIterator>::IntoIter,
    compiler: &'a mut SnippetCompiler<'compiler, 'source, 'snippets, 'uberstates>,
}
fn try_next<'source>(
    context: &mut ArgContext<'_, '_, 'source, '_, '_>,
) -> Option<ast::Expression<'source>> {
    let next = context.parameters.next();
    if next.is_none() {
        context.compiler.errors.push(Error::custom(
            "Too few parameters".to_string(), // TODO help would be great here
            context.span.clone(),
        ))
    }
    next
}
fn arg<T: CompileInto>(context: &mut ArgContext) -> Option<T> {
    try_next(context)?.compile_into(context.compiler)
}
fn spanned_arg<T: CompileInto>(context: &mut ArgContext) -> Option<(T, Range<usize>)> {
    let next = try_next(context)?;
    let span = next.span();
    let next = next.compile_into(context.compiler)?;
    Some((next, span))
}
fn boxed_arg<T: CompileInto>(context: &mut ArgContext) -> Option<Box<T>> {
    arg(context).map(Box::new)
}
fn spanned_string_literal(context: &mut ArgContext) -> Option<(String, Range<usize>)> {
    let (arg, span) = spanned_arg(context)?;
    match arg {
        CommandString::Constant {
            value: StringOrPlaceholder::Value(value),
        } => Some((value, span)),
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

#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum FunctionIdentifier {
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
    SetMessageScreenPosition,
    Store,
    StoreWithoutTriggers,
    SetBoolean,
    SetInteger,
    SetFloat,
    SetString,
    Save,
    Checkpoint,
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
    ClearAllWheels,
}

impl<'source> Compile<'source> for ast::FunctionCall<'source> {
    type Output = Option<Command>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(&index) = compiler.function_indices.get(self.identifier.data.0) {
            // TODO are we dropping a result here?
            if let Ok(parameters) = &self.parameters.content {
                if !parameters.is_empty() {
                    compiler.errors.push(Error::custom(
                    "parameters for custom functions aren't (yet) supported".to_string(),
                    parameters.first().unwrap().span().start..parameters.last.as_ref().unwrap().span().end,
                ).with_help("Use set commands for the values you want to pass and get them again in the function".to_string()))
                }
            }
            return Some(Command::Void(CommandVoid::Lookup { index }));
        }
        let identifier =
            compiler.consume_result(self.identifier.data.0.parse().map_err(|_| {
                Error::custom("Unknown function".to_string(), self.identifier.span)
            }))?;

        let span = self.parameters.span();
        let content = compiler.consume_result(self.parameters.content)?;

        let span = match &content.last {
            Some(last) => content.first().unwrap().span().start..last.span().end,
            None => span,
        };

        let mut context = ArgContext {
            span,
            parameters: content.into_iter(),
            compiler,
        };

        let action = match identifier {
            FunctionIdentifier::Fetch => {
                let uber_identifier = try_next(&mut context)?;
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
            FunctionIdentifier::IsInHitbox => {
                Command::Boolean(CommandBoolean::IsInHitbox {
                    x1: boxed_arg(&mut context)?, // TODO we short circuit potential error messages here, but this does avoid duplicate "too few arguments" errors, so we'd need a different approach to begin with
                    y1: boxed_arg(&mut context)?,
                    x2: boxed_arg(&mut context)?,
                    y2: boxed_arg(&mut context)?,
                })
            }
            FunctionIdentifier::GetBoolean => Command::Boolean(CommandBoolean::GetBoolean {
                id: boolean_id(&mut context)?,
            }),
            FunctionIdentifier::GetInteger => Command::Integer(CommandInteger::GetInteger {
                id: integer_id(&mut context)?,
            }),
            FunctionIdentifier::ToInteger => {
                let float = arg(&mut context)?;
                let command = match float {
                    CommandFloat::Constant { value } => CommandInteger::Constant {
                        value: value.round() as i32,
                    },
                    _ => CommandInteger::FromFloat {
                        float: Box::new(float),
                    },
                };
                Command::Integer(command)
            }
            FunctionIdentifier::GetFloat => Command::Float(CommandFloat::GetFloat {
                id: float_id(&mut context)?,
            }),
            FunctionIdentifier::ToFloat => {
                let integer = arg(&mut context)?;
                let command = match integer {
                    CommandInteger::Constant { value } => CommandFloat::Constant {
                        value: (value as f32).into(),
                    },
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
                    Command::Boolean(command) => match command {
                        CommandBoolean::Constant { value } => CommandString::Constant {
                            value: value.to_string().into(),
                        },
                        other => CommandString::FromBoolean {
                            boolean: Box::new(other),
                        },
                    },
                    Command::Integer(command) => match command {
                        CommandInteger::Constant { value } => CommandString::Constant {
                            value: value.to_string().into(),
                        },
                        other => CommandString::FromInteger {
                            integer: Box::new(other),
                        },
                    },
                    Command::Float(command) => match command {
                        CommandFloat::Constant { value } => CommandString::Constant {
                            value: value.to_string().into(),
                        },
                        other => CommandString::FromFloat {
                            float: Box::new(other),
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
            FunctionIdentifier::SpiritLight => {
                Command::Void(spirit_light(arg(&mut context)?, &mut context.compiler.rng))
            }
            FunctionIdentifier::RemoveSpiritLight => {
                let amount = arg::<CommandInteger>(&mut context)?;
                let negative = match amount.clone() {
                    CommandInteger::Constant { value } => {
                        CommandInteger::Constant { value: -value }
                    }
                    other => CommandInteger::Arithmetic {
                        operation: Box::new(Operation {
                            left: other,
                            operator: ArithmeticOperator::Multiply,
                            right: CommandInteger::Constant { value: -1 },
                        }),
                    },
                };
                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(spirit_light_string(amount, &mut context.compiler.rng, true)),
                        super::add_integer(uber_identifier::SPIRIT_LIGHT, negative),
                    ],
                })
            }
            FunctionIdentifier::GorlekOre => Command::Void(gorlek_ore()),
            FunctionIdentifier::RemoveGorlekOre => {
                remove_resource(gorlek_ore_string, uber_identifier::GORLEK_ORE)
            }
            FunctionIdentifier::Keystone => Command::Void(keystone()),
            FunctionIdentifier::RemoveKeystone => {
                remove_resource(keystone_string, uber_identifier::KEYSTONES)
            }
            FunctionIdentifier::ShardSlot => Command::Void(shard_slot()),
            FunctionIdentifier::RemoveShardSlot => {
                remove_resource(shard_slot_string, uber_identifier::SHARD_SLOTS)
            }
            FunctionIdentifier::HealthFragment => Command::Void(health_fragment()),
            FunctionIdentifier::RemoveHealthFragment => Command::Void(CommandVoid::Multi {
                commands: vec![
                    item_message(health_fragment_string(true)),
                    super::add_integer_value(uber_identifier::MAX_HEALTH, -5),
                ],
            }),
            FunctionIdentifier::EnergyFragment => Command::Void(energy_fragment()),
            FunctionIdentifier::RemoveEnergyFragment => Command::Void(CommandVoid::Multi {
                commands: vec![
                    item_message(energy_fragment_string(true)),
                    super::add_float_value(uber_identifier::MAX_ENERGY, (-0.5).into()),
                ],
            }),
            FunctionIdentifier::Skill => Command::Void(skill(arg(&mut context)?)),
            FunctionIdentifier::RemoveSkill => {
                let skill = arg(&mut context)?;
                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(skill_string(skill, true)),
                        super::set_boolean_value(skill.uber_identifier(), false),
                        CommandVoid::Unequip {
                            equipment: skill.equipment(),
                        },
                    ],
                })
            }
            FunctionIdentifier::Shard => Command::Void(shard(arg(&mut context)?)),
            FunctionIdentifier::RemoveShard => {
                let shard = arg(&mut context)?;
                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(shard_string(shard, true)),
                        super::set_boolean_value(shard.uber_identifier(), false),
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
                        super::set_boolean_value(teleporter.uber_identifier(), false),
                    ],
                })
            }
            FunctionIdentifier::CleanWater => Command::Void(clean_water()),
            FunctionIdentifier::RemoveCleanWater => Command::Void(CommandVoid::Multi {
                commands: vec![
                    item_message(clean_water_string(true)),
                    super::set_boolean_value(uber_identifier::CLEAN_WATER, false),
                ],
            }),
            FunctionIdentifier::WeaponUpgrade => Command::Void(weapon_upgrade(arg(&mut context)?)),
            FunctionIdentifier::RemoveWeaponUpgrade => {
                let weapon_upgrade = arg(&mut context)?;
                Command::Void(CommandVoid::Multi {
                    commands: vec![
                        item_message(weapon_upgrade_string(weapon_upgrade, true)),
                        super::set_boolean_value(weapon_upgrade.uber_identifier(), false),
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
            FunctionIdentifier::SetMessageScreenPosition => {
                Command::Void(CommandVoid::FreeMessageScreenPosition {
                    id: message_id(&mut context)?,
                    screen_position: arg(&mut context)?,
                })
            }
            FunctionIdentifier::Store => store(true, &mut context)?,
            FunctionIdentifier::StoreWithoutTriggers => store(false, &mut context)?,
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
            FunctionIdentifier::Checkpoint => Command::Void(CommandVoid::Checkpoint {}),
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
            FunctionIdentifier::ClearAllWheels => Command::Void(CommandVoid::ClearAllWheels {}),
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

fn spirit_light_string(amount: CommandInteger, rng: &mut Pcg64Mcg, remove: bool) -> CommandString {
    const SPIRIT_LIGHT_STRING_ID: usize = RESERVED_MEMORY + 1;
    CommandString::Multi {
        commands: vec![
            CommandVoid::If {
                condition: CommandBoolean::FetchBoolean {
                    uber_identifier: UberIdentifier {
                        group: 27,
                        member: 0,
                    },
                },
                command: Box::new(CommandVoid::SetString {
                    id: SPIRIT_LIGHT_STRING_ID,
                    value: CommandString::Constant {
                        value: (*SPIRIT_LIGHT_NAMES.choose(rng).unwrap()).into(),
                    },
                }),
            },
            CommandVoid::If {
                condition: CommandBoolean::CompareBoolean {
                    operation: Box::new(Operation {
                        left: CommandBoolean::FetchBoolean {
                            uber_identifier: UberIdentifier {
                                group: 27,
                                member: 0,
                            },
                        },
                        operator: EqualityComparator::Equal,
                        right: CommandBoolean::Constant { value: false },
                    }),
                },
                command: Box::new(CommandVoid::SetString {
                    id: SPIRIT_LIGHT_STRING_ID,
                    value: CommandString::Constant {
                        value: "Spirit Light".into(),
                    },
                }),
            },
        ],
        last: Box::new(if remove {
            CommandString::Concatenate {
                left: Box::new(match amount {
                    CommandInteger::Constant { value } => CommandString::Constant {
                        value: format!("@Remove {value} ").into(),
                    },
                    other => CommandString::Concatenate {
                        left: Box::new(CommandString::Constant {
                            value: "@Remove ".into(),
                        }),
                        right: Box::new(CommandString::Concatenate {
                            left: Box::new(CommandString::FromInteger {
                                integer: Box::new(other),
                            }),
                            right: Box::new(CommandString::Constant { value: " ".into() }),
                        }),
                    },
                }),
                right: Box::new(CommandString::Concatenate {
                    left: Box::new(CommandString::GetString {
                        id: SPIRIT_LIGHT_STRING_ID,
                    }),
                    right: Box::new(CommandString::Constant { value: "@".into() }),
                }),
            }
        } else {
            CommandString::Concatenate {
                left: Box::new(match amount {
                    CommandInteger::Constant { value } => CommandString::Constant {
                        value: format!("{value} ").into(),
                    },
                    other => CommandString::Concatenate {
                        left: Box::new(CommandString::FromInteger {
                            integer: Box::new(other),
                        }),
                        right: Box::new(CommandString::Constant { value: " ".into() }),
                    },
                }),
                right: Box::new(CommandString::GetString {
                    id: SPIRIT_LIGHT_STRING_ID,
                }),
            }
        }),
    }
}
fn resource_string(resource: &str, remove: bool) -> CommandString {
    let value = if remove {
        format!("@Remove {resource}@").into()
    } else {
        resource.into()
    };
    CommandString::Constant { value }
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
    let skill_cased = skill
        .to_string()
        .from_case(Case::Pascal)
        .to_case(Case::Title);
    let value = if remove {
        format!("@Remove {skill_cased}@")
    } else {
        match skill {
            Skill::GladesAncestralLight | Skill::InkwaterAncestralLight => {
                format!("#{skill_cased}#")
            }
            _ => format!("*{skill_cased}*"),
        }
    }
    .into();
    CommandString::Constant { value }
}
fn shard_string(shard: Shard, remove: bool) -> CommandString {
    let shard_cased = shard
        .to_string()
        .from_case(Case::Pascal)
        .to_case(Case::Title);
    let value = if remove {
        format!("@Remove {shard_cased}@")
    } else {
        format!("${shard_cased}$")
    }
    .into();
    CommandString::Constant { value }
}
fn teleporter_string(teleporter: Teleporter, remove: bool) -> CommandString {
    let value = if remove {
        format!("@Remove {teleporter}@")
    } else {
        format!("#{teleporter}#")
    }
    .into();
    CommandString::Constant { value }
}
fn clean_water_string(remove: bool) -> CommandString {
    let value = if remove {
        "@Remove Clean Water@"
    } else {
        "*Clean Water*"
    }
    .into();
    CommandString::Constant { value }
}
fn weapon_upgrade_string(weapon_upgrade: WeaponUpgrade, remove: bool) -> CommandString {
    let weapon_upgrade_cased = weapon_upgrade
        .to_string()
        .from_case(Case::Pascal)
        .to_case(Case::Title);
    let value = if remove {
        format!("@Remove {weapon_upgrade_cased}@")
    } else {
        format!("#{weapon_upgrade_cased}#")
    }
    .into();
    CommandString::Constant { value }
}

fn resource(string_fn: fn(bool) -> CommandString, uber_identifier: UberIdentifier) -> CommandVoid {
    CommandVoid::Multi {
        commands: vec![
            item_message(string_fn(false)),
            super::add_integer_value(uber_identifier, 1),
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
            super::add_integer_value(uber_identifier, -1),
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
        .map_or(false, |entry| entry.readonly)
    {
        context.compiler.errors.push(Error::custom(
            "this uberState is readonly".to_string(),
            span,
        ));
    }
    Some(Command::Void(command))
}

const SPIRIT_LIGHT_NAMES: &[&str] = &[
    "Spirit Light",
    "Gallons",
    "Spirit Bucks",
    "Gold",
    "Geo",
    "EXP",
    "Experience",
    "XP",
    "Gil",
    "GP",
    "Dollars",
    "Tokens",
    "Tickets",
    "Pounds Sterling",
    "Brownie Points",
    "Euros",
    "Credits",
    "Bells",
    "Fish",
    "Zenny",
    "Pesos",
    "Exalted Orbs",
    "Hryvnia",
    "Poké",
    "Glod",
    "Dollerydoos",
    "Boonbucks",
    "Pieces of Eight",
    "Shillings",
    "Farthings",
    "Kalganids",
    "Quatloos",
    "Crowns",
    "Solari",
    "Widgets",
    "Ori Money",
    "Money",
    "Cash",
    "Munny",
    "Nuyen",
    "Rings",
    "Rupees",
    "Coins",
    "Echoes",
    "Sovereigns",
    "Points",
    "Drams",
    "Doubloons",
    "Spheres",
    "Silver",
    "Slivers",
    "Rubies",
    "Emeralds",
    "Notes",
    "Yen",
    "Złoty",
    "Likes",
    "Comments",
    "Subs",
    "Bananas",
    "Sapphires",
    "Diamonds",
    "Fun",
    "Minerals",
    "Vespene Gas",
    "Sheep",
    "Brick",
    "Wheat",
    "Wood",
    "Quills",
    "Bits",
    "Bytes",
    "Nuts",
    "Bolts",
    "Souls",
    "Runes",
    "Pons",
    "Boxings",
    "Stonks",
    "Leaves",
    "Marbles",
    "Stamps",
    "Hugs",
    "Nobles",
    "Socks",
];
