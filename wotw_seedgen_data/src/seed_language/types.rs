use crate::{
    assets::{UberStateData, UberStateValue},
    seed_language::{
        ast::{
            Action, ConfigType, Constant, Expression, ExpressionValue, FunctionCall, Literal,
            Operation, Operator, UberStateType,
        },
        compile::{FunctionIdentifier, SnippetCompiler},
        output,
        token::Tokenizer,
    },
    UberIdentifier,
};
use serde::Deserialize;
use strum::{Display, VariantArray};
use wotw_seedgen_parse::{Ast, Identifier, Once, Spanned};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Display, VariantArray, Ast)]
pub enum Type {
    UberIdentifier,
    Boolean,
    Integer,
    Float,
    PlayerUberState,
    String,
    Action,
    Function,
    ClientEvent,
    Skill,
    Shard,
    Teleporter,
    WeaponUpgrade,
    Equipment,
    Zone,
    OpherIcon,
    LupoIcon,
    GromIcon,
    TuleyIcon,
    Icon,
    MapIcon,
    EquipSlot,
    WheelItemPosition,
    WheelBind,
    Alignment,
    HorizontalAnchor,
    VerticalAnchor,
    ScreenPosition,
    CoordinateSystem,
    Trigger,
    Expression,
    Void,
}

impl From<ConfigType> for Type {
    fn from(value: ConfigType) -> Self {
        match value {
            ConfigType::Boolean => Type::Boolean,
            ConfigType::Integer => Type::Integer,
            ConfigType::Float => Type::Float,
        }
    }
}

impl From<UberStateType> for Type {
    fn from(value: UberStateType) -> Self {
        match value {
            UberStateType::Boolean => Type::Boolean,
            UberStateType::Integer => Type::Integer,
            UberStateType::Float => Type::Float,
        }
    }
}

pub(crate) fn uber_state_type(
    uber_state_data: &UberStateData,
    uber_identifier: UberIdentifier,
) -> Option<UberStateType> {
    uber_state_data
        .id_lookup
        .get(&uber_identifier)
        .map(|meta| match meta.default_value {
            UberStateValue::Boolean(_) => UberStateType::Boolean,
            UberStateValue::Integer(_) => UberStateType::Integer,
            UberStateValue::Float(_) => UberStateType::Float,
        })
}

pub(crate) trait InferType {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type>;
}

impl<T: InferType> InferType for Spanned<T> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        self.data.infer_type(compiler)
    }
}

impl<T: InferType> InferType for Option<T> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        self.as_ref().and_then(|t| t.infer_type(compiler))
    }
}

impl<T: InferType> InferType for Box<T> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        (**self).infer_type(compiler)
    }
}

impl<T: InferType> InferType for Once<T> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        self.0.infer_type(compiler)
    }
}

impl InferType for Expression<'_> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        match self {
            Expression::Value(value) => value.infer_type(compiler),
            Expression::Operation(operation) => operation.infer_type(compiler),
        }
    }
}

impl InferType for ExpressionValue<'_> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        match self {
            ExpressionValue::Group(group) => group.content.infer_type(compiler),
            ExpressionValue::Action(action) => action.infer_type(compiler),
            ExpressionValue::Literal(literal) => Some(literal.data.literal_type()),
            ExpressionValue::Identifier(identifier) => identifier.infer_type(compiler),
        }
    }
}

impl InferType for Action<'_> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        match self {
            Action::Function(function) => function.infer_type(compiler),
            _ => Some(Type::Action),
        }
    }
}

impl InferType for FunctionCall<'_> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        if compiler
            .preprocessed
            .functions
            .contains(self.identifier.data.0)
        {
            return Some(Type::Void);
        }

        let identifier = self.identifier.data.0.parse().ok()?;

        let ty = match identifier {
            FunctionIdentifier::Fetch => self
                .parameters
                .content
                .as_ref()?
                .iter()
                .next()
                .and_then(|arg| arg.uber_state_type(compiler))
                .map(Type::from)?,
            FunctionIdentifier::GetBoolean | FunctionIdentifier::IsInBox => Type::Boolean,
            FunctionIdentifier::GetInteger | FunctionIdentifier::ToInteger => Type::Integer,
            FunctionIdentifier::GetFloat | FunctionIdentifier::ToFloat => Type::Float,
            FunctionIdentifier::GetString
            | FunctionIdentifier::ToString
            | FunctionIdentifier::SpiritLightString
            | FunctionIdentifier::RemoveSpiritLightString
            | FunctionIdentifier::GorlekOreString
            | FunctionIdentifier::RemoveGorlekOreString
            | FunctionIdentifier::KeystoneString
            | FunctionIdentifier::RemoveKeystoneString
            | FunctionIdentifier::ShardSlotString
            | FunctionIdentifier::RemoveShardSlotString
            | FunctionIdentifier::HealthFragmentString
            | FunctionIdentifier::RemoveHealthFragmentString
            | FunctionIdentifier::EnergyFragmentString
            | FunctionIdentifier::RemoveEnergyFragmentString
            | FunctionIdentifier::SkillString
            | FunctionIdentifier::RemoveSkillString
            | FunctionIdentifier::ShardString
            | FunctionIdentifier::RemoveShardString
            | FunctionIdentifier::TeleporterString
            | FunctionIdentifier::RemoveTeleporterString
            | FunctionIdentifier::CleanWaterString
            | FunctionIdentifier::RemoveCleanWaterString
            | FunctionIdentifier::WeaponUpgradeString
            | FunctionIdentifier::RemoveWeaponUpgradeString => Type::String,
            FunctionIdentifier::CurrentZone | FunctionIdentifier::CurrentMapZone => Type::Zone,
            FunctionIdentifier::SpiritLight
            | FunctionIdentifier::RemoveSpiritLight
            | FunctionIdentifier::GorlekOre
            | FunctionIdentifier::RemoveGorlekOre
            | FunctionIdentifier::Keystone
            | FunctionIdentifier::RemoveKeystone
            | FunctionIdentifier::ShardSlot
            | FunctionIdentifier::RemoveShardSlot
            | FunctionIdentifier::HealthFragment
            | FunctionIdentifier::RemoveHealthFragment
            | FunctionIdentifier::EnergyFragment
            | FunctionIdentifier::RemoveEnergyFragment
            | FunctionIdentifier::Skill
            | FunctionIdentifier::RemoveSkill
            | FunctionIdentifier::Shard
            | FunctionIdentifier::RemoveShard
            | FunctionIdentifier::Teleporter
            | FunctionIdentifier::RemoveTeleporter
            | FunctionIdentifier::CleanWater
            | FunctionIdentifier::RemoveCleanWater
            | FunctionIdentifier::WeaponUpgrade
            | FunctionIdentifier::RemoveWeaponUpgrade
            | FunctionIdentifier::ItemMessage
            | FunctionIdentifier::ItemMessageWithTimeout
            | FunctionIdentifier::PriorityMessage
            | FunctionIdentifier::PriorityMessageWithTimeout
            | FunctionIdentifier::ControlledPriorityMessage
            | FunctionIdentifier::FreeMessage
            | FunctionIdentifier::DestroyMessage
            | FunctionIdentifier::SetMessageText
            | FunctionIdentifier::SetMessageTimeout
            | FunctionIdentifier::SetMessageBackground
            | FunctionIdentifier::SetMessagePosition
            | FunctionIdentifier::SetMessageAlignment
            | FunctionIdentifier::SetMessageHorizontalAnchor
            | FunctionIdentifier::SetMessageVerticalAnchor
            | FunctionIdentifier::SetMessageScreenPosition
            | FunctionIdentifier::SetMessageBoxWidth
            | FunctionIdentifier::SetMessageCoordinateSystem
            | FunctionIdentifier::SetMapMessage
            | FunctionIdentifier::Store
            | FunctionIdentifier::StoreWithoutTriggers
            | FunctionIdentifier::StoreDefaults
            | FunctionIdentifier::StoreDefaultsExclude
            | FunctionIdentifier::SetString
            | FunctionIdentifier::SetBoolean
            | FunctionIdentifier::SetInteger
            | FunctionIdentifier::SetFloat
            | FunctionIdentifier::BoxTrigger
            | FunctionIdentifier::BoxTriggerDestroy
            | FunctionIdentifier::BoxTriggerEnterCallback
            | FunctionIdentifier::BoxTriggerLeaveCallback
            | FunctionIdentifier::Save
            | FunctionIdentifier::SaveToMemory
            | FunctionIdentifier::SaveAt
            | FunctionIdentifier::SaveToMemoryAt
            | FunctionIdentifier::Warp
            | FunctionIdentifier::Equip
            | FunctionIdentifier::Unequip
            | FunctionIdentifier::TriggerClientEvent
            | FunctionIdentifier::TriggerKeybind
            | FunctionIdentifier::EnableServerSync
            | FunctionIdentifier::DisableServerSync
            | FunctionIdentifier::CreateWarpIcon
            | FunctionIdentifier::SetWarpIconLabel
            | FunctionIdentifier::DestroyWarpIcon
            | FunctionIdentifier::SetShopItemData
            | FunctionIdentifier::SetShopItemPrice
            | FunctionIdentifier::SetShopItemName
            | FunctionIdentifier::SetShopItemDescription
            | FunctionIdentifier::SetShopItemIcon
            | FunctionIdentifier::SetShopItemHidden
            | FunctionIdentifier::SetShopItemLocked
            | FunctionIdentifier::SetWheelItemData
            | FunctionIdentifier::SetWheelItemName
            | FunctionIdentifier::SetWheelItemDescription
            | FunctionIdentifier::SetWheelItemIcon
            | FunctionIdentifier::SetWheelItemColor
            | FunctionIdentifier::SetWheelItemAction
            | FunctionIdentifier::DestroyWheelItem
            | FunctionIdentifier::SwitchWheel
            | FunctionIdentifier::SetWheelPinned
            | FunctionIdentifier::ResetAllWheels
            | FunctionIdentifier::CloseMenu
            | FunctionIdentifier::CloseWeaponWheel
            | FunctionIdentifier::DebugLog => Type::Void,
        };
        Some(ty)
    }
}

impl Expression<'_> {
    pub(crate) fn uber_state_type(&self, compiler: &mut SnippetCompiler) -> Option<UberStateType> {
        match self {
            Expression::Value(ExpressionValue::Literal(Spanned {
                data: Literal::UberIdentifier(uber_identifier),
                ..
            })) => {
                let uber_state = uber_identifier.resolve(compiler)?;

                if uber_state.value.is_none() {
                    return compiler.uber_state_type(uber_state.uber_identifier, uber_identifier);
                }
            }
            Expression::Value(ExpressionValue::Identifier(identifier)) => {
                let literal = compiler.resolve(identifier)?;

                if let output::Literal::UberIdentifier(uber_state) = literal {
                    if uber_state.value.is_none() {
                        let uber_identifier = uber_state.uber_identifier;
                        return compiler.uber_state_type(uber_identifier, identifier);
                    }
                }
            }
            _ => {}
        }

        panic!("Cannot determine UberState type of {self:?}")
    }
}

impl InferType for Spanned<Identifier<'_>> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        compiler.resolve(self).map(output::Literal::literal_type)
    }
}

impl Literal<'_> {
    pub(crate) fn literal_type(&self) -> Type {
        match self {
            Literal::UberIdentifier(_) => Type::UberIdentifier,
            Literal::Boolean(_) => Type::Boolean,
            Literal::Integer(_) => Type::Integer,
            Literal::Float(_) => Type::Float,
            Literal::String(_) => Type::String,
            Literal::Constant(constant) => constant.literal_type(),
        }
    }
}

impl output::Literal {
    pub(crate) fn literal_type(&self) -> Type {
        match self {
            output::Literal::UberIdentifier(uber_state) => match uber_state.value {
                None => Type::UberIdentifier,
                Some(_) => Type::Boolean,
            },
            output::Literal::Boolean(_) => Type::Boolean,
            output::Literal::Integer(_) => Type::Integer,
            output::Literal::Float(_) => Type::Float,
            output::Literal::String(_) => Type::String,
            output::Literal::Constant(constant) => constant.literal_type(),
            output::Literal::IconAsset(_) | output::Literal::CustomIcon(_) => Type::Icon,
        }
    }
}

impl Constant {
    pub(crate) fn literal_type(&self) -> Type {
        match self {
            Constant::ClientEvent(_) => Type::ClientEvent,
            Constant::Skill(_) => Type::Skill,
            Constant::Shard(_) => Type::Shard,
            Constant::Teleporter(_) => Type::Teleporter,
            Constant::WeaponUpgrade(_) => Type::WeaponUpgrade,
            Constant::Equipment(_) => Type::Equipment,
            Constant::Zone(_) => Type::Zone,
            Constant::OpherIcon(_) => Type::OpherIcon,
            Constant::LupoIcon(_) => Type::LupoIcon,
            Constant::GromIcon(_) => Type::GromIcon,
            Constant::TuleyIcon(_) => Type::TuleyIcon,
            Constant::MapIcon(_) => Type::MapIcon,
            Constant::EquipSlot(_) => Type::EquipSlot,
            Constant::WheelItemPosition(_) => Type::WheelItemPosition,
            Constant::WheelBind(_) => Type::WheelBind,
            Constant::Alignment(_) => Type::Alignment,
            Constant::HorizontalAnchor(_) => Type::HorizontalAnchor,
            Constant::VerticalAnchor(_) => Type::VerticalAnchor,
            Constant::ScreenPosition(_) => Type::ScreenPosition,
            Constant::CoordinateSystem(_) => Type::CoordinateSystem,
        }
    }
}

impl output::Command {
    pub(crate) fn command_type(&self) -> Type {
        match self {
            Self::Boolean(_) => Type::Boolean,
            Self::Integer(_) => Type::Integer,
            Self::Float(_) => Type::Float,
            Self::String(_) => Type::String,
            Self::Zone(_) => Type::Zone,
            Self::Void(_) => Type::Void,
        }
    }
}

impl InferType for Operation<'_> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        match self.operator.data {
            Operator::Arithmetic(_) => compiler.common_type(&self.left, &self.right),
            Operator::Logic(_) | Operator::Comparator(_) => Some(Type::Boolean),
        }
    }
}
