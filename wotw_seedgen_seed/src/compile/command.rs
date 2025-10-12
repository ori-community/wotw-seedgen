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
            Self::CompareBoolean { operation } => Args::new(command_lookup)
                .boolean(0, operation.left)
                .boolean(1, operation.right)
                .call(Command::CompareBoolean(operation.operator)),
            Self::CompareInteger { operation } => Args::new(command_lookup)
                .integer(0, operation.left)
                .integer(1, operation.right)
                .call(Command::CompareInteger(operation.operator)),
            Self::CompareFloat { operation } => Args::new(command_lookup)
                .float(0, operation.left)
                .float(1, operation.right)
                .call(Command::CompareFloat(operation.operator)),
            Self::CompareString { operation } => Args::new(command_lookup)
                .string(0, operation.left)
                .string(1, operation.right)
                .call(Command::CompareString(operation.operator)),
            Self::CompareZone { operation } => Args::new(command_lookup)
                .zone(0, operation.left)
                .zone(1, operation.right)
                .call(Command::CompareInteger(match operation.operator {
                    EqualityComparator::Equal => Comparator::Equal,
                    EqualityComparator::NotEqual => Comparator::NotEqual,
                })),
            Self::LogicOperation { operation } => Args::new(command_lookup)
                .boolean(0, operation.left)
                .boolean(1, operation.right)
                .call(Command::LogicOperation(operation.operator)),
            Self::FetchBoolean { uber_identifier } => (
                vec![Command::FetchBoolean(uber_identifier)],
                MemoryUsed::ZERO,
            ),
            Self::GetBoolean { id } => (vec![Command::CopyBoolean(id, 0)], MemoryUsed::ZERO),
            Self::IsInHitbox { x1, y1, x2, y2 } => Args::new(command_lookup)
                .float(0, *x1)
                .float(1, *y1)
                .float(2, *x2)
                .float(3, *y2)
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
            Self::Arithmetic { operation } => Args::new(command_lookup)
                .integer(0, operation.left)
                .integer(1, operation.right)
                .call(Command::ArithmeticInteger(operation.operator)),
            Self::FetchInteger { uber_identifier } => (
                vec![Command::FetchInteger(uber_identifier)],
                MemoryUsed::ZERO,
            ),
            Self::GetInteger { id } => (vec![Command::CopyInteger(id, 0)], MemoryUsed::ZERO),
            // TODO don't implicitely round
            Self::FromFloat { float } => {
                let mut commands = Args::new(command_lookup)
                    .float(0, *float)
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
            Self::Arithmetic { operation } => Args::new(command_lookup)
                .float(0, operation.left)
                .float(1, operation.right)
                .call(Command::ArithmeticFloat(operation.operator)),
            Self::FetchFloat { uber_identifier } => {
                (vec![Command::FetchFloat(uber_identifier)], MemoryUsed::ZERO)
            }
            Self::GetFloat { id } => (vec![Command::CopyFloat(id, 0)], MemoryUsed::ZERO),
            Self::FromInteger { integer } => Args::new(command_lookup)
                .integer(0, *integer)
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
            Self::Concatenate { left, right } => Args::new(command_lookup)
                .string(0, *left)
                .string(1, *right)
                .call(Command::Concatenate),
            Self::GetString { id } => (vec![Command::CopyString(id, 0)], MemoryUsed::ZERO),
            Self::WorldName { index } => (vec![Command::WorldName(index)], MemoryUsed::ZERO),
            Self::FromBoolean { boolean } => Args::new(command_lookup)
                .boolean(0, *boolean)
                .call(Command::BooleanToString),
            Self::FromInteger { integer } => Args::new(command_lookup)
                .integer(0, *integer)
                .call(Command::IntegerToString),
            Self::FromFloat { float } => Args::new(command_lookup)
                .float(0, *float)
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
            Self::CurrentMapZone {} => (
                vec![Command::FetchInteger(UberIdentifier::new(5, 51))],
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
                Args::new(command_lookup)
                    .boolean(0, condition)
                    .call(Command::ExecuteIf(index))
            }
            Self::DefineTimer { toggle, timer } => {
                (vec![Command::DefineTimer(toggle, timer)], MemoryUsed::ZERO)
            }
            Self::QueuedMessage {
                id,
                priority,
                message,
                timeout,
            } => Args::new(command_lookup)
                .string(0, message)
                .float(
                    0,
                    timeout.unwrap_or(CommandFloat::Constant { value: (4.).into() }),
                ) // TODO what's the default timeout
                .call(Command::QueuedMessage(id, priority)),
            Self::FreeMessage { id, message } => {
                let mut commands = Args::new(command_lookup)
                    .string(0, message)
                    .call(Command::FreeMessage(id));
                commands.0.push(Command::FreeMessageShow(id));
                commands.0.push(Command::MessageText(id)); // TODO seems more intuitive the other way around?
                commands
            }
            Self::MessageDestroy { id } => (vec![Command::MessageDestroy(id)], MemoryUsed::ZERO),
            Self::MessageText { id, message } => Args::new(command_lookup)
                .string(0, message)
                .call(Command::MessageText(id)),
            Self::MessageTimeout { id, timeout } => Args::new(command_lookup)
                .float(0, timeout)
                .call(Command::MessageTimeout(id)),
            Self::MessageBackground { id, background } => Args::new(command_lookup)
                .boolean(0, background)
                .call(Command::MessageBackground(id)),
            Self::FreeMessagePosition { id, x, y } => Args::new(command_lookup)
                .float(0, x)
                .float(1, y)
                .call(Command::FreeMessagePosition(id)),
            Self::FreeMessageAlignment { id, alignment } => (
                vec![Command::FreeMessageAlignment(id, alignment)],
                MemoryUsed::ZERO,
            ),
            CommandVoid::FreeMessageHorizontalAnchor {
                id,
                horizontal_anchor,
            } => (
                vec![Command::FreeMessageHorizontalAnchor(id, horizontal_anchor)],
                MemoryUsed::ZERO,
            ),
            CommandVoid::FreeMessageVerticalAnchor {
                id,
                vertical_anchor,
            } => (
                vec![Command::FreeMessageVerticalAnchor(id, vertical_anchor)],
                MemoryUsed::ZERO,
            ),
            CommandVoid::FreeMessageBoxWidth { id, width } => Args::new(command_lookup)
                .float(0, width)
                .call(Command::FreeMessageBoxWidth(id)),
            CommandVoid::FreeMessageCoordinateSystem {
                id,
                coordinate_system,
            } => (
                vec![Command::FreeMessageCoordinateSystem(id, coordinate_system)],
                MemoryUsed::ZERO,
            ),
            Self::SetMapMessage { value } => Args::new(command_lookup)
                .string(0, value)
                .call(Command::SetMapMessage),
            Self::StoreBoolean {
                uber_identifier,
                value,
                trigger_events,
            } => Args::new(command_lookup)
                .boolean(0, value)
                .call(Command::StoreBoolean(uber_identifier, trigger_events)),
            Self::StoreInteger {
                uber_identifier,
                value,
                trigger_events,
            } => Args::new(command_lookup)
                .integer(0, value)
                .call(Command::StoreInteger(uber_identifier, trigger_events)),
            Self::StoreFloat {
                uber_identifier,
                value,
                trigger_events,
            } => Args::new(command_lookup)
                .float(0, value)
                .call(Command::StoreFloat(uber_identifier, trigger_events)),
            Self::SetBoolean { id, value } => Args::new(command_lookup)
                .boolean(0, value)
                .call(Command::CopyBoolean(0, id)),
            Self::SetInteger { id, value } => Args::new(command_lookup)
                .integer(0, value)
                .call(Command::CopyInteger(0, id)),
            Self::SetFloat { id, value } => Args::new(command_lookup)
                .float(0, value)
                .call(Command::CopyFloat(0, id)),
            Self::SetString { id, value } => Args::new(command_lookup)
                .string(0, value)
                .call(Command::CopyString(0, id)),
            Self::Save {} => (vec![Command::Save], MemoryUsed::ZERO),
            Self::SaveToMemory {} => (vec![Command::SaveToMemory], MemoryUsed::ZERO),
            Self::Warp { x, y } => Args::new(command_lookup)
                .float(0, x)
                .float(1, y)
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
            } => Args::new(command_lookup)
                .string(0, label)
                .call(Command::SetSpoilerMapIcon(location, icon)),
            Self::CreateWarpIcon { id, x, y } => Args::new(command_lookup)
                .float(0, x)
                .float(1, y)
                .call(Command::CreateWarpIcon(id)),
            Self::SetWarpIconLabel { id, label } => Args::new(command_lookup)
                .string(0, label)
                .call(Command::SetWarpIconLabel(id)),
            Self::DestroyWarpIcon { id } => (vec![Command::DestroyWarpIcon(id)], MemoryUsed::ZERO),
            Self::SetShopItemPrice {
                uber_identifier,
                price,
            } => Args::new(command_lookup)
                .integer(0, price)
                .call(Command::SetShopItemPrice(uber_identifier)),
            Self::SetShopItemName {
                uber_identifier,
                name,
            } => Args::new(command_lookup)
                .string(0, name)
                .call(Command::SetShopItemName(uber_identifier)),
            Self::SetShopItemDescription {
                uber_identifier,
                description,
            } => Args::new(command_lookup)
                .string(0, description)
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
            } => Args::new(command_lookup)
                .boolean(0, hidden)
                .call(Command::SetShopItemHidden(uber_identifier)),
            Self::SetShopItemLocked {
                uber_identifier,
                locked,
            } => Args::new(command_lookup)
                .boolean(0, locked)
                .call(Command::SetShopItemLocked(uber_identifier)),
            Self::SetWheelItemName {
                wheel,
                position,
                name,
            } => Args::new(command_lookup)
                .string(0, name)
                .call(Command::SetWheelItemName(wheel, position)),
            Self::SetWheelItemDescription {
                wheel,
                position,
                description,
            } => Args::new(command_lookup)
                .string(0, description)
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
            } => Args::new(command_lookup)
                .integer(0, red)
                .integer(1, green)
                .integer(2, blue)
                .integer(3, alpha)
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
            Self::SetWheelPinned { wheel, pinned } => Args::new(command_lookup)
                .boolean(0, pinned)
                .call(Command::SetWheelPinned(wheel)),
            Self::ResetAllWheels {} => (vec![Command::ResetAllWheels], MemoryUsed::ZERO),
            Self::DebugLog { message } => Args::new(command_lookup)
                .string(0, message)
                .call(Command::DebugLog),
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
