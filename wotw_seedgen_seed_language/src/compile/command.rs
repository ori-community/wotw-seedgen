// TODO this module name is confusing

use super::{Compile, SharedValue, SnippetCompiler};
use crate::{
    ast::{self, UberStateType},
    output::{intermediate::Literal, CommandVoid, ItemMetadataEntry, StringOrPlaceholder, Timer},
};
use ordered_float::OrderedFloat;
use rand::Rng;
use std::{iter, mem, ops::Range, path::PathBuf};
use wotw_seedgen_assets::UberStateAlias;
use wotw_seedgen_data::{Position, UberIdentifier, Zone};
use wotw_seedgen_parse::{Error, Identifier, Result, Span};

impl<'source> Compile<'source> for ast::Command<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Command::Include(..) => { /* all preprocessed ;) */ }
            ast::Command::IncludeIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::UseIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Callback(_, command) => {
                command.compile(compiler);
            }
            ast::Command::OnCallback(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Share(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Use(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Spawn(_, command) => {
                command.compile(compiler);
            }
            ast::Command::Flag(_, command) => {
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
impl<'source> Compile<'source> for ast::IncludeIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let content = compiler
            .global
            .snippet_access
            .read_file(self.path.data.as_ref())
            .map_err(|err| Error::custom(err, self.path.span()));
        if let Some(data) = compiler.consume_result(content) {
            // TODO make sure nothing gets overwritten
            match PathBuf::from(self.path.data).file_name() {
                None => compiler
                    .errors
                    .push(Error::custom("invalid path".to_string(), self.path.span)),
                Some(name) => {
                    let name = name.to_string_lossy().to_string();
                    compiler.global.output.icons.push((name, data));
                    compiler.variables.insert(
                        self.identifier.data,
                        Literal::CustomIcon(self.path.data.to_string()),
                    );
                }
            }
        }
    }
}
impl<'source> Compile<'source> for ast::UseIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        compiler.variables.insert(
            self.identifier.data,
            Literal::IconAsset(self.path.data.to_string()),
        );
    }
}
impl<'source> Compile<'source> for ast::CallbackArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let index = compiler.global.output.command_lookup.len();
        compiler
            .global
            .output
            .command_lookup
            .push(CommandVoid::Multi { commands: vec![] });
        compiler
            .global
            .callbacks
            .entry(compiler.identifier.clone())
            .or_default()
            .insert(self.0.data.0.to_string(), index);
    }
}
impl<'source> Compile<'source> for ast::OnCallbackArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if !compiler
            .preprocessed
            .includes
            .iter()
            .any(|include| include.data == self.snippet_name.data)
        {
            compiler.errors.push(Error::custom(
                "Unknown snippet. Maybe you should !include it first?".to_string(),
                self.snippet_name.span,
            ));
            return;
        }

        let callback = compiler
            .global
            .callbacks
            .get(self.snippet_name.data)
            .and_then(|callbacks| callbacks.get(self.identifier.data.0))
            .copied();
        if callback.is_none() {
            compiler.errors.push(Error::custom(
                "Could not find callback in snippet".to_string(),
                self.identifier.span,
            ));
        }

        let span = self.action.span();
        let action = self
            .action
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, span));

        if let (Some(callback), Some(action)) = (callback, action) {
            if let CommandVoid::Multi { commands } =
                &mut compiler.global.output.command_lookup[callback]
            {
                match action {
                    CommandVoid::Multi { commands: extend } => commands.extend(extend),
                    single => commands.push(single),
                }
            }
        }
    }
}
impl<'source> Compile<'source> for ast::ShareArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let identifier = self.0.data;

        let variable = compiler.variables.get(&self.0.data);
        let function = compiler.function_indices.get(self.0.data.0);

        let value = match (variable, function) {
            (None, Some(index)) => SharedValue::Function(*index),
            (Some(var), None) => SharedValue::Literal(var.clone()),
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
            .shared_values
            .entry(compiler.identifier.clone())
            .or_default()
            .insert(identifier.0.to_string(), value);
    }
}
impl<'source> Compile<'source> for ast::UseArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let value = compiler
            .global
            .shared_values
            .get(self.snippet_name.data)
            .ok_or_else(|| {
                Error::custom("unknown snippet".to_string(), self.snippet_name.span)
                    .with_help(format!("try !include(\"{}\")", self.snippet_name.data))
            })
            .and_then(|snippet_shared_values| {
                snippet_shared_values
                    .get(self.identifier.data.0)
                    .ok_or_else(|| {
                        Error::custom(
                            "identifier not found in snippet".to_string(),
                            self.identifier.span,
                        )
                        .with_help(format!(
                            "if it exists in {}, you have to share it there: !share({})",
                            self.snippet_name.data, self.identifier.data
                        ))
                    })
            });

        match value {
            Ok(SharedValue::Function(index)) => {
                compiler
                    .preprocessed
                    .functions
                    .insert(self.identifier.data.0.to_string());
                compiler
                    .function_indices
                    .insert(self.identifier.data.0.to_string(), *index);
                // TODO is this still used?
                compiler.function_imports.insert(
                    self.identifier.data.0.to_string(),
                    self.snippet_name.data.to_string(),
                );
            }
            Ok(SharedValue::Literal(literal)) => {
                compiler
                    .variables
                    .insert(self.identifier.data, literal.clone());
            }
            Err(err) => compiler.errors.push(err),
        }
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
        } else {
            let x = self.x.evaluate(compiler);
            let y = self.y.evaluate(compiler);
            if let (Some(x), Some(y)) = (x, y) {
                compiler.global.output.spawn = Some(Position { x, y });
            }
        }
    }
}
impl<'source> Compile<'source> for ast::FlagArg<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        if let Some(flag) = self.0.evaluate(compiler) {
            compiler.global.output.tags.push(flag);
        }
    }
}
impl<'source> Compile<'source> for ast::ConfigArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let config = compiler
            .global
            .config
            .get(&compiler.identifier)
            .and_then(|config| config.get(self.identifier.data.0));
        let value = match config {
            None => self.default.evaluate(compiler),
            Some(value) => {
                let parsed = match self.ty.data {
                    ast::UberStateType::Boolean => value.parse().ok().map(Literal::Boolean),
                    ast::UberStateType::Integer => value.parse().ok().map(Literal::Integer),
                    ast::UberStateType::Float => value.parse().ok().map(Literal::Float),
                };
                if parsed.is_none() {
                    compiler.errors.push(Error::custom(
                    format!("failed to parse provided configuration value \"{}\" as a {}, which is the required type for this configuration parameter", value, self.ty.data),
                    self.ty.span,
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
        let span = self.identifier.span.start..self.ty.span.end;
        let uber_identifier = match self.ty.data {
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
        let timer = float_uber_state(compiler, self.timer_identifier.span);
        let timer = compiler.consume_result(timer);

        if let (Some(toggle), Some(timer)) = (toggle, timer) {
            compiler.global.output.timers.push(Timer { toggle, timer });
            compiler.variables.insert(
                self.toggle_identifier.data,
                Literal::UberIdentifier(UberStateAlias {
                    uber_identifier: toggle,
                    value: None,
                }),
            );
            compiler.variables.insert(
                self.timer_identifier.data,
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
        if let Some(value) = self.value.evaluate(compiler) {
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
    let amount = args.amount.evaluate::<i32>(compiler);
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
        let name = self.name.evaluate(compiler);
        let price = self.price.compile_into(compiler);
        let description = self.description.compile_into(compiler);
        let icon = self.icon.compile_into(compiler);
        let map_icon = self.map_icon.compile_into(compiler);

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
        let name = self.name.evaluate(compiler);
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
        let price = self.price.compile_into(compiler);

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
        let description = self.description.compile_into(compiler);

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
        let icon = self.icon.compile_into(compiler);

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
        let zone = self.zone.evaluate(compiler);

        if let (Some(item), Some(zone)) = (item, zone) {
            compiler.global.output.preplacements.push((item, zone));
        }
    }
}
impl<'source> Compile<'source> for ast::ZoneOfArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
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
        if let Some(trigger) = self.trigger.compile(compiler) {
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
        let items = compiler
            .consume_result(self.items.content)
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
        self.zone
            .evaluate(compiler)
            .map(|zone| (self.identifier.data, zone))
    }
}
impl<'source> Compile<'source> for ast::RandomIntegerArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let min = self.0.min.evaluate(compiler);
        let max = self.0.max.evaluate(compiler);
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
        let min = self.0.min.evaluate::<OrderedFloat<f32>>(compiler);
        let max = self.0.max.evaluate::<OrderedFloat<f32>>(compiler);
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
        let mut options_iter = compiler
            .consume_result(self.values.content)
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
        let values = match compiler.random_pools.get_mut(self.pool_identifier.data.0) {
            None => {
                compiler.errors.push(Error::custom(
                    "Unknown pool. Use !random_pool first".to_string(),
                    self.pool_identifier.span,
                ));
                return;
            }
            Some(values) => values,
        };

        if values.is_empty() {
            compiler
                .errors
                .push(Error::custom("Pool already empty".to_string(), self.span()));
            return;
        }

        let index = compiler.rng.gen_range(0..values.len());
        let chosen = values.swap_remove(index);

        compiler.variables.insert(self.identifier.data, chosen);
    }
}
