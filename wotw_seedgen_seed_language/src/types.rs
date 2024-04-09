use crate::{
    ast::{
        Action, Constant, Expression, ExpressionValue, FunctionCall, Literal, Operation, Operator,
        UberStateType,
    },
    compile::{FunctionIdentifier, SnippetCompiler},
    output::intermediate::{self, ConstantDiscriminants},
    token::Tokenizer,
};
use serde::Deserialize;
use strum::Display;
use wotw_seedgen_assets::{UberStateData, UberStateValue};
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_parse::{Ast, Identifier, Once, Result, Spanned};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Display, Ast)]
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
    ScreenPosition,
    Trigger,
    Expression,
    Void,
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

pub fn uber_state_type(
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
            FunctionIdentifier::CurrentZone => Type::Zone,
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
            | FunctionIdentifier::SetMessageScreenPosition
            | FunctionIdentifier::Store
            | FunctionIdentifier::StoreWithoutTriggers
            | FunctionIdentifier::SetString
            | FunctionIdentifier::SetBoolean
            | FunctionIdentifier::SetInteger
            | FunctionIdentifier::SetFloat
            | FunctionIdentifier::Save
            | FunctionIdentifier::Checkpoint
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
            | FunctionIdentifier::ClearAllWheels => Type::Void,
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
                    intermediate::Literal::UberIdentifier(uber_state) => match uber_state.value {
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
                ConstantDiscriminants::ScreenPosition => Type::ScreenPosition,
            })
    }
}
impl InferType for Spanned<Identifier<'_>> {
    fn infer_type(&self, compiler: &mut SnippetCompiler) -> Option<Type> {
        compiler
            .resolve(self)
            .map(intermediate::Literal::literal_type)
    }
}
impl intermediate::Literal {
    pub fn literal_type(&self) -> Type {
        match self {
            intermediate::Literal::UberIdentifier(uber_state) => match uber_state.value {
                None => Type::UberIdentifier,
                Some(_) => Type::Boolean,
            },
            intermediate::Literal::Boolean(_) => Type::Boolean,
            intermediate::Literal::Integer(_) => Type::Integer,
            intermediate::Literal::Float(_) => Type::Float,
            intermediate::Literal::String(_) => Type::String,
            intermediate::Literal::Constant(constant) => constant.literal_type(),
            intermediate::Literal::IconAsset(_) | intermediate::Literal::CustomIcon(_) => {
                Type::Icon
            }
        }
    }
}
impl intermediate::Constant {
    pub fn literal_type(&self) -> Type {
        match self {
            intermediate::Constant::Skill(_) => Type::Skill,
            intermediate::Constant::Shard(_) => Type::Shard,
            intermediate::Constant::Teleporter(_) => Type::Teleporter,
            intermediate::Constant::WeaponUpgrade(_) => Type::WeaponUpgrade,
            intermediate::Constant::Equipment(_) => Type::Equipment,
            intermediate::Constant::Zone(_) => Type::Zone,
            intermediate::Constant::OpherIcon(_) => Type::OpherIcon,
            intermediate::Constant::LupoIcon(_) => Type::LupoIcon,
            intermediate::Constant::GromIcon(_) => Type::GromIcon,
            intermediate::Constant::TuleyIcon(_) => Type::TuleyIcon,
            intermediate::Constant::MapIcon(_) => Type::MapIcon,
            intermediate::Constant::EquipSlot(_) => Type::EquipSlot,
            intermediate::Constant::WheelItemPosition(_) => Type::WheelItemPosition,
            intermediate::Constant::WheelBind(_) => Type::WheelBind,
            intermediate::Constant::Alignment(_) => Type::Alignment,
            intermediate::Constant::ScreenPosition(_) => Type::ScreenPosition,
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

pub fn common_type(left: Type, right: Type) -> Option<Type> {
    if left == right {
        return Some(left);
    }
    match (left, right) {
        (Type::UberIdentifier, value @ (Type::Boolean | Type::Integer | Type::Float))
        | (value @ (Type::Boolean | Type::Integer | Type::Float), Type::UberIdentifier) => {
            Some(value)
        }
        (Type::Integer, Type::Float) | (Type::Float, Type::Integer) => Some(Type::Float),
        (_, Type::String) | (Type::String, _) => Some(Type::String),
        _ => None,
    }
}
