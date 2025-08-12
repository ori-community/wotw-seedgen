// TODO this module name is confusing

use super::{Compile, ExportedValue, SnippetCompiler};
use crate::{
    ast::{self, CommandArg, UberStateType},
    output::{CommandVoid, Event, ItemMetadataEntry, Literal, StringOrPlaceholder, Trigger},
    types::InferType,
};
use ast::ClientEvent;
use ordered_float::OrderedFloat;
use rand::Rng;
use std::{iter, mem, ops::Range};
use wotw_seedgen_assets::UberStateAlias;
use wotw_seedgen_data::{Position, UberIdentifier, Zone};
use wotw_seedgen_parse::{Error, Identifier, Result, Span, SpanEnd, SpanStart};

fn consume_command_arg<T>(arg: CommandArg<T>, compiler: &mut SnippetCompiler) -> Option<T> {
    compiler
        .consume_result(arg.result)
        .and_then(|(_, arg)| compiler.consume_result(arg.result))
}

impl<'source> Compile<'source> for ast::Command<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Command::Include(_, command) => {
                command.compile(compiler);
            }
            ast::Command::BundleIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::BuiltinIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Event(_, command) => {
                command.compile(compiler);
            }
            ast::Command::OnEvent(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Export(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Spawn(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Tags(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Config(_, command) => {
                command.compile(compiler);
            }
            ast::Command::State(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Timer(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Let(_, command) => {
                command.compile(compiler);
            }
            ast::Command::If(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Repeat(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Add(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Remove(_, command) => {
                command.compile(compiler);
            }
            ast::Command::ItemData(_, command) => {
                command.compile(compiler);
            }
            ast::Command::ItemDataName(_, command) => {
                command.compile(compiler);
            }
            ast::Command::ItemDataPrice(_, command) => {
                command.compile(compiler);
            }
            ast::Command::ItemDataDescription(_, command) => {
                command.compile(compiler);
            }
            ast::Command::ItemDataIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RemoveLocation(_, command) => {
                command.compile(compiler);
            }
            ast::Command::SetLogicState(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Preplace(_, command) => {
                command.compile(compiler);
            }
            ast::Command::ZoneOf(_, command) => {
                command.compile(compiler);
            }
            ast::Command::ItemOn(_, command) => {
                command.compile(compiler);
            }
            ast::Command::CountInZone(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RandomInteger(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RandomFloat(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RandomPool(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RandomFromPool(_, command) => {
                command.compile(compiler);
            }
        }
    }
}
impl<'source> Compile<'source> for ast::IncludeArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let Some(snippet_exported_values) = compiler.global.exported_values.get(self.path.data)
        else {
            compiler
                .errors
                .push(Error::custom("unknown snippet".to_string(), self.path.span));
            return;
        };

        if let Some((_, imports)) = self.imports.data {
            for import in imports {
                let Some(value) = snippet_exported_values.get(import.data.0) else {
                    compiler.errors.push(
                        Error::custom("identifier not found in snippet".to_string(), import.span)
                            .with_help(format!(
                                "if it exists in {}, you have to export it there: !export({})",
                                self.path.data, import.data
                            )),
                    );
                    continue;
                };

                match value {
                    ExportedValue::Function(index) => {
                        compiler
                            .preprocessed
                            .functions
                            .insert(import.data.0.to_string());
                        compiler
                            .function_indices
                            .insert(import.data.0.to_string(), *index);
                        // TODO is this still used?
                        compiler
                            .function_imports
                            .insert(import.data.0.to_string(), self.path.data.to_string());
                    }
                    ExportedValue::Literal(literal) => {
                        compiler.variables.insert(import.data, literal.clone());
                    }
                }
            }
        }
    }
}
impl<'source> Compile<'source> for ast::BundleIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let Some(path) = consume_command_arg(self.path, compiler) else {
            return;
        };

        let content = compiler
            .global
            .snippet_access
            .read_file(path.data.as_ref())
            .map_err(|err| Error::custom(err, path.span()));
        if let Some(data) = compiler.consume_result(content) {
            compiler
                .global
                .output
                .icons
                .push((path.data.to_string(), data));
            compiler.variables.insert(
                self.identifier.data,
                Literal::CustomIcon(path.data.to_string()),
            );
        }
    }
}
impl<'source> Compile<'source> for ast::BuiltinIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let Some(path) = consume_command_arg(self.path, compiler) else {
            return;
        };

        compiler.variables.insert(
            self.identifier.data,
            Literal::IconAsset(path.data.to_string()),
        );
    }
}
impl<'source> Compile<'source> for ast::EventArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let index = compiler.function_indices[self.0.data.0];
        compiler
            .global
            .events
            .entry(compiler.identifier.clone())
            .or_default()
            .insert(self.0.data.0.to_string(), index);
    }
}
impl<'source> Compile<'source> for ast::OnEventArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let identifier = consume_command_arg(self.identifier, compiler);
        let action = consume_command_arg(self.action, compiler);

        if !compiler
            .preprocessed
            .includes
            .iter()
            .any(|include| include.data == self.snippet_name.data)
        {
            compiler.errors.push(
                Error::custom("unknown snippet".to_string(), self.snippet_name.span)
                    .with_help(format!("try !include(\"{}\")", self.snippet_name.data)),
            );
            return;
        }

        let (Some(identifier), Some(action)) = (identifier, action) else {
            return;
        };

        let event = compiler
            .global
            .events
            .get(self.snippet_name.data)
            .and_then(|events| events.get(identifier.data.0))
            .copied();
        if event.is_none() {
            compiler.errors.push(Error::custom(
                "Could not find event in snippet".to_string(),
                identifier.span,
            ));
        }

        let span = action.span();
        let action = action
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, span));

        if let (Some(event), Some(action)) = (event, action) {
            if let CommandVoid::Multi { commands } =
                &mut compiler.global.output.command_lookup[event]
            {
                match action {
                    CommandVoid::Multi { commands: extend } => commands.extend(extend),
                    single => commands.push(single),
                }
            }
        }
    }
}
impl<'source> Compile<'source> for ast::ExportArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let identifier = self.0.data;

        let variable = compiler.variables.get(&self.0.data);
        let function = compiler.function_indices.get(self.0.data.0);

        let value = match (variable, function) {
            (None, Some(index)) => ExportedValue::Function(*index),
            (Some(var), None) => ExportedValue::Literal(var.clone()),
            (Some(_), Some(_)) => {
                compiler.errors.push(Error::custom(
                    "Could refer to either a function or a variable in the current scope. Consider renaming one of them to resolve the ambiguity".to_string(),
                    self.0.span,
                ));
                return;
            }
            (None, None) => {
                compiler.errors.push(Error::custom(
                    "Could not find function or variable in the current scope".to_string(),
                    self.0.span,
                ));
                return;
            }
        };

        compiler
            .global
            .exported_values
            .entry(compiler.identifier.clone())
            .or_default()
            .insert(identifier.0.to_string(), value);
    }
}
impl<'source> Compile<'source> for ast::SpawnArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if compiler.global.output.spawn.is_some() {
            compiler.errors.push(Error::custom(
                "Multiple spawn commands".to_string(),
                self.span(),
            ));
        }

        let x = self.x.evaluate(compiler);
        let y = consume_command_arg(self.y, compiler).and_then(|y| y.evaluate(compiler));

        let (Some(x), Some(y)) = (x, y) else { return };

        compiler.global.output.spawn = Some(Position { x, y });
    }
}
impl<'source> Compile<'source> for ast::TagsArg<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(tag) = self.0.evaluate(compiler) {
            compiler.global.output.tags.push(tag);
        }
    }
}
impl<'source> Compile<'source> for ast::ConfigArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        consume_command_arg(self.description, compiler);
        let ty = consume_command_arg(self.ty, compiler);
        let default = consume_command_arg(self.default, compiler);

        let (Some(ty), Some(default)) = (ty, default) else {
            return;
        };

        if default.data.infer_type(compiler) != Some(ty.data.into()) {
            compiler
                .errors
                .push(Error::custom(format!("expected {}", ty.data), default.span));
        }

        let config = compiler
            .global
            .config
            .get(&compiler.identifier)
            .and_then(|config| config.get(self.identifier.data.0));
        let value = match config {
            None => default.data.compile(compiler),
            Some(value) => {
                let parsed = match ty.data {
                    ast::ConfigType::Boolean => value.parse().ok().map(Literal::Boolean),
                    ast::ConfigType::Integer => value.parse().ok().map(Literal::Integer),
                    ast::ConfigType::Float => value.parse().ok().map(Literal::Float),
                };
                if parsed.is_none() {
                    compiler.errors.push(Error::custom(
                    format!("failed to parse provided configuration value \"{}\" as a {}, which is the required type for this configuration parameter", value, ty.data),
                    ty.span,
                ));
                }
                parsed
            }
        };
        if let Some(value) = value {
            compiler.variables.insert(self.identifier.data, value);
        }
    }
}
impl<'source> Compile<'source> for ast::StateArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.span();

        let Some(ty) = consume_command_arg(self.ty, compiler) else {
            return;
        };

        let uber_identifier = match ty.data {
            UberStateType::Boolean => boolean_uber_state(compiler, span),
            UberStateType::Integer => integer_uber_state(compiler, span),
            UberStateType::Float => float_uber_state(compiler, span),
        };

        if let Some(uber_identifier) = compiler.consume_result(uber_identifier) {
            compiler.variables.insert(
                self.identifier.data,
                Literal::UberIdentifier(UberStateAlias {
                    uber_identifier,
                    value: None,
                }),
            );
        }
    }
}
impl<'source> Compile<'source> for ast::TimerArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let toggle = boolean_uber_state(compiler, self.toggle_identifier.span);
        let toggle = compiler.consume_result(toggle);

        let Some(timer_identifier) = consume_command_arg(self.timer_identifier, compiler) else {
            return;
        };
        let timer = float_uber_state(compiler, timer_identifier.span);
        let timer = compiler.consume_result(timer);

        if let (Some(toggle), Some(timer)) = (toggle, timer) {
            compiler.global.output.events.push(Event {
                trigger: Trigger::ClientEvent(ClientEvent::Reload),
                command: CommandVoid::DefineTimer { toggle, timer },
            });
            compiler.variables.insert(
                self.toggle_identifier.data,
                Literal::UberIdentifier(UberStateAlias {
                    uber_identifier: toggle,
                    value: None,
                }),
            );
            compiler.variables.insert(
                timer_identifier.data,
                Literal::UberIdentifier(UberStateAlias {
                    uber_identifier: timer,
                    value: None,
                }),
            );
        }
    }
}
// TODO make internal states this order?
fn boolean_uber_state<S: Span>(compiler: &mut SnippetCompiler, span: S) -> Result<UberIdentifier> {
    check_limit(&mut compiler.global.boolean_state_id, 100, 50, span)
}
fn integer_uber_state<S: Span>(compiler: &mut SnippetCompiler, span: S) -> Result<UberIdentifier> {
    check_limit(&mut compiler.global.integer_state_id, 0, 100, span)
}
fn float_uber_state<S: Span>(compiler: &mut SnippetCompiler, span: S) -> Result<UberIdentifier> {
    check_limit(&mut compiler.global.float_state_id, 150, 25, span)
}
fn check_limit<S: Span>(
    id: &mut usize,
    offset: usize,
    available: usize,
    span: S,
) -> Result<UberIdentifier> {
    if *id - offset < available {
        let uber_identifier = UberIdentifier {
            group: 9,
            member: *id as i32,
        };
        *id += 1;
        Ok(uber_identifier)
    } else {
        Err(Error::custom(format!("Only {available} UberStates of this type are available (What on earth are you doing?)"), span.span()))
    }
}
impl<'source> Compile<'source> for ast::LetArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(value) =
            consume_command_arg(self.value, compiler).and_then(|value| value.evaluate(compiler))
        {
            compiler.variables.insert(self.identifier.data, value);
        }
    }
}
impl<'source> Compile<'source> for ast::CommandIf<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(true) = self.condition.evaluate(compiler) {
            self.contents.compile(compiler);
        }
    }
}
impl<'source> Compile<'source> for ast::CommandRepeat<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.amount.span();

        if let Some(repetitions) = self.amount.evaluate::<i32>(compiler) {
            if repetitions < 0 {
                compiler.errors.push(Error::custom(
                    format!("!repeat only allows for positive numbers, but this evaluated to {repetitions}"),
                    span,
                ));
                return;
            }

            for contents in iter::repeat(self.contents.content).take(repetitions as usize) {
                // short circuit on errors to avoid adding the same errors repeatedly
                match contents.compile(compiler) {
                    None => break,
                    Some(nested) => {
                        if nested.contains(&None) {
                            break;
                        }
                    }
                }
            }
        }
    }
}
impl<'source> Compile<'source> for ast::AddArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        compile_item_pool_change::<1>(self.0, compiler)
    }
}
impl<'source> Compile<'source> for ast::RemoveArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        compile_item_pool_change::<-1>(self.0, compiler)
    }
}
fn compile_item_pool_change<'source, const FACTOR: i32>(
    args: ast::ChangeItemPoolArgs<'source>,
    compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
) {
    let span = args.item.span();
    let item = args
        .item
        .compile(compiler)
        .and_then(|command| command.expect_void(compiler, span));

    let amount = consume_command_arg(args.amount, compiler)
        .and_then(|amount| amount.evaluate::<i32>(compiler));

    if let (Some(item), Some(amount)) = (item, amount) {
        *compiler
            .global
            .output
            .item_pool_changes
            .entry(item)
            .or_default() += amount * FACTOR;
    }
}
// TODO the practice of writing out the full item everytime seems a little dated now...
// maybe there could be a better system here that allows you to reference existing items easily, but still reference them by their full form e.g. to rename default pool items
impl<'source> Compile<'source> for ast::ItemDataArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));
        let name =
            consume_command_arg(self.name, compiler).and_then(|name| name.evaluate(compiler));
        let price = consume_command_arg(self.price, compiler)
            .and_then(|price| price.compile_into(compiler));
        let description = consume_command_arg(self.description, compiler)
            .and_then(|description| description.compile_into(compiler));
        let icon =
            consume_command_arg(self.icon, compiler).and_then(|icon| icon.compile_into(compiler));
        let map_icon = consume_command_arg(self.map_icon, compiler)
            .and_then(|map_icon| map_icon.compile_into(compiler));

        if let Some(item) = item {
            if compiler
                .global
                .output
                .item_metadata
                .0
                .insert(
                    item,
                    ItemMetadataEntry {
                        name,
                        price,
                        description,
                        icon,
                        map_icon,
                    },
                )
                .is_some()
            {
                compiler.errors.push(Error::custom(
                    "Already defined data for this item".to_string(),
                    span,
                ));
            }
        }
    }
}
impl<'source> Compile<'source> for ast::ItemDataNameArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));
        let name =
            consume_command_arg(self.name, compiler).and_then(|name| name.evaluate(compiler));
        if let (Some(item), Some(name)) = (item, name) {
            insert_item_data(compiler, item, span, name, "name", |data| &mut data.name);
        }
    }
}
impl<'source> Compile<'source> for ast::ItemDataPriceArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));
        let price = consume_command_arg(self.price, compiler)
            .and_then(|price| price.compile_into(compiler));

        if let (Some(item), Some(price)) = (item, price) {
            insert_item_data(compiler, item, span, price, "price", |data| &mut data.price);
        }
    }
}
impl<'source> Compile<'source> for ast::ItemDataDescriptionArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));
        let description = consume_command_arg(self.description, compiler)
            .and_then(|description| description.compile_into(compiler));

        if let (Some(item), Some(description)) = (item, description) {
            insert_item_data(compiler, item, span, description, "description", |data| {
                &mut data.description
            });
        }
    }
}
impl<'source> Compile<'source> for ast::ItemDataIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));
        let icon =
            consume_command_arg(self.icon, compiler).and_then(|icon| icon.compile_into(compiler));

        if let (Some(item), Some(icon)) = (item, icon) {
            insert_item_data(compiler, item, span, icon, "icon", |data| &mut data.icon);
        }
    }
}
fn insert_item_data<T, F: FnOnce(&mut ItemMetadataEntry) -> &mut Option<T>>(
    compiler: &mut SnippetCompiler,
    item: CommandVoid,
    span: Range<usize>,
    value: T,
    field: &str,
    f: F,
) {
    if mem::replace(
        f(compiler
            .global
            .output
            .item_metadata
            .0
            .entry(item)
            .or_default()),
        Some(value),
    )
    .is_some()
    {
        compiler.errors.push(Error::custom(
            format!("Already defined {field} for this item"),
            span,
        ))
    }
}
impl<'source> Compile<'source> for ast::RemoveLocationArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(condition) = self.condition.compile_into(compiler) {
            compiler.global.output.removed_locations.insert(condition);
        }
    }
}
impl<'source> Compile<'source> for ast::SetLogicStateArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        compiler
            .global
            .output
            .logical_state_sets
            .insert(self.0.data.to_string());
    }
}
impl<'source> Compile<'source> for ast::PreplaceArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, span));
        let zone =
            consume_command_arg(self.zone, compiler).and_then(|zone| zone.evaluate(compiler));

        if let (Some(item), Some(zone)) = (item, zone) {
            compiler.global.output.preplacements.push((item, zone));
        }
    }
}
impl<'source> Compile<'source> for ast::ZoneOfArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let Some(item) = consume_command_arg(self.item, compiler) else {
            return;
        };

        let span = item.span();
        let item = item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, span));
        if let Some(item) = item {
            compiler.variables.insert(
                self.identifier.data,
                Literal::String(StringOrPlaceholder::ZoneOfPlaceholder(Box::new(item))),
            );
        }
    }
}
impl<'source> Compile<'source> for ast::ItemOnArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(trigger) = consume_command_arg(self.trigger, compiler)
            .and_then(|trigger| trigger.compile(compiler))
        {
            compiler.variables.insert(
                self.identifier.data,
                Literal::String(StringOrPlaceholder::ItemOnPlaceholder(Box::new(trigger))),
            );
        }
    }
}
impl<'source> Compile<'source> for ast::CountInZoneArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let zone_bindings = self
            .zone_bindings
            .compile(compiler)
            .into_iter()
            .flatten()
            .flatten()
            .flatten();

        let Some(items) = consume_command_arg(self.items, compiler) else {
            return;
        };

        let items = compiler
            .consume_result(items.content)
            .into_iter()
            .flatten()
            .filter_map(|action| {
                let span = action.span();
                action
                    .compile(compiler)
                    .and_then(|command| command.expect_void(compiler, span))
            })
            .collect::<Vec<_>>();

        for (identifier, zone) in zone_bindings {
            compiler.variables.insert(
                identifier,
                Literal::String(StringOrPlaceholder::CountInZonePlaceholder(
                    items.clone(),
                    zone,
                )),
            );
        }
    }
}
impl<'source> Compile<'source> for ast::CountInZoneBinding<'source> {
    type Output = Option<(Identifier<'source>, Zone)>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        consume_command_arg(self.zone, compiler)
            .and_then(|zone| zone.evaluate(compiler))
            .map(|zone| (self.identifier.data, zone))
    }
}
impl<'source> Compile<'source> for ast::RandomIntegerArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let min = consume_command_arg(self.0.min, compiler).and_then(|min| min.evaluate(compiler));
        let max = consume_command_arg(self.0.max, compiler).and_then(|max| max.evaluate(compiler));
        if let (Some(min), Some(max)) = (min, max) {
            let value = compiler.rng.gen_range(min..=max);
            compiler
                .variables
                .insert(self.0.identifier.data, Literal::Integer(value));
        }
    }
}
impl<'source> Compile<'source> for ast::RandomFloatArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let min = consume_command_arg(self.0.min, compiler)
            .and_then(|min| min.evaluate::<OrderedFloat<f32>>(compiler));
        let max = consume_command_arg(self.0.max, compiler)
            .and_then(|max| max.evaluate::<OrderedFloat<f32>>(compiler));
        if let (Some(min), Some(max)) = (min, max) {
            let value: f32 = compiler.rng.gen_range(min.into()..=max.into());
            compiler
                .variables
                .insert(self.0.identifier.data, Literal::Float(value.into()));
        }
    }
}
impl<'source> Compile<'source> for ast::RandomPoolArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        consume_command_arg(self.ty, compiler);
        let Some(values) = consume_command_arg(self.values, compiler) else {
            return;
        };

        let mut options_iter = compiler
            .consume_result(values.content)
            .into_iter()
            .flatten()
            .map(|expression| expression.evaluate(compiler));

        // TODO How handle the type here?

        match iter::from_fn(|| options_iter.next()).collect::<Option<_>>() {
            None => options_iter.for_each(drop), // Consume remaining errors
            Some(values) => {
                // overwriting existing pools seems fine
                compiler
                    .random_pools
                    .insert(self.identifier.data.0.to_string(), values);
            }
        }
    }
}
impl<'source> Compile<'source> for ast::RandomFromPoolArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let Some(pool_identifier) = consume_command_arg(self.pool_identifier, compiler) else {
            return;
        };

        let Some(values) = compiler.random_pools.get_mut(pool_identifier.data.0) else {
            compiler.errors.push(Error::custom(
                "Unknown pool. Use !random_pool first".to_string(),
                pool_identifier.span,
            ));
            return;
        };

        if values.is_empty() {
            compiler.errors.push(Error::custom(
                "Pool already empty".to_string(),
                self.identifier.span_start()..pool_identifier.span_end(),
            ));
            return;
        }

        let index = compiler.rng.gen_range(0..values.len());
        let chosen = values.swap_remove(index);

        compiler.variables.insert(self.identifier.data, chosen);
    }
}
