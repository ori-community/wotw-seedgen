use std::ops::Range;

use wotw_seedgen_parse::{
    Delimited, Identifier, Once, Punctuated, Recoverable, SeparatedNonEmpty, Spanned, SpannedOption,
};

use crate::seed_language::ast::{
    Action, ActionCondition, AddItemArgs, Annotation, AugmentFunArgs, BuiltinIconArgs,
    ChangeItemPoolArgs, Command, CommandArg, CommandArgs, CommandIf, CommandRepeat, ConfigArgs,
    Content, CountInZoneArgs, CountInZoneBinding, Event, ExportArgs, Expression, ExpressionValue,
    FunctionCall, FunctionDefinition, IncludeArgs, IncludeIconArgs, ItemDataArgs,
    ItemDataDescriptionArgs, ItemDataIconArgs, ItemDataMapIconArgs, ItemDataNameArgs,
    ItemDataPriceArgs, ItemOnArgs, LetArgs, Literal, Operation, PreplaceArgs, RandomFloatArgs,
    RandomFromPoolArgs, RandomIntegerArgs, RandomNumberArgs, RandomPoolArgs, RemoveItemArgs,
    RemoveLocationArgs, SetConfigArgs, SetLogicStateArgs, Snippet, SpawnArgs, StateArgs, TagsArg,
    TimerArgs, Trigger, TriggerBinding, UberIdentifier, ZoneOfArgs,
};

#[must_use]
pub fn get_command_arg<T>(arg: CommandArg<T>) -> Option<T> {
    match arg {
        Recoverable {
            value:
                SpannedOption::Some((
                    _,
                    Recoverable {
                        value: SpannedOption::Some(t),
                        ..
                    },
                )),
            ..
        } => Some(t),
        _ => None,
    }
}

#[must_use]
pub fn get_command_arg_ref<T>(arg: &CommandArg<T>) -> Option<&T> {
    match arg {
        Recoverable {
            value:
                SpannedOption::Some((
                    _,
                    Recoverable {
                        value: SpannedOption::Some(t),
                        ..
                    },
                )),
            ..
        } => Some(t),
        _ => None,
    }
}

pub fn inspect_command_arg<T, F: FnOnce(&T)>(arg: &CommandArg<T>, f: F) {
    get_command_arg_ref(arg).inspect(|t| f(*t));
}

#[must_use]
pub fn get_command_args_ref<T>(args: &CommandArgs<T>) -> Option<&T> {
    match args {
        Recoverable {
            value:
                SpannedOption::Some(Delimited {
                    content: Some(Once(t)),
                    ..
                }),
            ..
        } => Some(t),
        _ => None,
    }
}

pub fn inspect_command_args<T, F: FnOnce(&T)>(args: &CommandArgs<T>, f: F) {
    get_command_args_ref(args).inspect(|t| f(*t));
}

pub trait Traverse<H: Handler> {
    fn traverse(&self, handler: &mut H);
}

impl<H: Handler, T: Traverse<H>> Traverse<H> for Box<T> {
    fn traverse(&self, handler: &mut H) {
        (**self).traverse(handler);
    }
}

impl<H: Handler, T: Traverse<H>> Traverse<H> for Option<T> {
    fn traverse(&self, handler: &mut H) {
        if let Some(t) = self {
            t.traverse(handler);
        }
    }
}

impl<H: Handler, T: Traverse<H>> Traverse<H> for SpannedOption<T> {
    fn traverse(&self, handler: &mut H) {
        if let SpannedOption::Some(t) = self {
            t.traverse(handler);
        }
    }
}

impl<H: Handler, T: Traverse<H>> Traverse<H> for Vec<T> {
    fn traverse(&self, handler: &mut H) {
        for t in self {
            t.traverse(handler);
        }
    }
}

impl<H: Handler, T: Traverse<H>, Open, Close> Traverse<H> for Delimited<Open, T, Close> {
    fn traverse(&self, handler: &mut H) {
        self.content.traverse(handler);
    }
}

impl<H: Handler, T: Traverse<H>> Traverse<H> for Once<T> {
    fn traverse(&self, handler: &mut H) {
        self.0.traverse(handler);
    }
}

impl<H: Handler, T: Traverse<H>, Punctuation> Traverse<H> for Punctuated<T, Punctuation> {
    fn traverse(&self, handler: &mut H) {
        for t in self {
            t.traverse(handler);
        }
    }
}

impl<H: Handler, T: Traverse<H>, Separator> Traverse<H> for SeparatedNonEmpty<T, Separator> {
    fn traverse(&self, handler: &mut H) {
        for t in self {
            t.traverse(handler);
        }
    }
}

impl<H: Handler, T: Traverse<H>, R> Traverse<H> for Recoverable<T, R> {
    fn traverse(&self, handler: &mut H) {
        self.value.traverse(handler)
    }
}

impl<H: Handler> Traverse<H> for Expression<'_> {
    fn traverse(&self, handler: &mut H) {
        match self {
            Self::Value(value) => value.traverse(handler),
            Self::Operation(operation) => operation.traverse(handler),
        }
    }
}

impl<H: Handler> Traverse<H> for ExpressionValue<'_> {
    fn traverse(&self, handler: &mut H) {
        match self {
            Self::Group(group) => group.traverse(handler),
            Self::Action(action) => action.traverse(handler),
            Self::Literal(literal) => literal.traverse(handler),
            Self::Identifier(identifier) => handler.identifier_use(identifier),
        }
    }
}

impl<H: Handler> Traverse<H> for Spanned<Literal<'_>> {
    fn traverse(&self, handler: &mut H) {
        match &self.data {
            Literal::UberIdentifier(value) => value.traverse(handler),
            Literal::Boolean(_) => handler.boolean(&self.span),
            Literal::Integer(_) => handler.integer(&self.span),
            Literal::Float(_) => handler.float(&self.span),
            Literal::String(_) => handler.string(&self.span),
            Literal::Constant(_) => handler.constant(&self.span),
        }
    }
}

impl<H: Handler> Traverse<H> for UberIdentifier<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.uber_identifier(self)
    }
}

impl<H: Handler> Traverse<H> for Operation<'_> {
    fn traverse(&self, handler: &mut H) {
        self.left.traverse(handler);
        handler.operator(&self.operator.span);
        self.right.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for Snippet<'_> {
    fn traverse(&self, handler: &mut H) {
        self.contents.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for Content<'_> {
    fn traverse(&self, handler: &mut H) {
        match self {
            Self::Event(keyword, event) => {
                handler.keyword(&keyword.span);
                event.traverse(handler);
            }
            Self::Function(keyword, function) => {
                handler.keyword(&keyword.span);
                function.traverse(handler);
            }
            Self::Command(symbol, command) => {
                handler.command_symbol(&symbol.span);
                command.traverse(handler);
            }
            Self::Annotation(symbol, annotation) => {
                handler.annotation_symbol(&symbol.span);
                annotation.traverse(handler);
            }
        }
    }
}

impl<H: Handler> Traverse<H> for Event<'_> {
    fn traverse(&self, handler: &mut H) {
        self.trigger.traverse(handler);
        self.action.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for Trigger<'_> {
    fn traverse(&self, handler: &mut H) {
        match self {
            Self::ClientEvent(client_event) => {
                handler.constant(&client_event.span);
            }
            Self::Binding(keyword, binding) => {
                handler.keyword(&keyword.span);
                binding.traverse(handler);
            }
            Self::Condition(expression) => expression.traverse(handler),
        }
    }
}

impl<H: Handler> Traverse<H> for TriggerBinding<'_> {
    fn traverse(&self, handler: &mut H) {
        match self {
            Self::UberIdentifier(uber_identifier) => uber_identifier.traverse(handler),
            Self::Identifier(identifier) => {
                handler.identifier_use(identifier);
            }
        }
    }
}

impl<H: Handler> Traverse<H> for Action<'_> {
    fn traverse(&self, handler: &mut H) {
        match self {
            Self::Condition(keyword, condition) => {
                handler.keyword(&keyword.span);
                condition.traverse(handler);
            }
            Self::Function(function) => function.traverse(handler),
            Self::Multi(multi) => multi.traverse(handler),
        }
    }
}

impl<H: Handler> Traverse<H> for ActionCondition<'_> {
    fn traverse(&self, handler: &mut H) {
        self.condition.traverse(handler);
        self.action.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for FunctionCall<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.function_use(&self.identifier);
        self.parameters.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for FunctionDefinition<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.function_def(&self.identifier);
        self.actions.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for Command<'_> {
    fn traverse(&self, handler: &mut H) {
        match self {
            Self::Include(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::IncludeIcon(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::BuiltinIcon(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::AugmentFun(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Export(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Spawn(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Tags(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Config(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::SetConfig(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::State(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Timer(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Let(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::If(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Repeat(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::AddItem(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::RemoveItem(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ItemData(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ItemDataName(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ItemDataPrice(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ItemDataDescription(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ItemDataIcon(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ItemDataMapIcon(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::RemoveLocation(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::SetLogicState(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::Preplace(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ZoneOf(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::ItemOn(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::CountInZone(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::RandomInteger(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::RandomFloat(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::RandomPool(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
            Self::RandomFromPool(keyword, args) => {
                handler.command_keyword(&keyword.span);
                args.traverse(handler);
            }
        }
    }
}

impl<H: Handler> Traverse<H> for IncludeArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.string(&self.path.span);
        if let Some((_, imports)) = &self.imports.data {
            for import in imports {
                if let SpannedOption::Some(import) = &import.value {
                    match &import.rename {
                        SpannedOption::Some((keyword, identifier)) => {
                            handler.identifier(&import.identifier);
                            handler.keyword(&keyword.span);
                            handler.identifier_def(identifier);
                        }
                        SpannedOption::None(_) => {
                            handler.identifier_def(&import.identifier);
                        }
                    }
                }
            }
        }
    }
}

impl<H: Handler> Traverse<H> for IncludeIconArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.path, |path| handler.string(&path.span));
    }
}

impl<H: Handler> Traverse<H> for BuiltinIconArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.path, |path| handler.string(&path.span));
    }
}

impl<H: Handler> Traverse<H> for AugmentFunArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_use(&self.identifier);
        inspect_command_arg(&self.action, |action| action.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ExportArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_use(&self.0);
    }
}

impl<H: Handler> Traverse<H> for SpawnArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.x.traverse(handler);
        inspect_command_arg(&self.y, |y| y.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for TagsArg<'_> {
    fn traverse(&self, handler: &mut H) {
        self.0.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for ConfigArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.config(self);

        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.description, |description| {
            handler.string(&description.span)
        });
        inspect_command_arg(&self.ty, |ty| handler.ty(&ty.span));
        inspect_command_arg(&self.default, |default| default.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for SetConfigArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.string(&self.snippet_name.span);
        inspect_command_arg(&self.identifier, |identifier| {
            handler.identifier(identifier)
        });
        inspect_command_arg(&self.value, |value| handler.string(&value.span));
    }
}

impl<H: Handler> Traverse<H> for StateArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.ty, |ty| handler.ty(&ty.span));
    }
}

impl<H: Handler> Traverse<H> for TimerArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.toggle_identifier);
        inspect_command_arg(&self.timer_identifier, |timer_identifier| {
            handler.identifier_def(timer_identifier)
        });
    }
}

impl<H: Handler> Traverse<H> for LetArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.value, |value| value.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for CommandIf<'_> {
    fn traverse(&self, handler: &mut H) {
        self.condition.traverse(handler);
        self.contents.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for CommandRepeat<'_> {
    fn traverse(&self, handler: &mut H) {
        self.amount.traverse(handler);
        self.contents.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for AddItemArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.0.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for RemoveItemArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.0.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for ChangeItemPoolArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.amount, |amount| amount.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ItemDataArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.name, |name| name.traverse(handler));
        inspect_command_arg(&self.price, |price| price.traverse(handler));
        inspect_command_arg(&self.description, |description| {
            description.traverse(handler)
        });
        inspect_command_arg(&self.icon, |icon| icon.traverse(handler));
        inspect_command_arg(&self.map_icon, |map_icon| map_icon.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ItemDataNameArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.name, |name| name.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ItemDataPriceArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.price, |price| price.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ItemDataDescriptionArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.description, |description| {
            description.traverse(handler)
        });
    }
}

impl<H: Handler> Traverse<H> for ItemDataIconArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.icon, |icon| icon.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ItemDataMapIconArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.map_icon, |map_icon| map_icon.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for RemoveLocationArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.condition.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for SetLogicStateArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.string(&self.0.span);
    }
}

impl<H: Handler> Traverse<H> for PreplaceArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.item.traverse(handler);
        inspect_command_arg(&self.zone, |zone| zone.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ZoneOfArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.item, |item| item.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for ItemOnArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.trigger, |trigger| trigger.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for CountInZoneArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.zone_bindings.traverse(handler);
        inspect_command_arg(&self.items, |items| items.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for CountInZoneBinding<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.zone, |zone| zone.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for RandomIntegerArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.0.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for RandomFloatArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        self.0.traverse(handler);
    }
}

impl<H: Handler> Traverse<H> for RandomNumberArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.min, |min| min.traverse(handler));
        inspect_command_arg(&self.max, |max| max.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for RandomPoolArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.ty, |ty| handler.ty(&ty.span));
        inspect_command_arg(&self.values, |values| values.traverse(handler));
    }
}

impl<H: Handler> Traverse<H> for RandomFromPoolArgs<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.identifier_def(&self.identifier);
        inspect_command_arg(&self.pool_identifier, |pool_identifier| {
            handler.identifier_use(pool_identifier)
        });
    }
}

impl<H: Handler> Traverse<H> for Annotation<'_> {
    fn traverse(&self, handler: &mut H) {
        handler.annotation(self);

        match self {
            Annotation::Hidden(keyword) => {
                handler.annotation_keyword(&keyword.span);
            }
            Annotation::Name(keyword, args) => {
                handler.annotation_keyword(&keyword.span);
                inspect_command_args(args, |name| handler.string(&name.span));
            }
            Annotation::Category(keyword, args) => {
                handler.annotation_keyword(&keyword.span);
                inspect_command_args(args, |category| handler.string(&category.span));
            }
            Annotation::Description(keyword, args) => {
                handler.annotation_keyword(&keyword.span);
                inspect_command_args(args, |description| handler.string(&description.span));
            }
        }
    }
}

// TODO spans by ref or by clone?
pub trait Handler {
    fn keyword(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn command_keyword(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn annotation_keyword(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn command_symbol(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn annotation_symbol(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn operator(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn boolean(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn integer(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn float(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn string(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn constant(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn ty(&mut self, span: &Range<usize>) {
        let _ = span;
    }

    fn uber_identifier(&mut self, uber_identifier: &UberIdentifier) {
        let _ = uber_identifier;
    }

    fn identifier(&mut self, identifier: &Spanned<Identifier>) {
        let _ = identifier;
    }

    fn identifier_def(&mut self, identifier: &Spanned<Identifier>) {
        let _ = identifier;
    }

    fn identifier_use(&mut self, identifier: &Spanned<Identifier>) {
        let _ = identifier;
    }

    fn function_def(&mut self, identifier: &Spanned<Identifier>) {
        let _ = identifier;
    }

    fn function_use(&mut self, identifier: &Spanned<Identifier>) {
        let _ = identifier;
    }

    fn config(&mut self, config: &ConfigArgs) {
        let _ = config;
    }

    fn annotation(&mut self, annotation: &Annotation) {
        let _ = annotation;
    }
}
