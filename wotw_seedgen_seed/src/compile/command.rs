use super::{args::Args, compile_into_lookup, unwrap_string_placeholder, Compile};
use crate::assembly::Command;
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_seed_language::output::{
    self as input, CommandFloat, CommandVoid, Comparator, EqualityComparator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryUsed {
    pub boolean: usize,
    pub integer: usize,
    pub float: usize,
    pub string: usize,
}
impl MemoryUsed {
    pub const ZERO: Self = Self {
        boolean: 0,
        integer: 0,
        float: 0,
        string: 0,
    };

    pub fn combine(&mut self, other: Self) {
        self.boolean = usize::max(self.boolean, other.boolean);
        self.integer = usize::max(self.integer, other.integer);
        self.float = usize::max(self.float, other.float);
        self.string = usize::max(self.string, other.string);
    }
}

impl Compile for input::Command {
    type Output = (Vec<Command>, MemoryUsed);

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::Boolean(command) => command.compile(command_lookup),
            Self::Integer(command) => command.compile(command_lookup),
            Self::Float(command) => command.compile(command_lookup),
            Self::String(command) => command.compile(command_lookup),
            Self::Zone(command) => command.compile(command_lookup),
            Self::Void(command) => command.compile(command_lookup),
        }
    }
}

impl Compile for input::CommandBoolean {
    type Output = (Vec<Command>, MemoryUsed);

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::Constant { value } => (vec![Command::SetBoolean(value)], MemoryUsed::ZERO),
            Self::Multi { commands, last } => multi_with_return(commands, *last, command_lookup),
            Self::CompareBoolean { operation } => Args::new(2, command_lookup)
                .boolean(operation.left)
                .boolean(operation.right)
                .call(Command::CompareBoolean(operation.operator)),
            Self::CompareInteger { operation } => Args::new(2, command_lookup)
                .integer(operation.left)
                .integer(operation.right)
                .call(Command::CompareInteger(operation.operator)),
            Self::CompareFloat { operation } => Args::new(2, command_lookup)
                .float(operation.left)
                .float(operation.right)
                .call(Command::CompareFloat(operation.operator)),
            Self::CompareString { operation } => Args::new(2, command_lookup)
                .string(operation.left)
                .string(operation.right)
                .call(Command::CompareString(operation.operator)),
            Self::CompareZone { operation } => Args::new(2, command_lookup)
                .zone(operation.left)
                .zone(operation.right)
                .call(Command::CompareInteger(match operation.operator {
                    EqualityComparator::Equal => Comparator::Equal,
                    EqualityComparator::NotEqual => Comparator::NotEqual,
                })),
            Self::LogicOperation { operation } => Args::new(2, command_lookup)
                .boolean(operation.left)
                .boolean(operation.right)
                .call(Command::LogicOperation(operation.operator)),
            Self::FetchBoolean { uber_identifier } => (
                vec![Command::FetchBoolean(uber_identifier)],
                MemoryUsed::ZERO,
            ),
            Self::GetBoolean { id } => (vec![Command::CopyBoolean(id, 0)], MemoryUsed::ZERO),
            Self::IsInHitbox { x1, y1, x2, y2 } => Args::new(4, command_lookup)
                .float(*x1)
                .float(*y1)
                .float(*x2)
                .float(*y2)
                .call(Command::IsInHitbox),
        }
    }
}

impl Compile for input::CommandInteger {
    type Output = (Vec<Command>, MemoryUsed);

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::Constant { value } => (vec![Command::SetInteger(value)], MemoryUsed::ZERO),
            Self::Multi { commands, last } => multi_with_return(commands, *last, command_lookup),
            Self::Arithmetic { operation } => Args::new(2, command_lookup)
                .integer(operation.left)
                .integer(operation.right)
                .call(Command::ArithmeticInteger(operation.operator)),
            Self::FetchInteger { uber_identifier } => (
                vec![Command::FetchInteger(uber_identifier)],
                MemoryUsed::ZERO,
            ),
            Self::GetInteger { id } => (vec![Command::CopyInteger(id, 0)], MemoryUsed::ZERO),
            // TODO don't implicitely round
            Self::FromFloat { float } => {
                let mut commands = Args::new(1, command_lookup)
                    .float(*float)
                    .call(Command::Round);
                commands.0.push(Command::FloatToInteger);
                commands
            }
        }
    }
}

impl Compile for input::CommandFloat {
    type Output = (Vec<Command>, MemoryUsed);

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::Constant { value } => (vec![Command::SetFloat(value.into())], MemoryUsed::ZERO),
            Self::Multi { commands, last } => multi_with_return(commands, *last, command_lookup),
            Self::Arithmetic { operation } => Args::new(2, command_lookup)
                .float(operation.left)
                .float(operation.right)
                .call(Command::ArithmeticFloat(operation.operator)),
            Self::FetchFloat { uber_identifier } => {
                (vec![Command::FetchFloat(uber_identifier)], MemoryUsed::ZERO)
            }
            Self::GetFloat { id } => (vec![Command::CopyFloat(id, 0)], MemoryUsed::ZERO),
            Self::FromInteger { integer } => Args::new(1, command_lookup)
                .integer(*integer)
                .call(Command::IntegerToFloat),
        }
    }
}

impl Compile for input::CommandString {
    type Output = (Vec<Command>, MemoryUsed);

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::Constant { value } => (
                vec![Command::SetString(unwrap_string_placeholder(value))],
                MemoryUsed::ZERO,
            ),
            Self::Multi { commands, last } => multi_with_return(commands, *last, command_lookup),
            Self::Concatenate { left, right } => Args::new(2, command_lookup)
                .string(*left)
                .string(*right)
                .call(Command::Concatenate),
            Self::GetString { id } => (vec![Command::CopyString(id, 0)], MemoryUsed::ZERO),
            Self::WorldName { index } => (vec![Command::WorldName(index)], MemoryUsed::ZERO),
            Self::FromBoolean { boolean } => Args::new(1, command_lookup)
                .boolean(*boolean)
                .call(Command::BooleanToString),
            Self::FromInteger { integer } => Args::new(1, command_lookup)
                .integer(*integer)
                .call(Command::IntegerToString),
            Self::FromFloat { float } => Args::new(1, command_lookup)
                .float(*float)
                .call(Command::FloatToString),
        }
    }
}

impl Compile for input::CommandZone {
    type Output = (Vec<Command>, MemoryUsed);

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::Constant { value } => (vec![Command::SetInteger(value as i32)], MemoryUsed::ZERO),
            Self::Multi { commands, last } => multi_with_return(commands, *last, command_lookup),
            Self::CurrentZone {} => (
                vec![Command::FetchInteger(UberIdentifier::new(5, 50))],
                MemoryUsed::ZERO,
            ),
        }
    }
}

impl Compile for input::CommandVoid {
    type Output = (Vec<Command>, MemoryUsed);

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::Multi { commands } => multi(commands, command_lookup),
            Self::Lookup { index } => (vec![Command::Execute(index)], MemoryUsed::ZERO),
            Self::If { condition, command } => {
                let index = compile_into_lookup(*command, command_lookup);
                Args::new(1, command_lookup)
                    .boolean(condition)
                    .call(Command::ExecuteIf(index))
            }
            Self::QueuedMessage {
                id,
                priority,
                message,
                timeout,
            } => Args::new(2, command_lookup)
                .string(message)
                .float(timeout.unwrap_or(CommandFloat::Constant { value: (4.).into() })) // TODO what's the default timeout
                .call(Command::QueuedMessage(id, priority)),
            Self::FreeMessage { id, message } => {
                let mut commands = Args::new(1, command_lookup)
                    .string(message)
                    .call(Command::FreeMessage(id));
                commands.0.push(Command::FreeMessageShow(id));
                commands.0.push(Command::MessageText(id));
                commands
            }
            Self::MessageDestroy { id } => (vec![Command::MessageDestroy(id)], MemoryUsed::ZERO),
            Self::MessageText { id, message } => Args::new(1, command_lookup)
                .string(message)
                .call(Command::MessageText(id)),
            Self::MessageTimeout { id, timeout } => Args::new(1, command_lookup)
                .float(timeout)
                .call(Command::MessageTimeout(id)),
            Self::MessageBackground { id, background } => Args::new(1, command_lookup)
                .boolean(background)
                .call(Command::MessageBackground(id)),
            Self::FreeMessagePosition { id, x, y } => Args::new(2, command_lookup)
                .float(x)
                .float(y)
                .call(Command::FreeMessagePosition(id)),
            Self::FreeMessageAlignment { id, alignment } => (
                vec![Command::FreeMessageAlignment(id, alignment)],
                MemoryUsed::ZERO,
            ),
            Self::FreeMessageScreenPosition {
                id,
                screen_position,
            } => (
                vec![Command::FreeMessageScreenPosition(id, screen_position)],
                MemoryUsed::ZERO,
            ),
            Self::StoreBoolean {
                uber_identifier,
                value,
                trigger_events,
            } => Args::new(1, command_lookup)
                .boolean(value)
                .call(Command::StoreBoolean(uber_identifier, trigger_events)),
            Self::StoreInteger {
                uber_identifier,
                value,
                trigger_events,
            } => Args::new(1, command_lookup)
                .integer(value)
                .call(Command::StoreInteger(uber_identifier, trigger_events)),
            Self::StoreFloat {
                uber_identifier,
                value,
                trigger_events,
            } => Args::new(1, command_lookup)
                .float(value)
                .call(Command::StoreFloat(uber_identifier, trigger_events)),
            Self::SetBoolean { id, value } => Args::new(1, command_lookup)
                .boolean(value)
                .call(Command::CopyBoolean(0, id)),
            Self::SetInteger { id, value } => Args::new(1, command_lookup)
                .integer(value)
                .call(Command::CopyInteger(0, id)),
            Self::SetFloat { id, value } => Args::new(1, command_lookup)
                .float(value)
                .call(Command::CopyFloat(0, id)),
            Self::SetString { id, value } => Args::new(1, command_lookup)
                .string(value)
                .call(Command::CopyString(0, id)),
            Self::Save {} => (vec![Command::Save], MemoryUsed::ZERO),
            Self::Checkpoint {} => (vec![Command::Checkpoint], MemoryUsed::ZERO),
            Self::Warp { x, y } => Args::new(2, command_lookup)
                .float(x)
                .float(y)
                .call(Command::Warp),
            Self::Equip { slot, equipment } => {
                (vec![Command::Equip(slot, equipment)], MemoryUsed::ZERO)
            }
            Self::Unequip { equipment } => (vec![Command::Unequip(equipment)], MemoryUsed::ZERO),
            Self::TriggerKeybind { bind } => {
                (vec![Command::TriggerKeybind(bind)], MemoryUsed::ZERO)
            }
            Self::EnableServerSync { uber_identifier } => (
                vec![Command::EnableServerSync(uber_identifier)],
                MemoryUsed::ZERO,
            ),
            Self::DisableServerSync { uber_identifier } => (
                vec![Command::DisableServerSync(uber_identifier)],
                MemoryUsed::ZERO,
            ),
            Self::SetSpoilerMapIcon {
                location,
                icon,
                label,
            } => Args::new(1, command_lookup)
                .string(label)
                .call(Command::SetSpoilerMapIcon(location, icon)),
            Self::CreateWarpIcon { id, x, y } => Args::new(2, command_lookup)
                .float(x)
                .float(y)
                .call(Command::CreateWarpIcon(id)),
            Self::SetWarpIconLabel { id, label } => Args::new(1, command_lookup)
                .string(label)
                .call(Command::SetWarpIconLabel(id)),
            Self::DestroyWarpIcon { id } => (vec![Command::DestroyWarpIcon(id)], MemoryUsed::ZERO),
            Self::SetShopItemPrice {
                uber_identifier,
                price,
            } => Args::new(1, command_lookup)
                .integer(price)
                .call(Command::SetShopItemPrice(uber_identifier)),
            Self::SetShopItemName {
                uber_identifier,
                name,
            } => Args::new(1, command_lookup)
                .string(name)
                .call(Command::SetShopItemName(uber_identifier)),
            Self::SetShopItemDescription {
                uber_identifier,
                description,
            } => Args::new(1, command_lookup)
                .string(description)
                .call(Command::SetShopItemDescription(uber_identifier)),
            Self::SetShopItemIcon {
                uber_identifier,
                icon,
            } => (
                vec![Command::SetShopItemIcon(uber_identifier, icon)],
                MemoryUsed::ZERO,
            ),
            Self::SetShopItemHidden {
                uber_identifier,
                hidden,
            } => Args::new(1, command_lookup)
                .boolean(hidden)
                .call(Command::SetShopItemHidden(uber_identifier)),
            Self::SetShopItemLocked {
                uber_identifier,
                locked,
            } => Args::new(1, command_lookup)
                .boolean(locked)
                .call(Command::SetShopItemLocked(uber_identifier)),
            Self::SetWheelItemName {
                wheel,
                position,
                name,
            } => Args::new(1, command_lookup)
                .string(name)
                .call(Command::SetWheelItemName(wheel, position)),
            Self::SetWheelItemDescription {
                wheel,
                position,
                description,
            } => Args::new(1, command_lookup)
                .string(description)
                .call(Command::SetWheelItemDescription(wheel, position)),
            Self::SetWheelItemIcon {
                wheel,
                position,
                icon,
            } => (
                vec![Command::SetWheelItemIcon(wheel, position, icon)],
                MemoryUsed::ZERO,
            ),
            Self::SetWheelItemColor {
                wheel,
                position,
                red,
                green,
                blue,
                alpha,
            } => Args::new(4, command_lookup)
                .integer(red)
                .integer(green)
                .integer(blue)
                .integer(alpha)
                .call(Command::SetWheelItemColor(wheel, position)),
            Self::SetWheelItemAction {
                wheel,
                position,
                bind,
                action,
            } => (
                vec![Command::SetWheelItemCommand(wheel, position, bind, action)],
                MemoryUsed::ZERO,
            ),
            Self::DestroyWheelItem { wheel, position } => (
                vec![Command::DestroyWheelItem(wheel, position)],
                MemoryUsed::ZERO,
            ),
            Self::SwitchWheel { wheel } => (vec![Command::SwitchWheel(wheel)], MemoryUsed::ZERO),
            Self::SetWheelPinned { wheel, pinned } => Args::new(1, command_lookup)
                .boolean(pinned)
                .call(Command::SetWheelPinned(wheel)),
            Self::ClearAllWheels {} => (vec![Command::ClearAllWheels], MemoryUsed::ZERO),
        }
    }
}

fn multi(
    commands: Vec<CommandVoid>,
    command_lookup: &mut Vec<Vec<Command>>,
) -> (Vec<Command>, MemoryUsed) {
    // these commands don't return values, so we don't have to worry about commands overwriting previous results
    // since multis might be used in arguments, we still have to faithfully return the biggest amount of memory used at any point
    let mut memory_used = MemoryUsed::ZERO;
    let commands = commands
        .into_iter()
        .flat_map(|command| {
            let (commands, used) = command.compile(command_lookup);
            memory_used.combine(used);
            commands
        })
        .collect::<Vec<_>>();
    (commands, memory_used)
}
fn multi_with_return<T: Compile<Output = (Vec<Command>, MemoryUsed)>>(
    commands: Vec<CommandVoid>,
    last: T,
    command_lookup: &mut Vec<Vec<Command>>,
) -> (Vec<Command>, MemoryUsed) {
    let (mut commands, mut memory_used) = multi(commands, command_lookup);
    let (last, used) = last.compile(command_lookup);
    memory_used.combine(used);
    commands.extend(last);
    (commands, memory_used)
}
