// TODO this module name is confusing

use super::{Compile, ExportedValue, SnippetCompiler};
use crate::{
    assets::UberStateAlias,
    seed_language::{
        ast::{self, get_command_arg, UberStateType},
        compile::{self, ids::IdMap, FunctionSignature},
        output::{
            CommandVoid, ContainedWrites, Event, ItemMetadataEntry, Literal, StringOrPlaceholder,
            VariableValue,
        },
    },
    Position, UberIdentifier, Zone,
};
use ordered_float::OrderedFloat;
use rand::Rng;
use std::{iter, mem, ops::Range};
use wotw_seedgen_parse::{Error, Identifier, Result, Span, SpanEnd, SpanStart, SpannedOption};

impl<'source> Compile<'source> for ast::Command<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        match self {
            ast::Command::Include(_, command) => {
                command.compile(compiler);
            }
            ast::Command::IncludeIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::BuiltinIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::AugmentFun(_, command) => {
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
            ast::Command::SetConfig(_, command) => {
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
            ast::Command::AddItem(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RemoveItem(_, command) => {
                command.compile(compiler);
            }
            ast::Command::AddSpiritLight(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RemoveSpiritLight(_, command) => {
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
            ast::Command::ItemDataMapIcon(_, command) => {
                command.compile(compiler);
            }
            ast::Command::RemoveLocation(_, command) => {
                command.compile(compiler);
            }
            ast::Command::LocationSlots(_, command) => {
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

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let Some(snippet_exported_values) = compiler.global.exported_values.get(self.path.data)
        else {
            return;
        };

        if let Some((_, imports)) = self.imports.data {
            for import in imports {
                if let SpannedOption::Some(import) = import.value {
                    let Some(value) = snippet_exported_values.get(import.identifier.data.0) else {
                        compiler.errors.push(
                            Error::error(
                                "identifier not found in snippet".to_string(),
                                import.identifier.span,
                            )
                            .with_help(format!(
                                "if it exists in {}, you have to export it there: !export({})",
                                self.path.data, import.identifier.data
                            )),
                        );

                        continue;
                    };

                    let identifier = import
                        .rename
                        .into_option()
                        .map_or(import.identifier, |(_, identifier)| identifier);

                    match value {
                        ExportedValue::Function(function) => {
                            compiler
                                .preprocessed
                                .functions
                                .insert(identifier.data.0.to_string(), function.clone());
                        }
                        ExportedValue::Literal(literal) => {
                            compiler
                                .scopes
                                .define_variable(identifier.data.0, literal.clone());
                        }
                    }
                }
            }
        }
    }
}

impl<'source> Compile<'source> for ast::IncludeIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let Some(path) = get_command_arg(self.path) else {
            return;
        };

        let content = compiler
            .global
            .snippet_access
            .read_file(path.data.as_ref())
            .map_err(|err| Error::error(err, path.span()));

        if let Some(data) = compiler.consume_result(content) {
            compiler
                .global
                .output
                .assets
                .icons
                .push((path.data.to_string(), data));

            compiler.define_variable(
                self.identifier.data,
                Literal::CustomIcon(path.data.to_string()),
            );
        }
    }
}

impl<'source> Compile<'source> for ast::BuiltinIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let Some(path) = get_command_arg(self.path) else {
            return;
        };

        compiler.define_variable(
            self.identifier.data,
            Literal::IconAsset(path.data.to_string()),
        );
    }
}

impl<'source> Compile<'source> for ast::AugmentFunArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        fn original_signature_help(
            identifier: Identifier,
            signature: &FunctionSignature,
        ) -> String {
            format!("Original signature: {identifier}{signature}")
        }

        let function = compiler.resolve_function(&self.identifier).cloned();
        let Some(function) = function else { return };

        if function.signature.return_ty.is_some() {
            compiler.errors.push(Error::error(
                // TODO oh no our error message casing is inconsistent D:
                "Cannot augment function with return value".to_string(),
                self.identifier.span_start()..self.signature.span_end(),
            ));
        }

        if let Some(signature) = self.signature.content {
            if signature.len() != function.signature.args.len() {
                compiler.errors.push(
                    Error::error(
                        format!(
                            "Original signature had {} argument{}",
                            function.signature.args.len(),
                            if function.signature.args.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ),
                        self.signature.open.span_start()..self.signature.close.span_end(),
                    )
                    .with_help(original_signature_help(
                        self.identifier.data,
                        &function.signature,
                    )),
                );
            }

            for (definition_arg, augmentation_arg) in
                function.signature.args.iter().zip(signature.iter())
            {
                if definition_arg.identifier != augmentation_arg.identifier.data.0 {
                    compiler.errors.push(
                        Error::error(
                            format!("Original identifier was \"{}\"", definition_arg.identifier),
                            augmentation_arg.identifier.span.clone(),
                        )
                        .with_help(original_signature_help(
                            self.identifier.data,
                            &function.signature,
                        )),
                    );
                }

                if definition_arg.ty != augmentation_arg.ty.data {
                    compiler.errors.push(
                        Error::error(
                            format!("Original type was \"{}\"", definition_arg.ty),
                            augmentation_arg.ty.span.clone(),
                        )
                        .with_help(original_signature_help(
                            self.identifier.data,
                            &function.signature,
                        )),
                    );
                }
            }
        }

        let action = get_command_arg(self.action);
        let Some(action) = action else { return };

        compiler.scopes.push_function(&function.signature);

        let span = action.span();
        let action = action
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, span));

        compiler.scopes.pop();

        let Some(action) = action else { return };

        let function = &mut compiler.global.output.commands.lookup[function.index];

        match (function, action) {
            (CommandVoid::Multi { commands }, CommandVoid::Multi { commands: mut more }) => {
                commands.append(&mut more)
            }
            (CommandVoid::Multi { commands }, action) => commands.push(action),
            (function, CommandVoid::Multi { mut commands }) => {
                let head = mem::replace(function, compile::empty());

                commands.insert(0, head);

                *function = CommandVoid::Multi { commands };
            }
            (function, action) => {
                let head = mem::replace(function, compile::empty());

                let commands = vec![head, action];

                *function = CommandVoid::Multi { commands };
            }
        }
    }
}

impl<'source> Compile<'source> for ast::ExportArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let identifier = self.0.data;

        let variable = compiler.scopes.resolve_variable(self.0.data.0);
        let function = compiler.preprocessed.functions.get(self.0.data.0);

        let value = match (variable, function) {
            (None, Some(function)) => ExportedValue::Function(function.clone()),
            (Some(VariableValue::Literal(var)), None) => ExportedValue::Literal(var.clone()),
            (Some(VariableValue::Reference(_)), None) => {
                compiler.errors.push(Error::error(
                    "Cannot export a local reference".to_string(),
                    self.0.span,
                ));
                return;
            }
            (Some(_), Some(_)) => {
                compiler.errors.push(Error::error(
                    "Could refer to either a function or a variable in the current scope. Consider renaming one of them to resolve the ambiguity".to_string(),
                    self.0.span,
                ));
                return;
            }
            (None, None) => {
                compiler.errors.push(Error::error(
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

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if compiler.global.output.preload.spawn.is_some() {
            compiler.errors.push(Error::error(
                "Multiple spawn commands".to_string(),
                self.span(),
            ));
        }

        let x = self.x.evaluate::<f32>(compiler);
        let y = get_command_arg(self.y).and_then(|y| y.evaluate::<f32>(compiler));

        let (Some(x), Some(y)) = (x, y) else { return };

        compiler.global.output.preload.spawn = Some(Position {
            x: x.into(),
            y: y.into(),
        });
    }
}

impl<'source> Compile<'source> for ast::TagsArg<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if let Some(tag) = self.0.evaluate(compiler) {
            compiler.global.output.preload.tags.push(tag);
        }
    }
}

impl<'source> Compile<'source> for ast::ConfigArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let ty = get_command_arg(self.ty);
        let default = get_command_arg(self.default);

        let (Some(ty), Some(default)) = (ty, default) else {
            return;
        };

        if default.data.ty() != ty.data.into() {
            compiler
                .errors
                .push(Error::error(format!("expected {}", ty.data), default.span));
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
                    compiler.errors.push(Error::error(
                        format!("failed to parse provided configuration value \"{}\" as a {}, which is the required type for this configuration parameter", value, ty.data),
                        ty.span,
                    ));
                }

                parsed
            }
        };
        if let Some(value) = value {
            compiler.define_variable(self.identifier.data, value);
        }
    }
}

impl<'source> Compile<'source> for ast::SetConfigArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        // setting the config happens in preprocessor
        compiler.check_snippet_included(&self.snippet_name);

        // TODO verify identifier exists?
    }
}

impl<'source> Compile<'source> for ast::StateArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.span();

        let Some(ty) = get_command_arg(self.ty) else {
            return;
        };

        let uber_identifier = match ty.data {
            UberStateType::Boolean => boolean_uber_state(compiler, self.identifier.data.0, span),
            UberStateType::Integer => integer_uber_state(compiler, self.identifier.data.0, span),
            UberStateType::Float => float_uber_state(compiler, self.identifier.data.0, span),
        };

        if let Some(uber_identifier) = compiler.consume_result(uber_identifier) {
            compiler.define_variable(
                self.identifier.data,
                UberStateAlias::regular(uber_identifier),
            );
        }
    }
}

impl<'source> Compile<'source> for ast::TimerArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let toggle = boolean_uber_state(
            compiler,
            self.toggle_identifier.data.0,
            self.toggle_identifier.span,
        );
        let toggle = compiler.consume_result(toggle);

        let Some(timer_identifier) = get_command_arg(self.timer_identifier) else {
            return;
        };
        let timer = float_uber_state(compiler, timer_identifier.data.0, timer_identifier.span);
        let timer = compiler.consume_result(timer);

        if let (Some(toggle), Some(timer)) = (toggle, timer) {
            compiler
                .global
                .output
                .commands
                .events
                .push(Event::on_reload(CommandVoid::DefineTimer { toggle, timer }));

            compiler.define_variable(self.toggle_identifier.data, UberStateAlias::regular(toggle));
            compiler.define_variable(timer_identifier.data, UberStateAlias::regular(timer));
        }
    }
}

fn boolean_uber_state<S: Span>(
    compiler: &mut SnippetCompiler,
    identifier: &str,
    span: S,
) -> Result<UberIdentifier> {
    uber_state::<8, 100>(
        &mut compiler.global.id_resolver.boolean_state,
        &compiler.identifier,
        identifier,
        span,
    )
}

fn integer_uber_state<S: Span>(
    compiler: &mut SnippetCompiler,
    identifier: &str,
    span: S,
) -> Result<UberIdentifier> {
    uber_state::<9, 100>(
        &mut compiler.global.id_resolver.integer_state,
        &compiler.identifier,
        identifier,
        span,
    )
}

fn float_uber_state<S: Span>(
    compiler: &mut SnippetCompiler,
    identifier: &str,
    span: S,
) -> Result<UberIdentifier> {
    uber_state::<10, 25>(
        &mut compiler.global.id_resolver.float_state,
        &compiler.identifier,
        identifier,
        span,
    )
}

fn uber_state<const GROUP: i32, const AVAILABLE: usize>(
    ids: &mut IdMap<0>,
    snippet_identifier: &str,
    state_identifier: &str,
    span: impl Span,
) -> Result<UberIdentifier> {
    let id = ids.id(format!("{snippet_identifier}_{state_identifier}"));

    if id < AVAILABLE {
        Ok(UberIdentifier {
            group: GROUP,
            member: id as i32,
        })
    } else {
        Err(Error::error(format!("Only {AVAILABLE} UberStates of this type are available (What on earth are you doing?)"), span.span()))
    }
}

impl<'source> Compile<'source> for ast::LetArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if let Some(value) =
            get_command_arg(self.value).and_then(|value| value.evaluate::<Literal>(compiler))
        {
            compiler.define_variable(self.identifier.data, value);
        }
    }
}

impl<'source> Compile<'source> for ast::CommandIf<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if let Some(true) = self.condition.evaluate(compiler) {
            self.contents.compile(compiler);
        }
    }
}

impl<'source> Compile<'source> for ast::CommandRepeat<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.amount.span();

        if let Some(repetitions) = self.amount.evaluate::<i32>(compiler) {
            if repetitions < 0 {
                compiler.errors.push(Error::error(
                    format!("!repeat only allows for positive numbers, but this evaluated to {repetitions}"),
                    span,
                ));

                return;
            }

            if let Some(contents) = self.contents.content {
                for contents in iter::repeat_n(contents, repetitions as usize) {
                    // short circuit on errors to avoid adding the same errors repeatedly
                    if contents.compile(compiler).contains(&None) {
                        break;
                    }
                }
            }
        }
    }
}

impl<'source> Compile<'source> for ast::AddItemArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        compile_item_pool_change::<1>(self.0, compiler)
    }
}

impl<'source> Compile<'source> for ast::RemoveItemArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        compile_item_pool_change::<-1>(self.0, compiler)
    }
}

fn compile_item_pool_change<'source, const FACTOR: i32>(
    args: ast::ChangeItemPoolArgs<'source>,
    compiler: &mut SnippetCompiler<'source, '_, '_, '_>,
) {
    let span = args.item.span();
    let item = args
        .item
        .compile(compiler)
        .and_then(|command| command.expect_void(compiler, span));

    let amount = get_command_arg(args.amount).and_then(|amount| amount.evaluate::<i32>(compiler));

    if let (Some(item), Some(amount)) = (item, amount) {
        *compiler
            .global
            .output
            .modifiers
            .item_pool_changes
            .entry(item)
            .or_default() += amount * FACTOR;
    }
}

impl<'source> Compile<'source> for ast::AddSpiritLightArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if let Some(amount) = self.0.evaluate::<i32>(compiler) {
            compiler.global.output.modifiers.spirit_light_change += amount;
        }
    }
}

impl<'source> Compile<'source> for ast::RemoveSpiritLightArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if let Some(amount) = self.0.evaluate::<i32>(compiler) {
            compiler.global.output.modifiers.spirit_light_change -= amount;
        }
    }
}

// TODO the practice of writing out the full item everytime seems a little dated now...
// maybe there could be a better system here that allows you to reference existing items easily, but still reference them by their full form e.g. to rename default pool items
impl<'source> Compile<'source> for ast::ItemDataArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));

        let name = get_command_arg(self.name).and_then(|name| name.evaluate(compiler));
        let shop_price = get_command_arg(self.price).and_then(|price| price.compile_into(compiler));
        let description = get_command_arg(self.description)
            .and_then(|description| description.compile_into(compiler));
        let icon = get_command_arg(self.icon).and_then(|icon| icon.compile_into(compiler));
        let map_icon =
            get_command_arg(self.map_icon).and_then(|map_icon| map_icon.compile_into(compiler));

        if let Some(item) = item {
            if compiler
                .global
                .output
                .modifiers
                .item_metadata
                .0
                .insert(
                    item,
                    ItemMetadataEntry {
                        name,
                        shop_price,
                        description,
                        icon,
                        map_icon,
                    },
                )
                .is_some()
            {
                compiler.errors.push(Error::error(
                    "Already defined data for this item".to_string(),
                    span,
                ));
            }
        }
    }
}

impl<'source> Compile<'source> for ast::ItemDataNameArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));

        let name = get_command_arg(self.name).and_then(|name| name.evaluate(compiler));

        if let (Some(item), Some(name)) = (item, name) {
            insert_item_data(compiler, item, span, name, "name", |data| &mut data.name);
        }
    }
}

impl<'source> Compile<'source> for ast::ItemDataPriceArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));

        let price = get_command_arg(self.price).and_then(|price| price.compile_into(compiler));

        if let (Some(item), Some(price)) = (item, price) {
            insert_item_data(compiler, item, span, price, "price", |data| {
                &mut data.shop_price
            });
        }
    }
}

impl<'source> Compile<'source> for ast::ItemDataDescriptionArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));

        let description = get_command_arg(self.description)
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

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));

        let icon = get_command_arg(self.icon).and_then(|icon| icon.compile_into(compiler));

        if let (Some(item), Some(icon)) = (item, icon) {
            insert_item_data(compiler, item, span, icon, "icon", |data| &mut data.icon);
        }
    }
}

// TODO these related impls are pretty similar (same for other impls on those types)
impl<'source> Compile<'source> for ast::ItemDataMapIconArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, &span));

        let map_icon =
            get_command_arg(self.map_icon).and_then(|map_icon| map_icon.compile_into(compiler));

        if let (Some(item), Some(map_icon)) = (item, map_icon) {
            insert_item_data(compiler, item, span, map_icon, "map_icon", |data| {
                &mut data.map_icon
            });
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
    if f(compiler
        .global
        .output
        .modifiers
        .item_metadata
        .0
        .entry(item)
        .or_default())
    .replace(value)
    .is_some()
    {
        compiler.errors.push(Error::error(
            format!("Already defined {field} for this item"),
            span,
        ))
    }
}

impl<'source> Compile<'source> for ast::RemoveLocationArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if let Some(condition) = self.condition.compile_into(compiler) {
            compiler
                .global
                .output
                .modifiers
                .removed_locations
                .insert(condition);
        }
    }
}

impl<'source> Compile<'source> for ast::LocationSlotsArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let location = self.location.compile_into(compiler);
        let slots =
            get_command_arg(self.slots).and_then(|slots| slots.compile_into::<i32>(compiler));

        if let (Some(location), Some(slots)) = (location, slots) {
            compiler
                .global
                .output
                .modifiers
                .location_slots
                .insert(location, slots.try_into().unwrap_or_default());
        }
    }
}

impl<'source> Compile<'source> for ast::SetLogicStateArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        compiler
            .global
            .output
            .modifiers
            .logical_state_sets
            .insert(self.0.data.to_string());
    }
}

impl<'source> Compile<'source> for ast::PreplaceArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let span = self.item.span();
        let item = self
            .item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, span));

        let zone = get_command_arg(self.zone).and_then(|zone| zone.evaluate(compiler));

        if let (Some(item), Some(zone)) = (item, zone) {
            compiler
                .global
                .output
                .modifiers
                .preplacements
                .push((item, zone));
        }
    }
}

impl<'source> Compile<'source> for ast::ZoneOfArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let Some(item) = get_command_arg(self.item) else {
            return;
        };

        let span = item.span();
        let item = item
            .compile(compiler)
            .and_then(|command| command.expect_void(compiler, span));

        if let Some(item) = item {
            let write_identifiers = item.contained_write_identifiers().collect();

            compiler.define_variable(
                self.identifier.data,
                Literal::String(StringOrPlaceholder::ZoneOfPlaceholder(write_identifiers)),
            );
        }
    }
}

impl<'source> Compile<'source> for ast::ItemOnArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        if let Some(trigger) =
            get_command_arg(self.trigger).and_then(|trigger| trigger.compile(compiler))
        {
            compiler.define_variable(
                self.identifier.data,
                Literal::String(StringOrPlaceholder::ItemOnPlaceholder(Box::new(trigger))),
            );
        }
    }
}

impl<'source> Compile<'source> for ast::CountInZoneArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let zone_bindings = self
            .zone_bindings
            .compile(compiler)
            .into_iter()
            .flatten()
            .flatten()
            .flatten();

        let Some(items) = get_command_arg(self.items) else {
            return;
        };

        let items = items.content.into_iter().flatten().filter_map(|action| {
            let span = action.span();
            action
                .compile(compiler)
                .and_then(|command| command.expect_void(compiler, span))
        });

        let mut write_identifiers = vec![];
        for item in items {
            for uber_identifier in item.contained_write_identifiers() {
                write_identifiers.push(uber_identifier);
            }
        }

        for (identifier, zone) in zone_bindings {
            compiler.define_variable(
                identifier,
                Literal::String(StringOrPlaceholder::CountInZonePlaceholder(
                    write_identifiers.clone(),
                    zone,
                )),
            );
        }
    }
}

impl<'source> Compile<'source> for ast::CountInZoneBinding<'source> {
    type Output = Option<(Identifier<'source>, Zone)>;

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        get_command_arg(self.zone)
            .and_then(|zone| zone.evaluate(compiler))
            .map(|zone| (self.identifier.data, zone))
    }
}

impl<'source> Compile<'source> for ast::RandomIntegerArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let min = get_command_arg(self.0.min).and_then(|min| min.evaluate(compiler));
        let max = get_command_arg(self.0.max).and_then(|max| max.evaluate(compiler));

        if let (Some(min), Some(max)) = (min, max) {
            let value: i32 = compiler.rng.gen_range(min..=max);

            compiler.define_variable(self.0.identifier.data, value);
        }
    }
}

impl<'source> Compile<'source> for ast::RandomFloatArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let min = get_command_arg(self.0.min).and_then(|min| min.evaluate::<f32>(compiler));
        let max = get_command_arg(self.0.max).and_then(|max| max.evaluate::<f32>(compiler));

        if let (Some(min), Some(max)) = (min, max) {
            let value: f32 = compiler.rng.gen_range(min.into()..=max.into());

            compiler.define_variable(self.0.identifier.data, OrderedFloat(value));
        }
    }
}

impl<'source> Compile<'source> for ast::RandomPoolArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let Some(values) = get_command_arg(self.values) else {
            return;
        };

        let mut options_iter = values
            .content
            .into_iter()
            .flatten()
            .map(|expression| expression.evaluate(compiler));

        // TODO How handle the type here?

        match iter::from_fn(|| options_iter.next()).collect::<Option<_>>() {
            None => options_iter.for_each(drop), // Consume remaining errors
            Some(values) => {
                // overwriting existing pools seems fine
                compiler
                    .scopes
                    .define_random_pool(self.identifier.data, values);
            }
        }
    }
}

impl<'source> Compile<'source> for ast::RandomFromPoolArgs<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        let Some(pool_identifier) = get_command_arg(self.pool_identifier) else {
            return;
        };

        let Some(values) = compiler.scopes.resolve_random_pool(pool_identifier.data) else {
            compiler.errors.push(Error::error(
                "Unknown pool. Use !random_pool first".to_string(),
                pool_identifier.span,
            ));
            return;
        };

        if values.is_empty() {
            compiler.errors.push(Error::error(
                "Pool already empty".to_string(),
                self.identifier.span_start()..pool_identifier.span_end(),
            ));
            return;
        }

        let index = compiler.rng.gen_range(0..values.len());
        let chosen = values.swap_remove(index);

        compiler.define_variable(self.identifier.data, chosen);
    }
}
