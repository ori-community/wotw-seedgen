use crate::{
    ast::{
        Action, ConfigType, Constant, Expression, ExpressionValue, FunctionCall, Literal,
        Operation, Operator, UberStateType,
    },
    compile::{FunctionIdentifier, SnippetCompiler},
    output::{self, ConstantDiscriminants},
    token::Tokenizer,
};
use serde::Deserialize;
use strum::{Display, VariantArray};
use wotw_seedgen_assets::{UberStateData, UberStateValue};
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_parse::{Ast, Identifier, Once, Result, Spanned};

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

impl<T: InferType> InferType for Result<T> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        self.as_ref().ok().and_then(|t| t.infer_type(compiler))
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
            ExpressionValue::Literal(literal) => literal.infer_type(compiler),
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
                .as_ref()
                .ok()?
                .iter()
                .next()
                .and_then(|arg| arg.uber_state_type(compiler))
                .map(Type::from)?,
            FunctionIdentifier::GetBoolean | FunctionIdentifier::IsInHitbox => Type::Boolean,
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
            | FunctionIdentifier::WeaponUpgradeString => Type::String,
            FunctionIdentifier::RemoveWeaponUpgradeString => Type::String,
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
            | FunctionIdentifier::Save
            | FunctionIdentifier::SaveToMemory
            | FunctionIdentifier::Warp
            | FunctionIdentifier::Equip
            | FunctionIdentifier::Unequip
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

                match uber_state.value {
                    None => compiler.uber_state_type(uber_state.uber_identifier, uber_identifier),
                    Some(_) => None,
                }
            }
            Expression::Value(ExpressionValue::Identifier(identifier)) => {
                match compiler.resolve(identifier)? {
                    output::Literal::UberIdentifier(uber_state) => match uber_state.value {
                        None => {
                            let uber_identifier = uber_state.uber_identifier;
                            compiler.uber_state_type(uber_identifier, identifier)
                        }
                        Some(_) => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl InferType for Literal<'_> {
    // TODO unused, not sure any infertype implementation here is used...
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        match self {
            Literal::UberIdentifier(_) => Some(Type::UberIdentifier),
            Literal::Boolean(_) => Some(Type::Boolean),
            Literal::Integer(_) => Some(Type::Integer),
            Literal::Float(_) => Some(Type::Float),
            Literal::String(_) => Some(Type::String),
            Literal::Constant(constant) => constant.infer_type(compiler),
        }
    }
}

impl InferType for Constant<'_> {
    fn infer_type(&self, _compiler: &mut SnippetCompiler) -> Option<Type> {
        self.kind
            .data
            .0
            .parse()
            .ok()
            .map(|discriminant| match discriminant {
                ConstantDiscriminants::Skill => Type::Skill,
                ConstantDiscriminants::Shard => Type::Shard,
                ConstantDiscriminants::Teleporter => Type::Teleporter,
                ConstantDiscriminants::WeaponUpgrade => Type::WeaponUpgrade,
                ConstantDiscriminants::Equipment => Type::Equipment,
                ConstantDiscriminants::Zone => Type::Zone,
                ConstantDiscriminants::OpherIcon => Type::OpherIcon,
                ConstantDiscriminants::LupoIcon => Type::LupoIcon,
                ConstantDiscriminants::GromIcon => Type::GromIcon,
                ConstantDiscriminants::TuleyIcon => Type::TuleyIcon,
                ConstantDiscriminants::MapIcon => Type::MapIcon,
                ConstantDiscriminants::EquipSlot => Type::EquipSlot,
                ConstantDiscriminants::WheelItemPosition => Type::WheelItemPosition,
                ConstantDiscriminants::WheelBind => Type::WheelBind,
                ConstantDiscriminants::Alignment => Type::Alignment,
                ConstantDiscriminants::HorizontalAnchor => Type::HorizontalAnchor,
                ConstantDiscriminants::VerticalAnchor => Type::VerticalAnchor,
                ConstantDiscriminants::ScreenPosition => Type::ScreenPosition,
                ConstantDiscriminants::CoordinateSystem => Type::CoordinateSystem,
            })
    }
}

impl InferType for Spanned<Identifier<'_>> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        compiler.resolve(self).map(output::Literal::literal_type)
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

impl output::Constant {
    pub(crate) fn literal_type(&self) -> Type {
        match self {
            output::Constant::Skill(_) => Type::Skill,
            output::Constant::Shard(_) => Type::Shard,
            output::Constant::Teleporter(_) => Type::Teleporter,
            output::Constant::WeaponUpgrade(_) => Type::WeaponUpgrade,
            output::Constant::Equipment(_) => Type::Equipment,
            output::Constant::Zone(_) => Type::Zone,
            output::Constant::OpherIcon(_) => Type::OpherIcon,
            output::Constant::LupoIcon(_) => Type::LupoIcon,
            output::Constant::GromIcon(_) => Type::GromIcon,
            output::Constant::TuleyIcon(_) => Type::TuleyIcon,
            output::Constant::MapIcon(_) => Type::MapIcon,
            output::Constant::EquipSlot(_) => Type::EquipSlot,
            output::Constant::WheelItemPosition(_) => Type::WheelItemPosition,
            output::Constant::WheelBind(_) => Type::WheelBind,
            output::Constant::Alignment(_) => Type::Alignment,
            output::Constant::HorizontalAnchor(_) => Type::HorizontalAnchor,
            output::Constant::VerticalAnchor(_) => Type::VerticalAnchor,
            output::Constant::ScreenPosition(_) => Type::ScreenPosition,
            output::Constant::CoordinateSystem(_) => Type::CoordinateSystem,
        }
    }
}

impl InferType for Operation<'_> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        match self.operator.data {
            Operator::Arithmetic(_) => self.left.infer_type(compiler),
            Operator::Logic(_) | Operator::Comparator(_) => Some(Type::Boolean),
        }
    }
}
