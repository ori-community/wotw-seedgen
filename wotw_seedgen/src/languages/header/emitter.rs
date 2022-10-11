use std::fmt::Display;

use rustc_hash::FxHashMap;

use crate::{Item, VItem, util::Icon, settings::Goal};

use super::{HeaderContent, V, VResolve, VString, HeaderCommand, Pickup, VPickup, GoalmodeHack};

/// Configurable details for how to treat an [`Item`] during seed generation
#[derive(Debug, Clone, Default)]
pub struct ItemDetails {
    /// The generic name of this item, which will be used for instance in shops or multiworld send messages
    pub name: Option<String>,
    /// The display of this item, which the player will see when collecting it
    pub display: Option<String>,
    /// The description of this item, which will be visible in shops
    pub description: Option<String>,
    /// The average price of this item when placing it in a shop
    pub price: Option<u32>,
    /// The icon to use when placing this item in a shop
    pub icon: Option<Icon>,
}

/// Compilation of all the manipulations a [`Header`](super::Header) wishes to do
#[derive(Debug, Clone, Default)]
pub struct HeaderBuild {
    /// Dependencies on other headers
    pub includes: Vec<String>,
    /// Incompabilities with other headers
    pub excludes: Vec<String>,
    /// Syntax to add into the seed
    pub seed_content: String,
    /// Flags to add to the seed
    pub flags: Vec<String>,
    /// Preplaced items
    pub preplacements: Vec<Pickup>,
    /// Changes to the item pool
    pub item_pool_changes: FxHashMap<Item, i32>,
    /// Item configurations. See [`ItemDetails`] for details
    pub item_details: FxHashMap<Item, ItemDetails>,
    /// Logical states to be set at the start of seed generation
    pub state_sets: Vec<String>,
    pub goals: Vec<Goal>,
}

pub(super) fn build(contents: Vec<HeaderContent>, parameters: &FxHashMap<String, String>) -> Result<HeaderBuild, String> {
    let mut header_build = HeaderBuild::default();

    // A `true` in the if stack represents "met, don't skip", `false` represents "unmet, skip"
    let mut if_stack = vec![];
    let mut lines = vec![];

    for content in contents {
        if if_stack.last().copied().unwrap_or(true) {
            match content {
                HeaderContent::OuterDocumentation(_) | HeaderContent::InnerDocumentation(_) |  HeaderContent::Annotation(_) => {},
                HeaderContent::Flags(mut flag_string) => header_build.flags.append(&mut flag_string),
                HeaderContent::Timer(timer) => lines.push(format!("timer: {}", timer.code())),
                HeaderContent::Command(command) => build_command(command, &mut header_build, &mut if_stack, parameters)?,
                HeaderContent::Pickup(pickup) => build_pickup(pickup, &mut lines, &mut header_build.preplacements, parameters)?,
            }
        } else if let HeaderContent::Command(command) = content {
            match command {
                HeaderCommand::If { .. } => if_stack.push(false),
                HeaderCommand::EndIf => build_endif(&mut if_stack)?,
                _ => { /* Continue skipping */ }
            }
        }
    }

    header_build.seed_content = lines.join("\n");

    Ok(header_build)
}

fn build_pickup(pickup: VPickup, lines: &mut Vec<String>, preplacements: &mut Vec<Pickup>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
    let pickup = pickup.resolve(parameters)?;
    lines.push(pickup.code().to_string());

    if !pickup.ignore {
        preplacements.push(pickup);
    }

    Ok(())
}

fn build_command(command: HeaderCommand, header_build: &mut HeaderBuild, if_stack: &mut Vec<bool>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
    match command {
        HeaderCommand::Include { name } => header_build.includes.push(name),
        HeaderCommand::Exclude { name } => header_build.excludes.push(name),
        HeaderCommand::Add { item, amount } => build_add(item, amount, &mut header_build.item_pool_changes, parameters)?,
        HeaderCommand::Remove { item, amount } => build_remove(item, amount, &mut header_build.item_pool_changes, parameters)?,
        HeaderCommand::Name { item, name } => build_name(item, name, &mut header_build.item_details, parameters)?,
        HeaderCommand::Display { item, name } => build_display(item, name, &mut header_build.item_details, parameters)?,
        HeaderCommand::Description { item, description } => build_description(item, description, &mut header_build.item_details, parameters)?,
        HeaderCommand::Price { item, price } => build_price(item, price, &mut header_build.item_details, parameters)?,
        HeaderCommand::Icon { item, icon } => build_icon(item, icon, &mut header_build.item_details, parameters)?,
        HeaderCommand::Parameter { .. } => { /* Skip, parameters have been processed earlier */ },
        HeaderCommand::Set { state } => header_build.state_sets.push(state),
        HeaderCommand::If { parameter, value } => build_if(&parameter, &value, if_stack, parameters)?,
        HeaderCommand::EndIf => build_endif(if_stack)?,
        HeaderCommand::GoalmodeHack(goalmode) => build_goalmode(goalmode, &mut header_build.goals, parameters)?,
    }

    Ok(())
}

fn build_if(parameter: &str, value: &str, if_stack: &mut Vec<bool>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
    let met = parameters.get(parameter).ok_or_else(|| format!("Unknown parameter {parameter} in if"))? == value;
    if_stack.push(met);
    Ok(())
}
fn build_endif(if_stack: &mut Vec<bool>) -> Result<(), String> {
    if_stack.pop().ok_or_else(|| "Unexpected !!endif without an open !!if block".to_string()).map(|_| ())
}

fn build_add(item: VItem, amount: V<i32>, item_pool_changes: &mut FxHashMap<Item, i32>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
    let amount = amount.resolve(parameters)?;
    change_item_pool(item, amount, item_pool_changes, parameters)
}
fn build_remove(item: VItem, amount: V<i32>, item_pool_changes: &mut FxHashMap<Item, i32>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
    let amount = -amount.resolve(parameters)?;
    change_item_pool(item, amount, item_pool_changes, parameters)
}

fn change_item_pool(item: VItem, amount: i32, item_pool_changes: &mut FxHashMap<Item, i32>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
    let item = item.resolve(parameters)?;
    item_pool_changes.entry(item)
        .and_modify(|prior| *prior += amount)
        .or_insert(amount);

    Ok(())
}

macro_rules! __vdetails {
    ($fn_ident:ident $field_ident:ident $field_name:literal $ty:ty) => {
        fn $fn_ident(item: VItem, $field_ident: $ty, item_details: &mut FxHashMap<Item, ItemDetails>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
            let $field_ident = $field_ident.resolve(parameters)?;
            let item = item.resolve(parameters)?;
            let detail = &mut item_details.entry(item).or_default().$field_ident;
            change_item_details($field_ident, detail, $field_name)
        }
    };
}
macro_rules! details {
    ($fn_ident:ident $field_ident:ident $field_name:literal V<$ty:ty>) => {
        __vdetails!($fn_ident $field_ident $field_name V<$ty>);
    };
    ($fn_ident:ident $field_ident:ident $field_name:literal VString) => {
        __vdetails!($fn_ident $field_ident $field_name VString);
    };
    ($fn_ident:ident $field_ident:ident $field_name:literal $ty:ty) => {
        fn $fn_ident(item: VItem, $field_ident: $ty, item_details: &mut FxHashMap<Item, ItemDetails>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
            let item = item.resolve(parameters)?;
            let detail = &mut item_details.entry(item).or_default().$field_ident;
            change_item_details($field_ident, detail, $field_name)
        }
    };
}
details!(build_name name "name" VString);
details!(build_display display "display" VString);
details!(build_description description "description" VString);
details!(build_price price "price" V<u32>);
details!(build_icon icon "icon" Icon);

fn change_item_details<T: Display>(value: T, detail: &mut Option<T>, field_name: impl Display) -> Result<(), String> {
    if let Some(prior) = detail {
        Err(format!("Tried to assign {field_name} {value}, but already assigned {prior} earlier"))
    } else {
        *detail = Some(value);
        Ok(())
    }
}

fn build_goalmode(goalmode: GoalmodeHack, goals: &mut Vec<Goal>, parameters: &FxHashMap<String, String>) -> Result<(), String> {
    let goal = match goalmode {
        GoalmodeHack::Trees => Goal::Trees,
        GoalmodeHack::Wisps => Goal::Wisps,
        GoalmodeHack::Quests => Goal::Quests,
        GoalmodeHack::Relics { chance, amount } => {
            let chance = chance.resolve(parameters)?.min(1.0);
            let amount = amount.resolve(parameters)?;

            if amount == 0 {
                Goal::RelicChance(chance)
            } else {
                Goal::Relics(amount)
            }
        },
    };

    goals.push(goal);

    Ok(())
}
