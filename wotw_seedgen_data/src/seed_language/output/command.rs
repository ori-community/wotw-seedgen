use super::{
    ArithmeticOperator, Comparator, EqualityComparator, Icon, LogicOperator, Operation,
    StringOrPlaceholder,
};
use crate::{
    seed_language::{ast::ClientEvent, output::operation::Concatenator},
    Alignment, CoordinateSystem, EquipSlot, Equipment, HorizontalAnchor, MapIcon, UberIdentifier,
    VerticalAnchor, WheelBind, WheelItemPosition, Zone,
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

/// A Command, which may be used to affect the world, player or client state
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    /// Commands returning [`bool`]
    Boolean(CommandBoolean),
    /// Commands returning [`i32`]
    Integer(CommandInteger),
    /// Commands returning [`f32`]
    Float(CommandFloat),
    /// Commands returning [`StringOrPlaceholder`]
    String(CommandString),
    /// Commands returning [`Zone`]
    Zone(CommandZone),
    /// Commands returning nothing
    Void(CommandVoid),
}

pub trait AsConstant {
    type Output;

    fn as_constant(&self) -> Option<&Self::Output>;
}

/// Command which returns [`bool`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandBoolean {
    /// Return `value`
    Constant { value: bool },
    /// Execute `commands`, then use `last` for the return value
    Multi {
        commands: Vec<CommandVoid>,
        last: Box<CommandBoolean>,
    },
    /// Return the result of `operation`
    CompareBoolean {
        operation: Box<Operation<CommandBoolean, EqualityComparator>>,
    },
    /// Return the result of `operation`
    CompareInteger {
        operation: Box<Operation<CommandInteger, Comparator>>,
    },
    /// Return the result of `operation`
    CompareFloat {
        operation: Box<Operation<CommandFloat, Comparator>>,
    },
    /// Return the result of `operation`
    CompareString {
        operation: Box<Operation<CommandString, EqualityComparator>>,
    },
    /// Return the result of `operation`
    CompareZone {
        operation: Box<Operation<CommandZone, EqualityComparator>>,
    },
    /// Return the result of `operation`
    LogicOperation {
        operation: Box<Operation<CommandBoolean, LogicOperator>>,
    },
    /// Return the value stored in `uber_identifier`
    // TODO could there be a better naming convention than fetch vs get etc.?
    FetchBoolean { uber_identifier: UberIdentifier },
    /// Get the value stored under `id`
    GetBoolean { id: usize },
    // TODO some kind of lint if things like this appear in trigger conditions
    /// Check if Ori is in box `id`
    IsInBox {
        x1: Box<CommandFloat>,
        y1: Box<CommandFloat>,
        x2: Box<CommandFloat>,
        y2: Box<CommandFloat>,
    },
}

impl CommandBoolean {
    pub fn loc_data_condition(uber_identifier: UberIdentifier, value: Option<i32>) -> Self {
        Self::condition_with_operator(uber_identifier, value, Comparator::GreaterOrEqual)
    }

    pub fn door_condition(uber_identifier: UberIdentifier, value: Option<i32>) -> Self {
        Self::condition_with_operator(uber_identifier, value, Comparator::Equal)
    }

    fn condition_with_operator(
        uber_identifier: UberIdentifier,
        value: Option<i32>,
        operator: Comparator,
    ) -> Self {
        match value {
            None => CommandBoolean::FetchBoolean { uber_identifier },
            Some(value) => CommandBoolean::CompareInteger {
                operation: Box::new(Operation {
                    left: CommandInteger::FetchInteger { uber_identifier },
                    operator,
                    right: value.into(),
                }),
            },
        }
    }
}

impl AsConstant for CommandBoolean {
    type Output = bool;

    fn as_constant(&self) -> Option<&Self::Output> {
        match self {
            Self::Constant { value } => Some(value),
            _ => None,
        }
    }
}

impl From<bool> for CommandBoolean {
    fn from(value: bool) -> Self {
        Self::Constant { value }
    }
}

/// Command which returns [`i32`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandInteger {
    /// Return `value`
    Constant { value: i32 },
    /// Execute `commands`, then use `last` for the return value
    Multi {
        commands: Vec<CommandVoid>,
        last: Box<CommandInteger>,
    },
    /// Return the result of `operation`
    Arithmetic {
        operation: Box<Operation<CommandInteger, ArithmeticOperator>>,
    },
    /// Return the value stored in `uber_identifier`
    // TODO these could just be called "Fetch"? This is redundant with the type name
    FetchInteger { uber_identifier: UberIdentifier },
    /// Get the value stored under `id`
    GetInteger { id: usize },
    /// Convert `float` to `f32`
    FromFloat { float: Box<CommandFloat> },
}

impl AsConstant for CommandInteger {
    type Output = i32;

    fn as_constant(&self) -> Option<&Self::Output> {
        match self {
            Self::Constant { value } => Some(value),
            _ => None,
        }
    }
}

impl From<i32> for CommandInteger {
    fn from(value: i32) -> Self {
        Self::Constant { value }
    }
}

/// Command which returns [`f32`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandFloat {
    /// Return `value`
    Constant { value: OrderedFloat<f32> },
    /// Execute `commands`, then use `last` for the return value
    Multi {
        commands: Vec<CommandVoid>,
        last: Box<CommandFloat>,
    },
    /// Return the result of `operation`
    Arithmetic {
        operation: Box<Operation<CommandFloat, ArithmeticOperator>>,
    },
    /// Return the value stored in `uber_identifier`
    FetchFloat { uber_identifier: UberIdentifier },
    /// Get the value stored under `id`
    GetFloat { id: usize },
    /// Convert `integer` to `f32`
    FromInteger { integer: Box<CommandInteger> },
}

impl AsConstant for CommandFloat {
    type Output = OrderedFloat<f32>;

    fn as_constant(&self) -> Option<&Self::Output> {
        match self {
            Self::Constant { value } => Some(value),
            _ => None,
        }
    }
}

impl From<OrderedFloat<f32>> for CommandFloat {
    fn from(value: OrderedFloat<f32>) -> Self {
        Self::Constant { value }
    }
}

impl From<f32> for CommandFloat {
    fn from(value: f32) -> Self {
        OrderedFloat::from(value).into()
    }
}

/// Command which returns [`StringOrPlaceholder`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandString {
    /// Return `value`
    Constant { value: StringOrPlaceholder },
    /// Execute `commands`, then use `last` for the return value
    Multi {
        commands: Vec<CommandVoid>,
        last: Box<CommandString>,
    },
    /// Return a String consisting of `left`, then `right`
    Concatenate {
        operation: Box<Operation<CommandString, Concatenator>>,
    },
    /// Get the value stored under `id`
    GetString { id: usize },
    /// Return the name of world number `index`
    WorldName { index: usize },
    /// Convert `boolean` to `String`
    FromBoolean { boolean: Box<CommandBoolean> },
    /// Convert `integer` to `String`
    FromInteger { integer: Box<CommandInteger> },
    /// Convert `float` to `String`
    FromFloat { float: Box<CommandFloat> },
}

impl AsConstant for CommandString {
    type Output = String;

    fn as_constant(&self) -> Option<&Self::Output> {
        match self {
            Self::Constant {
                value: StringOrPlaceholder::Value(value),
            } => Some(value),
            _ => None,
        }
    }
}

impl From<StringOrPlaceholder> for CommandString {
    fn from(value: StringOrPlaceholder) -> Self {
        Self::Constant { value }
    }
}

impl From<String> for CommandString {
    fn from(value: String) -> Self {
        StringOrPlaceholder::Value(value).into()
    }
}

impl From<&str> for CommandString {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

/// Command which returns [`Zone`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandZone {
    /// Return `value`
    Constant { value: Zone },
    /// Execute `commands`, then use `last` for the return value
    Multi {
        commands: Vec<CommandVoid>,
        last: Box<CommandZone>,
    },
    /// Return the zone Ori is currently in
    CurrentZone {},
    /// Return the zone currently selected in the map
    CurrentMapZone {},
}

impl AsConstant for CommandZone {
    type Output = Zone;

    fn as_constant(&self) -> Option<&Self::Output> {
        match self {
            Self::Constant { value } => Some(value),
            _ => None,
        }
    }
}

impl From<Zone> for CommandZone {
    fn from(value: Zone) -> Self {
        Self::Constant { value }
    }
}

/// Command which returns nothing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandVoid {
    /// Execute `commands`
    Multi {
        commands: Vec<CommandVoid>,
    },
    /// Lookup and perform the action at `index`
    Lookup {
        index: usize,
    },
    /// Only perform `command` if `condition` evaluates to true
    If {
        condition: CommandBoolean,
        command: Box<CommandVoid>,
    },
    /// Until the next reload, on every tick where `toggle` is `true` increment `timer` by the delta time in seconds
    DefineTimer {
        toggle: UberIdentifier,
        timer: UberIdentifier,
    },
    /// Add `message` to the queue with `timeout` or a default timeout.
    /// If `priority` is true, it should be a priority message.
    /// If `id` is specified, it can later be used to update the message
    QueuedMessage {
        id: Option<usize>,
        priority: bool,
        message: CommandString,
        timeout: Option<CommandFloat>,
    },
    // TODO
    // /// Update the `callback` that triggers when queued message `id` is shown
    // QueuedMessageShownCallback { id: usize, callback: usize },
    // /// Update the `callback` that triggers when queued message `id` is hidden
    // QueuedMessageHiddenCallback { id: usize, callback: usize },
    /// Show `message` immediately independent of the queue
    FreeMessage {
        id: usize,
        message: CommandString,
    },
    /// DESTROY message `id`
    MessageDestroy {
        id: usize,
    },
    /// Update the `message` of message `id`
    MessageText {
        id: usize,
        message: CommandString,
    },
    /// Update the `timeout` of message `id`
    MessageTimeout {
        id: usize,
        timeout: CommandFloat,
    },
    /// Update whether message `id` has a `background`
    MessageBackground {
        id: usize,
        background: CommandBoolean,
    },
    // TODO
    // /// Show free message `id`
    // FreeMessageShow { id: usize },
    // /// Hide free message `id`
    // FreeMessageHide { id: usize },
    // TODO world coordinates
    /// Update the `position` of free message `id`
    FreeMessagePosition {
        id: usize,
        x: CommandFloat,
        y: CommandFloat,
    },
    /// Update the `alignment` of free message `id`
    FreeMessageAlignment {
        id: usize,
        alignment: Alignment,
    },
    /// Set the `horizontal_anchor` of free message `id`
    FreeMessageHorizontalAnchor {
        id: usize,
        horizontal_anchor: HorizontalAnchor,
    },
    /// Set the `vertical_anchor` of free message `id`
    FreeMessageVerticalAnchor {
        id: usize,
        vertical_anchor: VerticalAnchor,
    },
    FreeMessageBoxWidth {
        id: usize,
        width: CommandFloat,
    },
    FreeMessageCoordinateSystem {
        id: usize,
        coordinate_system: CoordinateSystem,
    },
    /// Sets the map message content to `value`
    SetMapMessage {
        value: CommandString,
    },
    /// Store `value` in `uber_identifier` and check if any events are triggered
    StoreBoolean {
        uber_identifier: UberIdentifier,
        value: CommandBoolean,
        trigger_events: bool,
    },
    /// Store `value` in `uber_identifier` and check if any events are triggered
    StoreInteger {
        uber_identifier: UberIdentifier,
        value: CommandInteger,
        trigger_events: bool,
    },
    /// Store `value` in `uber_identifier` and check if any events are triggered
    StoreFloat {
        uber_identifier: UberIdentifier,
        value: CommandFloat,
        trigger_events: bool,
    },
    /// Temporarily store `value` under `id`. The value should live at least until the next tick
    SetBoolean {
        id: usize,
        value: CommandBoolean,
    },
    /// Temporarily store `value` under `id`. The value should live at least until the next tick
    SetInteger {
        id: usize,
        value: CommandInteger,
    },
    /// Temporarily store `value` under `id`. The value should live at least until the next tick
    SetFloat {
        id: usize,
        value: CommandFloat,
    },
    /// Temporarily store `value` under `id`. The value should live at least until the next tick
    SetString {
        id: usize,
        value: CommandString,
    },
    /// Create a new box defined by (`x1`, `y1`) and (`x2`, `y2`)
    BoxTrigger {
        id: usize,
        x1: Box<CommandFloat>,
        y1: Box<CommandFloat>,
        x2: Box<CommandFloat>,
        y2: Box<CommandFloat>,
    },
    /// DESTROY box `id`
    BoxTriggerDestroy {
        id: usize,
    },
    /// Register `action` to trigger when Ori enters box `id`
    BoxTriggerEnterCallback {
        id: usize,
        action: usize,
    },
    /// Register `action` to trigger when Ori leaves box `id`
    BoxTriggerLeaveCallback {
        id: usize,
        action: usize,
    },
    /// Save the game
    Save {
        to_disk: bool,
    },
    /// Save the game at `position`
    SaveAt {
        to_disk: bool,
        x: CommandFloat,
        y: CommandFloat,
    },
    /// Warp the player to (`x`, `y`)
    Warp {
        x: CommandFloat,
        y: CommandFloat,
    },
    /// Equip `equipment` into `slot`
    Equip {
        slot: EquipSlot,
        equipment: Equipment,
    },
    /// Unequip `equipment` from any slot it may be equipped in
    Unequip {
        equipment: Equipment,
    },
    /// Act as though the client would have sent `client_event`
    TriggerClientEvent {
        client_event: ClientEvent,
    },
    /// Act as though the user would have pressed `bind`
    // TODO add possible binds to data crate and then simulate
    TriggerKeybind {
        bind: String,
    },
    /// Start syncing `uber_identifier` in co-op
    EnableServerSync {
        uber_identifier: UberIdentifier,
    },
    /// Stop syncing `uber_identifier` in co-op
    DisableServerSync {
        uber_identifier: UberIdentifier,
    },
    /// Create `icon` at (`x`, `y`) with the label set to `label`
    CreateSpoilerMapIcon {
        icon: MapIcon,
        x: CommandFloat,
        y: CommandFloat,
        label: CommandString,
    },
    /// Create a spirit well icon that you can warp to on the map at (`x`, `y`)
    CreateWarpIcon {
        id: usize,
        x: CommandFloat,
        y: CommandFloat,
    },
    /// Set the map label of an existing spirit well icon `id` to `label`
    SetWarpIconLabel {
        id: usize,
        label: CommandString,
    },
    /// DESTROY the spirit well icon `id`
    DestroyWarpIcon {
        id: usize,
    },
    // TODO would seem more efficient to set these at once to save uber_identifier lookups
    // (same for wheel stuff)
    /// Set the price of the shop item at `uber_identifier` to `price`
    SetShopItemPrice {
        uber_identifier: UberIdentifier,
        price: CommandInteger,
    },
    /// Set the display name of the shop item at `uber_identifier` to `name`
    SetShopItemName {
        uber_identifier: UberIdentifier,
        name: CommandString,
    },
    /// Set the description of the shop item at `uber_identifier` to `description`
    SetShopItemDescription {
        uber_identifier: UberIdentifier,
        description: CommandString,
    },
    /// Set the icon of the shop item at `uber_identifier` to `icon`
    SetShopItemIcon {
        uber_identifier: UberIdentifier,
        icon: Icon,
    },
    /// Set the shop item at `uber_identifier` to be `hidden`
    SetShopItemHidden {
        uber_identifier: UberIdentifier,
        hidden: CommandBoolean,
    },
    /// Set the shop item at `uber_identifier` to be `locked`
    SetShopItemLocked {
        uber_identifier: UberIdentifier,
        locked: CommandBoolean,
    },
    /// Set the display name of the wheel item in `wheel` at `position` to `name`
    SetWheelItemName {
        wheel: usize,
        position: WheelItemPosition,
        name: CommandString,
    },
    /// Set the description of the wheel item in `wheel` at `position` to `description`
    SetWheelItemDescription {
        wheel: usize,
        position: WheelItemPosition,
        description: CommandString,
    },
    /// Set the icon of the wheel item in `wheel` at `position` to `icon`
    SetWheelItemIcon {
        wheel: usize,
        position: WheelItemPosition,
        icon: Icon,
    },
    /// Set the rgba color of the wheel item in `wheel` at `position` to `red`, `green`, `blue`, `alpha`
    SetWheelItemColor {
        wheel: usize,
        position: WheelItemPosition,
        red: CommandInteger,
        green: CommandInteger,
        blue: CommandInteger,
        alpha: CommandInteger,
    },
    /// When pressing `bind` with the wheel item in `wheel` at `position` selected, lookup and perform `action`
    SetWheelItemAction {
        wheel: usize,
        position: WheelItemPosition,
        bind: WheelBind,
        action: usize,
    },
    /// Remove the wheel item in `wheel` at `position`
    DestroyWheelItem {
        wheel: usize,
        position: WheelItemPosition,
    },
    /// Switch the active wheel to `wheel`
    SwitchWheel {
        wheel: usize,
    },
    /// If a `wheel` is `pinned`, it should remain the active wheel after closing and reopening the randomizer wheel
    SetWheelPinned {
        wheel: usize,
        pinned: CommandBoolean,
    },
    /// Reset all wheel items to their default state
    ResetAllWheels {},
    /// Closes any open menu screen
    CloseMenu {},
    /// Closes the weapon wheel
    CloseWeaponWheel {},
    /// Write `message` into the client log
    DebugLog {
        message: CommandString,
    },
}

impl CommandVoid {
    pub fn find_message(&self) -> Option<&CommandString> {
        match self {
            CommandVoid::Multi { commands } => commands.iter().find_map(Self::find_message),
            CommandVoid::QueuedMessage { message, .. } => Some(message),
            _ => None,
        }
    }
}
