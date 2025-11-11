use serde::{Deserialize, Serialize};
use wotw_seedgen_data::{
    Alignment, CoordinateSystem, EquipSlot, Equipment, HorizontalAnchor, Icon, MapIcon,
    UberIdentifier, VerticalAnchor, WheelBind, WheelItemPosition,
};
use wotw_seedgen_seed_language::output::{
    ArithmeticOperator, ClientEvent, Comparator, EqualityComparator, LogicOperator,
};

/// Contains the compiled seedgen output that makes up the seed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assembly {
    /// Events from generation and snippets
    pub events: Vec<Event>,
    /// [`Command`]s that may be referenced from elsewhere by index
    ///
    /// Each index may store multiple [`Command`]s to execute
    pub command_lookup: Vec<Vec<Command>>,
}

// TODO maybe events should have a Vec<Command> instead of usize?
/// The main event (:badumtsss:)
///
/// The Trigger defines when to execute the command at the index
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event(pub Trigger, pub usize);

/// Trigger for an [`Event`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigger {
    /// Specific client events
    ClientEvent(ClientEvent),
    /// Trigger on every change to an UberIdentifier
    Binding(UberIdentifier),
    /// Index into `command_lookup`
    ///
    /// After executing the command, Boolean Memory 0 determines whether the condition is met.
    /// The last result of executing the command should be saved, with an initial value of `false`.
    /// The trigger should only fire if the last result was `false` and the current result is `true`
    Condition(usize),
}

// TODO breakpoint/debug logging toggle command?

/// A Command, which may be used to affect the world, player or client state
///
/// Mirrors https://github.com/ori-community/wotw-rando-client/blob/v5/projects/Randomizer/seed/instructions/save_at.h
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Execute the commands at `index` in command_lookup
    Execute(/*index*/ usize),
    /// Execute the commands at `index` in command_lookup if Boolean Memory 0 is `true`
    ExecuteIf(/*index*/ usize),
    /// Set Boolean Memory 0 to `value`
    SetBoolean(/*value*/ bool),
    /// Set Integer Memory 0 to `value`
    SetInteger(/*value*/ i32),
    /// Set Float Memory 0 to `value`
    SetFloat(/*value*/ f32),
    /// Set String Memory 0 to `value`
    SetString(/*value*/ String),
    /// Copy address `from` into address `to` in Boolean Memory
    CopyBoolean(/*from*/ usize, /*to*/ usize),
    /// Copy address `from` into address `to` in Integer Memory
    CopyInteger(/*from*/ usize, /*to*/ usize),
    /// Copy address `from` into address `to` in Float Memory
    CopyFloat(/*from*/ usize, /*to*/ usize),
    /// Copy address `from` into address `to` in String Memory
    CopyString(/*from*/ usize, /*to*/ usize),
    /// Copy the value of `uber_identifier` into Boolean Memory 0
    FetchBoolean(/*uber_identifier*/ UberIdentifier),
    /// Copy the value of `uber_identifier` into Integer Memory 0
    FetchInteger(/*uber_identifier*/ UberIdentifier),
    /// Copy the value of `uber_identifier` into Float Memory 0
    FetchFloat(/*uber_identifier*/ UberIdentifier),
    /// Copy the value of Boolean Memory 0 into `uber_identifier`
    StoreBoolean(
        /*uber_identifier*/ UberIdentifier,
        /*trigger_events*/ bool,
    ),
    /// Copy the value of Integer Memory 0 into `uber_identifier`
    StoreInteger(
        /*uber_identifier*/ UberIdentifier,
        /*trigger_events*/ bool,
    ),
    /// Copy the value of Float Memory 0 into `uber_identifier`
    StoreFloat(
        /*uber_identifier*/ UberIdentifier,
        /*trigger_events*/ bool,
    ),
    /// Perform `operator` on Boolean Memory 0 and Boolean Memory 1 and store the result in Boolean Memory 0
    CompareBoolean(/*operator*/ EqualityComparator),
    /// Perform `operator` on Integer Memory 0 and Integer Memory 1 and store the result in Integer Memory 0
    CompareInteger(/*operator*/ Comparator),
    /// Perform `operator` on Float Memory 0 and Float Memory 1 and store the result in Float Memory 0
    CompareFloat(/*operator*/ Comparator),
    /// Perform `operator` on String Memory 0 and String Memory 1 and store the result in String Memory 0
    CompareString(/*operator*/ EqualityComparator),
    /// Perform `operator` on Boolean Memory 0 and Boolean Memory 1 and store the result in Boolean Memory 0
    LogicOperation(/*operator*/ LogicOperator),
    /// Perform `operator` on Integer Memory 0 and Integer Memory 1 and store the result in Integer Memory 0
    ArithmeticInteger(/*operator*/ ArithmeticOperator),
    /// Perform `operator` on Float Memory 0 and Float Memory 1 and store the result in Float Memory 0
    ArithmeticFloat(/*operator*/ ArithmeticOperator),
    /// Round Float Memory 0
    Round,
    /// Concatenate String Memory 0 and String Memory 1 and store the result in String Memory 0
    Concatenate,
    /// Convert Float Memory 0 to an integer and store it in Integer Memory 0
    FloatToInteger,
    /// Convert Integer Memory 0 to a float and store it in Float Memory 0
    IntegerToFloat,
    /// Convert Boolean Memory 0 to a string and store it in String Memory 0
    BooleanToString,
    /// Convert Integer Memory 0 to a string and store it in String Memory 0
    IntegerToString,
    /// Convert Float Memory 0 to a string and store it in String Memory 0
    FloatToString,
    /// Until the next reload, on every tick where `toggle` is `true` increment `timer` by the delta time in seconds
    DefineTimer(
        /*toggle*/ UberIdentifier,
        /*timer*/ UberIdentifier,
    ),
    /// Check if Ori is in the hitbox defined by (Float Memory 0, Float Memory 1) and (Float Memory 2, Float Memory 3) and store the result in Boolean Memory 0
    IsInHitbox,
    /// Store the name of world number `index` in String Memory 0
    WorldName(/*index*/ usize),
    // TODO control whether messages play sound
    /// Create a queued message with String Memory 0 as content and Float Memory 0 as timeout
    QueuedMessage(/*index*/ Option<usize>, /*priority*/ bool),
    /// Create a free message with `id`
    FreeMessage(/*id*/ usize),
    /// DESTROY, OBLITERATE and ANNIHILATE message `id`
    MessageDestroy(/*id*/ usize),
    /// Update the content of message `id` with String Memory 0
    MessageText(/*id*/ usize),
    /// Update the timeout of message `id` with Float Memory 0
    MessageTimeout(/*id*/ usize),
    /// Update whether the background of message `id` is enabled based on Boolean Memory 0
    MessageBackground(/*id*/ usize),
    /// If queued message `id` get shown, execute `command`
    QueuedMessageShownCallback(/*id*/ usize, /*command*/ usize),
    /// If queued message `id` get hidden, execute `command`
    QueuedMessageHiddenCallback(/*id*/ usize, /*command*/ usize),
    /// Show free message `id` and play a sound if Boolean Memory 0 is `true`
    FreeMessageShow(/*id*/ usize),
    /// Hide free message `id`
    FreeMessageHide(/*id*/ usize),
    /// Set the position of free message `id` to (Float Memory 0, Float Memory 1)
    FreeMessagePosition(/*id*/ usize),
    /// Set the `alignment` of free message `id`
    FreeMessageAlignment(/*id*/ usize, /*alignment*/ Alignment),
    /// Set the `horizontal_anchor` of free message `id`
    FreeMessageHorizontalAnchor(
        /*id*/ usize,
        /*horizontal_anchor*/ HorizontalAnchor,
    ),
    /// Set the `vertical_anchor` of free message `id`
    FreeMessageVerticalAnchor(/*id*/ usize, /*vertical_anchor*/ VerticalAnchor),
    /// Sets the box width of free message `id` to Float Memory 0
    FreeMessageBoxWidth(/*id*/ usize),
    /// Sets the coordinate system of free message `id` to `coordinate_system`
    FreeMessageCoordinateSystem(
        /*id*/ usize,
        /*coordinate_system*/ CoordinateSystem,
    ),
    /// Sets the map message content to String Memory 0
    SetMapMessage,
    // TODO missing SetSideMapMessage
    /// Save the game. Only save to disk if Boolean Memory 0 is `true`.
    Save,
    /// Save the game, but with the position set to (Float Memory 0, Float Memory 1). Only save to disk if Boolean Memory 0 is `true`.
    SaveAt,
    // TODO preload area command
    /// Warp the player to (Float Memory 0, Float Memory 1)
    Warp,
    /// Equip `equipment` into `slot`
    Equip(/*slot*/ EquipSlot, /*equipment*/ Equipment),
    /// Unequip `equipment` from any slot it may be equipped in
    Unequip(/*equipment*/ Equipment),
    /// Act as though the user would have pressed `bind`
    TriggerKeybind(/*bind*/ String),
    /// Start syncing `uber_identifier` in co-op
    EnableServerSync(/*uber_identifier*/ UberIdentifier),
    /// Stop syncing `uber_identifier` in co-op
    DisableServerSync(/*uber_identifier*/ UberIdentifier),
    /// Set the map icon associated with the `location` identifier from loc_data to `icon` and the label to String Memory 0
    SetSpoilerMapIcon(/*location*/ String, /*icon*/ MapIcon),
    /// Create a spirit well map icon with `id` that you can warp to at (Float Memory 0, Float Memory 1)
    CreateWarpIcon(/*id*/ usize),
    /// If `id` refers to an existing spirit well icon, set its label to String Memory 0
    SetWarpIconLabel(/*id*/ usize),
    /// If `id` refers to an existing spirit well icon, DESTROY, OBLITERATE and ANNIHILATE it
    DestroyWarpIcon(/*id*/ usize),
    // TODO could instead do a SelectShopItem command and then omit the uber_identifier from all the other commands and similarly for other commands, might reduce seed size?
    /// Set the price of the shop item at `uber_identifier` to Integer Memory 0
    SetShopItemPrice(/*uber_identifier*/ UberIdentifier),
    /// Set the display name of the shop item at `uber_identifier` to String Memory 0
    SetShopItemName(/*uber_identifier*/ UberIdentifier),
    /// Set the description of the shop item at `uber_identifier` to String Memory 0
    SetShopItemDescription(/*uber_identifier*/ UberIdentifier),
    /// Set the icon of the shop item at `uber_identifier` to `icon`
    SetShopItemIcon(/*uber_identifier*/ UberIdentifier, Icon),
    /// Set whether the shop item at `uber_identifier` is hidden based on Boolean Memory 0
    SetShopItemHidden(/*uber_identifier*/ UberIdentifier),
    /// Set whether the shop item at `uber_identifier` is locked based on Boolean Memory 0
    SetShopItemLocked(/*uber_identifier*/ UberIdentifier),
    /// Set the display name of the wheel item in `wheel` at `position` to String Memory 0
    SetWheelItemName(/*wheel*/ usize, /*position*/ WheelItemPosition),
    /// Set the description of the wheel item in `wheel` at `position` to String Memory 0
    SetWheelItemDescription(/*wheel*/ usize, /*position*/ WheelItemPosition),
    /// Set the icon of the wheel item in `wheel` at `position` to `icon`
    SetWheelItemIcon(
        /*wheel*/ usize,
        /*position*/ WheelItemPosition,
        /*icon*/ Icon,
    ),
    /// Set the rgba color of the wheel item in `wheel` at `position` to Integer Memory 0, Integer Memory 1, Integer Memory 2, Integer Memory 3
    SetWheelItemColor(/*wheel*/ usize, /*position*/ WheelItemPosition),
    /// When pressing `bind` with the wheel item in `wheel` at `position` selected, execute `command`
    SetWheelItemCommand(
        /*wheel*/ usize,
        /*position*/ WheelItemPosition,
        /*bind*/ WheelBind,
        /*command*/ usize,
    ),
    /// If something exists in `wheel` at `position`, DESTROY, OBLITERATE and ANNIHILATE it
    DestroyWheelItem(/*wheel*/ usize, /*position*/ WheelItemPosition),
    /// Switch the active wheel to `wheel`
    SwitchWheel(/*wheel*/ usize),
    /// Sets whether `wheel` is pinned based on Boolean Memory 0
    SetWheelPinned(/*wheel*/ usize),
    /// Reset all wheel items to their default state
    ResetAllWheels,
    /// Write String Memory 0 into the client log
    DebugLog,
    // TODO missing SetDebuggerTrace
    // TODO missing SetTextWithId (?)
}
