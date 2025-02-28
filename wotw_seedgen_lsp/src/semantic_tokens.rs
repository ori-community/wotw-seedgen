use std::{mem, ops::Range};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};
use wotw_seedgen_seed_language::ast::{
    Action, ActionCondition, AddArgs, Annotation, BuiltinIconArgs, BundleIconArgs,
    ChangeItemPoolArgs, Command, CommandIf, CommandRepeat, ConfigArgs, Content, CountInZoneArgs,
    CountInZoneBinding, Delimited, Event, EventArgs, ExportArgs, Expression, ExpressionValue,
    FunctionCall, FunctionDefinition, IncludeArgs, ItemDataArgs, ItemDataDescriptionArgs,
    ItemDataIconArgs, ItemDataNameArgs, ItemDataPriceArgs, ItemOnArgs, LetArgs, Literal,
    OnEventArgs, Once, Operation, PreplaceArgs, Punctuated, RandomFloatArgs, RandomFromPoolArgs,
    RandomIntegerArgs, RandomNumberArgs, RandomPoolArgs, Recoverable, RemoveArgs, Result,
    Separated, SeparatedNonEmpty, SetLogicStateArgs, Snippet, Span, Spanned, SpawnArgs, StateArgs,
    TagsArg, TimerArgs, Trigger, TriggerBinding, UberIdentifier, ZoneOfArgs,
};

use crate::convert;

pub fn semantic_tokens(source: &str, ast: Result<Snippet>) -> Vec<SemanticToken> {
    let mut builder = TokenBuilder::new(source);
    ast.tokens(&mut builder);
    builder.finish()
}

pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::TYPE,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::MACRO,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::OPERATOR,
        ],
        token_modifiers: vec![],
    }
}

#[repr(u32)]
enum TokenType {
    Type,
    Variable,
    EnumMember,
    Function,
    Macro,
    Keyword,
    String,
    Number,
    Operator,
}

struct TokenBuilder<'source> {
    source: &'source str,
    tokens: Vec<SemanticToken>,
    previous_line: usize,
    previous_offset: usize,
}
impl<'source> TokenBuilder<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            tokens: Default::default(),
            previous_line: Default::default(),
            previous_offset: Default::default(),
        }
    }

    fn push_token(&mut self, span: Range<usize>, token_type: TokenType) {
        let (line, line_start) = convert::last_line(&self.source[..span.start]);

        let delta_line = (line - mem::replace(&mut self.previous_line, line)) as u32;
        let previous_offset = if delta_line == 0 {
            self.previous_offset
        } else {
            line_start
        };
        self.previous_offset = span.start;
        let delta_start = self.source[previous_offset..span.start]
            .encode_utf16()
            .count() as u32;
        let length = self.source[span].encode_utf16().count() as u32;

        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: token_type as u32,
            token_modifiers_bitset: 0,
        })
    }

    fn finish(self) -> Vec<SemanticToken> {
        self.tokens
    }
}

trait Tokens {
    fn tokens(self, builder: &mut TokenBuilder);
}

impl<T: Tokens> Tokens for Vec<T> {
    fn tokens(self, builder: &mut TokenBuilder) {
        for t in self {
            t.tokens(builder);
        }
    }
}
impl<T: Tokens> Tokens for Box<T> {
    fn tokens(self, builder: &mut TokenBuilder) {
        (*self).tokens(builder)
    }
}
impl<T: Tokens> Tokens for Result<T> {
    fn tokens(self, builder: &mut TokenBuilder) {
        if let Ok(t) = self {
            t.tokens(builder);
        }
    }
}
impl<T: Tokens, R> Tokens for Recoverable<T, R> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.result.tokens(builder);
    }
}
impl<const OPEN: char, Content: Tokens, const CLOSE: char> Tokens
    for Delimited<OPEN, Content, CLOSE>
{
    fn tokens(self, builder: &mut TokenBuilder) {
        self.content.tokens(builder);
    }
}
impl<V: Tokens> Tokens for Once<V> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.0.tokens(builder);
    }
}
impl<Item: Tokens, const PUNCTUATION: char> Tokens for Punctuated<Item, PUNCTUATION> {
    fn tokens(self, builder: &mut TokenBuilder) {
        for item in self {
            item.tokens(builder);
        }
    }
}
impl<Item: Tokens, Separator> Tokens for Separated<Item, Separator> {
    fn tokens(self, builder: &mut TokenBuilder) {
        for item in self {
            item.tokens(builder);
        }
    }
}
impl<Item: Tokens, Separator> Tokens for SeparatedNonEmpty<Item, Separator> {
    fn tokens(self, builder: &mut TokenBuilder) {
        for item in self {
            item.tokens(builder);
        }
    }
}

impl<'source> Tokens for Spanned<&'source str> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.span, TokenType::String);
    }
}
impl Tokens for UberIdentifier<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.span(), TokenType::Number);
    }
}
impl Tokens for Expression<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            Expression::Value(value) => value.tokens(builder),
            Expression::Operation(operation) => operation.tokens(builder),
        }
    }
}
impl Tokens for ExpressionValue<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            ExpressionValue::Group(group) => group.tokens(builder),
            ExpressionValue::Action(action) => action.tokens(builder),
            ExpressionValue::Literal(literal) => literal.tokens(builder),
            ExpressionValue::Identifier(identifier) => {
                builder.push_token(identifier.span, TokenType::Variable);
            }
        }
    }
}
impl Tokens for Spanned<Literal<'_>> {
    fn tokens(self, builder: &mut TokenBuilder) {
        let token_type = match self.data {
            Literal::UberIdentifier(_) | Literal::Integer(_) | Literal::Float(_) => {
                TokenType::Number
            }
            Literal::Boolean(_) => TokenType::Keyword,
            Literal::String(_) => TokenType::String,
            Literal::Constant(_) => TokenType::EnumMember,
        };
        builder.push_token(self.span, token_type);
    }
}
impl Tokens for Operation<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.left.tokens(builder);
        builder.push_token(self.operator.span, TokenType::Operator);
        self.right.tokens(builder);
    }
}
impl Tokens for Snippet<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.contents.tokens(builder);
    }
}
impl Tokens for Content<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            Content::Event(keyword, event) => {
                builder.push_token(keyword.span, TokenType::Keyword);
                event.tokens(builder);
            }
            Content::Command(symbol, command) => {
                builder.push_token(symbol.span, TokenType::Macro);
                command.tokens(builder);
            }
            Content::Function(keyword, function) => {
                builder.push_token(keyword.span, TokenType::Keyword);
                function.tokens(builder);
            }
            Content::Annotation(symbol, annotation) => {
                builder.push_token(symbol.span, TokenType::Macro);
                annotation.tokens(builder);
            }
        }
    }
}
impl Tokens for Event<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.trigger.tokens(builder);
        self.action.tokens(builder);
    }
}
impl Tokens for Trigger<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            Trigger::ClientEvent(client_event) => {
                builder.push_token(client_event.span, TokenType::EnumMember);
            }
            Trigger::Binding(keyword, binding) => {
                builder.push_token(keyword.span, TokenType::Keyword);
                binding.tokens(builder);
            }
            Trigger::Condition(expression) => expression.tokens(builder),
        }
    }
}
impl Tokens for TriggerBinding<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            TriggerBinding::UberIdentifier(uber_identifier) => uber_identifier.tokens(builder),
            TriggerBinding::Identifier(identifier) => {
                builder.push_token(identifier.span, TokenType::Variable);
            }
        }
    }
}
impl Tokens for Action<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            Action::Condition(keyword, condition) => {
                builder.push_token(keyword.span, TokenType::Keyword);
                condition.tokens(builder);
            }
            Action::Function(function) => function.tokens(builder),
            Action::Multi(multi) => multi.tokens(builder),
        }
    }
}
impl Tokens for ActionCondition<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.condition.tokens(builder);
        self.action.tokens(builder);
    }
}
impl Tokens for FunctionCall<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Function);
        self.parameters.tokens(builder);
    }
}
impl Tokens for Command<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            Command::Include(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::BundleIcon(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::BuiltinIcon(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Event(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::OnEvent(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Export(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Spawn(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Tags(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Config(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::State(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Timer(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Let(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::If(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Repeat(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Add(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Remove(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::ItemData(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::ItemDataName(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::ItemDataPrice(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::ItemDataDescription(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::ItemDataIcon(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::SetLogicState(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::Preplace(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::ZoneOf(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::ItemOn(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::CountInZone(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::RandomInteger(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::RandomFloat(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::RandomPool(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Command::RandomFromPool(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
        }
    }
}
impl Tokens for IncludeArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.path.tokens(builder);
        if let Some((_, imports)) = self.imports.data {
            for import in imports {
                builder.push_token(import.span, TokenType::Variable);
            }
        }
    }
}
impl Tokens for BundleIconArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.path.tokens(builder);
    }
}
impl Tokens for BuiltinIconArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.path.tokens(builder);
    }
}
impl Tokens for EventArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.0.span, TokenType::Variable);
    }
}
impl Tokens for OnEventArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.snippet_name.tokens(builder);
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.action.tokens(builder);
    }
}
impl Tokens for ExportArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.0.span, TokenType::Variable);
    }
}
impl Tokens for SpawnArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.x.tokens(builder);
        self.y.tokens(builder);
    }
}
impl Tokens for TagsArg<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.0.tokens(builder);
    }
}
impl Tokens for ConfigArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.description.tokens(builder);
        builder.push_token(self.ty.span, TokenType::Type);
        self.default.tokens(builder);
    }
}
impl Tokens for StateArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        builder.push_token(self.ty.span, TokenType::Type);
    }
}
impl Tokens for TimerArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.toggle_identifier.span, TokenType::Variable);
        builder.push_token(self.timer_identifier.span, TokenType::Variable);
    }
}
impl Tokens for LetArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.value.tokens(builder);
    }
}
impl Tokens for CommandIf<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.condition.tokens(builder);
        self.contents.tokens(builder);
    }
}
impl Tokens for CommandRepeat<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.amount.tokens(builder);
        self.contents.tokens(builder);
    }
}
impl Tokens for AddArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.0.tokens(builder);
    }
}
impl Tokens for RemoveArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.0.tokens(builder);
    }
}
impl Tokens for ChangeItemPoolArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.item.tokens(builder);
        self.amount.tokens(builder);
    }
}
impl Tokens for ItemDataArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.item.tokens(builder);
        self.name.tokens(builder);
        self.price.tokens(builder);
        self.description.tokens(builder);
        self.icon.tokens(builder);
        self.map_icon.tokens(builder);
    }
}
impl Tokens for ItemDataNameArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.item.tokens(builder);
        self.name.tokens(builder);
    }
}
impl Tokens for ItemDataPriceArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.item.tokens(builder);
        self.price.tokens(builder);
    }
}
impl Tokens for ItemDataDescriptionArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.item.tokens(builder);
        self.description.tokens(builder);
    }
}
impl Tokens for ItemDataIconArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.item.tokens(builder);
        self.icon.tokens(builder);
    }
}
impl Tokens for SetLogicStateArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.0.tokens(builder);
    }
}
impl Tokens for PreplaceArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.item.tokens(builder);
        self.zone.tokens(builder);
    }
}
impl Tokens for ZoneOfArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.item.tokens(builder);
    }
}
impl Tokens for ItemOnArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.trigger.tokens(builder);
    }
}
impl Tokens for CountInZoneArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.zone_bindings.tokens(builder);
        self.items.tokens(builder);
    }
}
impl Tokens for CountInZoneBinding<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.zone.tokens(builder);
    }
}
impl Tokens for RandomIntegerArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.0.tokens(builder);
    }
}
impl Tokens for RandomFloatArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        self.0.tokens(builder);
    }
}
impl Tokens for RandomNumberArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        self.min.tokens(builder);
        self.max.tokens(builder);
    }
}
impl Tokens for RandomPoolArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        builder.push_token(self.ty.span, TokenType::Type);
        self.values.tokens(builder);
    }
}
impl Tokens for RandomFromPoolArgs<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Variable);
        builder.push_token(self.pool_identifier.span, TokenType::Variable);
    }
}
impl Tokens for FunctionDefinition<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        builder.push_token(self.identifier.span, TokenType::Function);
        self.actions.tokens(builder);
    }
}
impl Tokens for Annotation<'_> {
    fn tokens(self, builder: &mut TokenBuilder) {
        match self {
            Annotation::Hidden(keyword) => {
                builder.push_token(keyword.span, TokenType::Macro);
            }
            Annotation::Name(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Annotation::Category(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
            Annotation::Description(keyword, args) => {
                builder.push_token(keyword.span, TokenType::Macro);
                args.tokens(builder);
            }
        }
    }
}
