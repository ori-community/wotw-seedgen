// TODO investigate optimizing type sizes

use std::{fmt::Display, ops::ControlFlow, str::FromStr};

use crate::{
    token::{Token, Tokenizer, TOKENIZER},
    types::Type,
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, EnumDiscriminants, VariantArray};
use wotw_seedgen_data::{
    Alignment, CoordinateSystem, EquipSlot, Equipment, GromIcon, HorizontalAnchor, LupoIcon,
    MapIcon, OpherIcon, ScreenPosition, Shard, Skill, Teleporter, TuleyIcon, VerticalAnchor,
    WeaponUpgrade, WheelBind, WheelItemPosition, Zone,
};
use wotw_seedgen_parse::{parse_ast, Error, ErrorKind, Mode};

pub use wotw_seedgen_parse::{
    Ast, Identifier, NoTrailingInput, Once, Parser, Recover, Recoverable, Result, Separated,
    SeparatedNonEmpty, Span, Spanned, Symbol,
};

pub type Delimited<const OPEN: char, Content, const CLOSE: char> =
    wotw_seedgen_parse::Delimited<Spanned<Symbol<OPEN>>, Content, Spanned<Symbol<CLOSE>>>;
pub type Punctuated<Item, const PUNCTUATION: char> =
    wotw_seedgen_parse::Punctuated<Item, Symbol<PUNCTUATION>>;

pub fn parse<'source, V>(source: &'source str) -> NoTrailingInput<V>
where
    V: Ast<'source, Tokenizer>,
{
    parse_ast(source, TOKENIZER)
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Ast)]
pub struct Snippet<'source> {
    pub contents: Vec<Recoverable<Content<'source>, RecoverContent>>,
}

pub struct RecoverContent;
impl<'source> Recover<'source, Tokenizer> for RecoverContent {
    fn recover(parser: &mut Parser<'source, Tokenizer>) {
        // TODO this can skip delimiters
        while !(parser.is_finished() || matches!(parser.current_slice(), "#" | "!" | "on" | "fun"))
        {
            parser.step()
        }
    }
}

pub struct RecoverPass;
impl<'source> Recover<'source, Tokenizer> for RecoverPass {
    fn recover(_parser: &mut Parser<'source, Tokenizer>) {}
}

pub struct RecoverSkipExpression;
impl<'source> Recover<'source, Tokenizer> for RecoverSkipExpression {
    fn recover(parser: &mut Parser<'source, Tokenizer>) {
        let _ = Expression::ast_option(parser);
    }
}

pub struct RecoverCommandArg;
impl<'source> Recover<'source, Tokenizer> for RecoverCommandArg {
    fn recover(parser: &mut Parser<'source, Tokenizer>) {
        while !(parser.is_finished() || matches!(parser.current_slice(), "," | ")")) {
            parser.step()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum Content<'source> {
    Event(Spanned<On>, Recoverable<Event<'source>, RecoverContent>),
    Function(
        Spanned<Fun>,
        Recoverable<FunctionDefinition<'source>, RecoverContent>,
    ),
    Command(
        Spanned<Symbol<'!'>>,
        Recoverable<Command<'source>, RecoverContent>,
    ),
    Annotation(
        Spanned<Symbol<'#'>>,
        Recoverable<Annotation<'source>, RecoverContent>,
    ),
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(token = Token::On)]
pub struct On;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct Event<'source> {
    pub trigger: Trigger<'source>,
    pub action: Recoverable<Action<'source>, RecoverContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum Trigger<'source> {
    ClientEvent(Spanned<ClientEvent>),
    Binding(
        Spanned<Change>,
        Recoverable<TriggerBinding<'source>, RecoverSkipExpression>,
    ),
    Condition(Expression<'source>),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Ast, Display, Serialize, Deserialize, VariantArray,
)]
#[ast(case = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClientEvent {
    /// Trigger when starting a new file
    Spawn,
    /// Trigger when starting a new file or loading the seed into an active file
    Reload,
    /// Trigger when respawning after death, void etc.
    Respawn,
    // TODO on input <bind>? Or add all bindings to the `input` uberState group so you can trigger on press/release or check their current state?
    /// Trigger on keybind
    Binding1,
    /// Trigger on keybind
    Binding2,
    /// Trigger on keybind
    Binding3,
    /// Trigger on keybind
    Binding4,
    /// Trigger on keybind
    Binding5,
    /// Trigger on the Show Progress keybind
    ProgressMessage,
    /// Trigger every frame
    Tick,
    /// Trigger when the Inkwater trial reward text should be updated
    InkwaterTrialTextRequest, // TODO rename Inkwater -> Marsh
    /// Trigger when the Hollow trial reward text should be updated
    HollowTrialTextRequest,
    /// Trigger when the Wellspring trial reward text should be updated
    WellspringTrialTextRequest,
    /// Trigger when the Woods trial reward text should be updated
    WoodsTrialTextRequest,
    /// Trigger when the Reach trial reward text should be updated
    ReachTrialTextRequest,
    /// Trigger when the Depths trial reward text should be updated
    DepthsTrialTextRequest,
    /// Trigger when the Luma trial reward text should be updated
    LumaTrialTextRequest, // TODO rename Luma -> Pools
    /// Trigger when the Wastes trial reward text should be updated
    WastesTrialTextRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum TriggerBinding<'source> {
    UberIdentifier(UberIdentifier<'source>),
    Identifier(Spanned<Identifier<'source>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(token = Token::Change)]
pub struct Change;

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(token = Token::Fun)]
pub struct Fun;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct FunctionDefinition<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub empty_args: (Symbol<'('>, Symbol<')'>),
    pub actions: Delimited<'{', Vec<Action<'source>>, '}'>,
}

// TODO I think I want a let-style syntax that compiles into all the set and get functions and then remove those to solve the confusion around store vs. set
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum Action<'source> {
    Condition(Spanned<If>, Box<ActionCondition<'source>>),
    Function(Box<FunctionCall<'source>>),
    Multi(Delimited<'{', Vec<Action<'source>>, '}'>),
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(token = Token::If)]
pub struct If;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ActionCondition<'source> {
    pub condition: Expression<'source>,
    pub action: Recoverable<Action<'source>, RecoverContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct FunctionCall<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub parameters: Delimited<'(', Punctuated<Expression<'source>, ','>, ')'>,
}

#[derive(Debug, Clone, PartialEq, Eq, Span)]
pub enum Expression<'source> {
    Value(ExpressionValue<'source>),
    Operation(Box<Operation<'source>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Span)]
pub struct Operation<'source> {
    pub left: Expression<'source>,
    pub operator: Spanned<Operator>,
    pub right: Expression<'source>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum ExpressionValue<'source> {
    Group(Delimited<'(', Once<Box<Expression<'source>>>, ')'>),
    Action(Action<'source>),
    Literal(Spanned<Literal<'source>>),
    Identifier(Spanned<Identifier<'source>>),
}

// Manual implementation to support operator precedence
impl<'source> Ast<'source, Tokenizer> for Expression<'source> {
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, Tokenizer>) -> ControlFlow<M::Error, Self> {
        fn precedence(operator: Operator) -> u8 {
            match operator {
                Operator::Arithmetic(ArithmeticOperator::Multiply | ArithmeticOperator::Divide) => {
                    3
                }
                Operator::Arithmetic(ArithmeticOperator::Add | ArithmeticOperator::Subtract) => 2,
                Operator::Comparator(_) => 1,
                Operator::Logic(_) => 0,
            }
        }

        fn resolve_precedence(
            mut sequence: SeparatedNonEmpty<ExpressionValue, Spanned<Operator>>,
        ) -> Expression {
            let current_operator_index = sequence
                .more
                .iter()
                .enumerate()
                // TODO this is important for correct a - b - c precedence, but it increases the memory slots required, maybe investigate why
                .rev()
                .min_by_key(|(_, (operator, _))| precedence(operator.data))
                .map(|(index, _)| index);

            match current_operator_index {
                None => Expression::Value(sequence.first),
                Some(index) => {
                    // We know index < len and split_off does not panic if index == len
                    let right = sequence.more.split_off(index + 1);
                    // We know len > 0 because we split off at index + 1
                    let (operator, first_right) = sequence.more.pop().unwrap();
                    let right_sequence = SeparatedNonEmpty {
                        first: first_right,
                        more: right,
                    };

                    Expression::Operation(Box::new(Operation {
                        left: resolve_precedence(sequence),
                        operator,
                        right: resolve_precedence(right_sequence),
                    }))
                }
            }
        }

        SeparatedNonEmpty::ast_impl::<M>(parser).map_continue(resolve_precedence)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ast)]
pub enum Operator {
    Arithmetic(ArithmeticOperator),
    Logic(LogicOperator),
    Comparator(Comparator),
}

/// Arithmetic Operations performed on numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr, Ast)]
#[repr(u8)]
pub enum ArithmeticOperator {
    /// `+`
    #[ast(token = Token::Add)]
    Add = 0,
    /// `-`
    #[ast(token = Token::Subtract)]
    Subtract = 1,
    /// `*`
    #[ast(token = Token::Multiply)]
    Multiply = 2,
    /// `/`
    #[ast(token = Token::Divide)]
    Divide = 3,
}

/// Logic Operations performed on booleans
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr, Ast)]
#[repr(u8)]
pub enum LogicOperator {
    /// `&&`
    #[ast(token = Token::And)]
    And = 0,
    /// `||`
    #[ast(token = Token::Or)]
    Or = 1,
}

/// Comparison Operations performed on numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr, Ast)]
#[repr(u8)]
pub enum Comparator {
    /// `==`
    #[ast(token = Token::Equal)]
    Equal = 0,
    /// `!=`
    #[ast(token = Token::NotEqual)]
    NotEqual = 1,
    /// `<`
    #[ast(token = Token::Less)]
    Less = 2,
    /// `<=`
    #[ast(token = Token::LessOrEqual)]
    LessOrEqual = 3,
    /// `>`
    #[ast(token = Token::Greater)]
    Greater = 4,
    /// `>=`
    #[ast(token = Token::GreaterOrEqual)]
    GreaterOrEqual = 5,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
pub enum Literal<'source> {
    UberIdentifier(UberIdentifier<'source>),
    Boolean(bool),
    Integer(i32),
    Float(OrderedFloat<f32>),
    String(&'source str),
    Constant(Constant),
}

// TODO EnumTryAs from strum fails to parse this enum, could otherwise simplify some compile impls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ast, EnumDiscriminants)]
#[strum_discriminants(derive(VariantArray))]
pub enum Constant {
    ClientEvent(ClientEvent),
    Skill(#[ast(with = "constant_ast::<Skill, M>")] Skill),
    Shard(#[ast(with = "constant_ast::<Shard, M>")] Shard),
    Teleporter(#[ast(with = "constant_ast::<Teleporter, M>")] Teleporter),
    WeaponUpgrade(#[ast(with = "constant_ast::<WeaponUpgrade, M>")] WeaponUpgrade),
    Equipment(#[ast(with = "constant_ast::<Equipment, M>")] Equipment),
    Zone(#[ast(with = "constant_ast::<Zone, M>")] Zone),
    OpherIcon(#[ast(with = "constant_ast::<OpherIcon, M>")] OpherIcon),
    LupoIcon(#[ast(with = "constant_ast::<LupoIcon, M>")] LupoIcon),
    GromIcon(#[ast(with = "constant_ast::<GromIcon, M>")] GromIcon),
    TuleyIcon(#[ast(with = "constant_ast::<TuleyIcon, M>")] TuleyIcon),
    MapIcon(#[ast(with = "constant_ast::<MapIcon, M>")] MapIcon),
    EquipSlot(#[ast(with = "constant_ast::<EquipSlot, M>")] EquipSlot),
    WheelItemPosition(#[ast(with = "constant_ast::<WheelItemPosition, M>")] WheelItemPosition),
    WheelBind(#[ast(with = "constant_ast::<WheelBind, M>")] WheelBind),
    Alignment(#[ast(with = "constant_ast::<Alignment, M>")] Alignment),
    HorizontalAnchor(#[ast(with = "constant_ast::<HorizontalAnchor, M>")] HorizontalAnchor),
    VerticalAnchor(#[ast(with = "constant_ast::<VerticalAnchor, M>")] VerticalAnchor),
    ScreenPosition(#[ast(with = "constant_ast::<ScreenPosition, M>")] ScreenPosition),
    CoordinateSystem(#[ast(with = "constant_ast::<CoordinateSystem, M>")] CoordinateSystem),
}

fn constant_ast<T, M: Mode>(parser: &mut Parser<Tokenizer>) -> ControlFlow<M::Error, T>
where
    T: FromStr<Err = String> + VariantArray + Display,
{
    let before = parser.position();

    let identifier = <Spanned<Identifier>>::ast_impl::<M>(parser)?;

    let flow = M::res(identifier.data.0.parse(), |_| {
        Error::all_failed(
            T::VARIANTS
                .iter()
                .map(|t| {
                    Error::new(
                        ErrorKind::ExpectedToken(t.to_string()),
                        identifier.span.clone(),
                    )
                })
                .collect(),
        )
    });

    if flow.is_break() {
        parser.jump(before);
    }

    flow
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum UberIdentifier<'source> {
    Numeric(UberIdentifierNumeric),
    Name(UberIdentifierName<'source>),
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct UberIdentifierNumeric {
    pub group: Spanned<i32>,
    pub separator: Symbol<'|'>,
    pub member: Recoverable<Spanned<i32>, RecoverPass>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct UberIdentifierName<'source> {
    pub group: Spanned<Identifier<'source>>,
    pub period: Symbol<'.'>,
    pub member: Recoverable<Spanned<Identifier<'source>>, RecoverPass>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum Command<'source> {
    // TODO have include be able to change the default config?
    Include(Spanned<Include>, CommandArgs<IncludeArgs<'source>>),
    BundleIcon(Spanned<BundleIcon>, CommandArgs<BundleIconArgs<'source>>),
    BuiltinIcon(Spanned<BuiltinIcon>, CommandArgs<BuiltinIconArgs<'source>>),
    AugmentFun(Spanned<AugmentFun>, CommandArgs<AugmentFunArgs<'source>>),
    Export(Spanned<Export>, CommandArgs<ExportArgs<'source>>),
    Spawn(Spanned<Spawn>, CommandArgs<SpawnArgs<'source>>),
    Tags(
        Spanned<Tags>,
        CommandArgsCollection<SeparatedNonEmpty<TagsArg<'source>, Symbol<','>>>,
    ),
    Config(Spanned<Config>, CommandArgs<ConfigArgs<'source>>),
    SetConfig(Spanned<SetConfig>, CommandArgs<SetConfigArgs<'source>>),
    State(Spanned<State>, CommandArgs<StateArgs<'source>>),
    Timer(Spanned<Timer>, CommandArgs<TimerArgs<'source>>),
    Let(Spanned<Let>, CommandArgs<LetArgs<'source>>),
    If(Spanned<If>, CommandIf<'source>),
    Repeat(Spanned<Repeat>, CommandRepeat<'source>),
    Add(Spanned<Add>, CommandArgs<AddArgs<'source>>),
    Remove(Spanned<Remove>, CommandArgs<RemoveArgs<'source>>),
    ItemData(Spanned<ItemData>, CommandArgs<ItemDataArgs<'source>>),
    ItemDataName(
        Spanned<ItemDataName>,
        CommandArgs<ItemDataNameArgs<'source>>,
    ),
    ItemDataPrice(
        Spanned<ItemDataPrice>,
        CommandArgs<ItemDataPriceArgs<'source>>,
    ),
    ItemDataDescription(
        Spanned<ItemDataDescription>,
        CommandArgs<ItemDataDescriptionArgs<'source>>,
    ),
    ItemDataIcon(
        Spanned<ItemDataIcon>,
        CommandArgs<ItemDataIconArgs<'source>>,
    ),
    ItemDataMapIcon(
        Spanned<ItemDataMapIcon>,
        CommandArgs<ItemDataMapIconArgs<'source>>,
    ),
    RemoveLocation(
        Spanned<RemoveLocation>,
        CommandArgs<RemoveLocationArgs<'source>>,
    ),
    SetLogicState(
        Spanned<SetLogicState>,
        CommandArgs<SetLogicStateArgs<'source>>,
    ),
    Preplace(Spanned<Preplace>, CommandArgs<PreplaceArgs<'source>>),
    ZoneOf(Spanned<ZoneOf>, CommandArgs<ZoneOfArgs<'source>>),
    ItemOn(Spanned<ItemOn>, CommandArgs<ItemOnArgs<'source>>),
    CountInZone(Spanned<CountInZone>, CommandArgs<CountInZoneArgs<'source>>),
    RandomInteger(
        Spanned<RandomInteger>,
        CommandArgs<RandomIntegerArgs<'source>>,
    ),
    RandomFloat(Spanned<RandomFloat>, CommandArgs<RandomFloatArgs<'source>>),
    RandomPool(Spanned<RandomPool>, CommandArgs<RandomPoolArgs<'source>>),
    RandomFromPool(
        Spanned<RandomFromPool>,
        CommandArgs<RandomFromPoolArgs<'source>>,
    ),
}

pub type CommandArgsCollection<Args> = Recoverable<Delimited<'(', Args, ')'>, RecoverContent>;
pub type CommandArgs<Args> = CommandArgsCollection<Once<Args>>;
pub type CommandArg<T> =
    Recoverable<(Symbol<','>, Recoverable<T, RecoverCommandArg>), RecoverCommandArg>;

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Include;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct IncludeArgs<'source> {
    pub path: Spanned<&'source str>,
    pub imports: Spanned<Option<(Symbol<','>, IncludeArgsImports<'source>)>>,
}

pub type IncludeArgsImports<'source> =
    Separated<Recoverable<Spanned<Identifier<'source>>, RecoverSkipExpression>, Symbol<','>>;

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct BundleIcon;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct BundleIconArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub path: CommandArg<Spanned<&'source str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct BuiltinIcon;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct BuiltinIconArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub path: CommandArg<Spanned<&'source str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct AugmentFun;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct AugmentFunArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub action: CommandArg<Action<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Export;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ExportArgs<'source>(pub Spanned<Identifier<'source>>);

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Spawn;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct SpawnArgs<'source> {
    pub x: Expression<'source>,
    pub y: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Tags;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct TagsArg<'source>(pub Expression<'source>);

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Config;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ConfigArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub description: CommandArg<Spanned<&'source str>>,
    pub ty: CommandArg<Spanned<ConfigType>>,
    pub default: CommandArg<Spanned<Literal<'source>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ast, Display, VariantArray)]
pub enum ConfigType {
    Boolean,
    Integer,
    Float,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct SetConfig;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct SetConfigArgs<'source> {
    pub snippet_name: Spanned<&'source str>,
    pub identifier: CommandArg<Spanned<Identifier<'source>>>,
    // TODO weird to have a string here, this ties into the whole preprocessing conundrum...
    pub value: CommandArg<Spanned<&'source str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct State;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct StateArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub ty: CommandArg<Spanned<UberStateType>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ast, Display, VariantArray)]
pub enum UberStateType {
    Boolean,
    Integer,
    Float,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Timer;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct TimerArgs<'source> {
    pub toggle_identifier: Spanned<Identifier<'source>>,
    pub timer_identifier: CommandArg<Spanned<Identifier<'source>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Let;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct LetArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub value: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CommandIf<'source> {
    pub condition: Expression<'source>,
    pub contents: Delimited<'{', Vec<Recoverable<Content<'source>, RecoverContent>>, '}'>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Repeat;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CommandRepeat<'source> {
    pub amount: Expression<'source>,
    pub contents: Delimited<'{', Vec<Recoverable<Content<'source>, RecoverContent>>, '}'>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Add;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct AddArgs<'source>(pub ChangeItemPoolArgs<'source>);

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Remove;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RemoveArgs<'source>(pub ChangeItemPoolArgs<'source>);

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ChangeItemPoolArgs<'source> {
    pub item: Action<'source>,
    pub amount: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ItemData;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataArgs<'source> {
    pub item: Action<'source>,
    pub name: CommandArg<Expression<'source>>,
    pub price: CommandArg<Expression<'source>>, // TODO why isn't the description after the name?
    pub description: CommandArg<Expression<'source>>,
    pub icon: CommandArg<Expression<'source>>,
    pub map_icon: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ItemDataName;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataNameArgs<'source> {
    pub item: Action<'source>,
    pub name: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ItemDataPrice;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataPriceArgs<'source> {
    pub item: Action<'source>,
    pub price: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ItemDataDescription;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataDescriptionArgs<'source> {
    pub item: Action<'source>,
    pub description: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ItemDataIcon;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataIconArgs<'source> {
    pub item: Action<'source>,
    pub icon: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ItemDataMapIcon;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataMapIconArgs<'source> {
    pub item: Action<'source>,
    pub map_icon: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct RemoveLocation;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RemoveLocationArgs<'source> {
    pub condition: Expression<'source>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct SetLogicState;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct SetLogicStateArgs<'source>(pub Spanned<&'source str>);

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Preplace;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct PreplaceArgs<'source> {
    pub item: Action<'source>,
    pub zone: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ZoneOf;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ZoneOfArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub item: CommandArg<Action<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct ItemOn;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemOnArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub trigger: CommandArg<Trigger<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct CountInZone;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CountInZoneArgs<'source> {
    pub zone_bindings: Delimited<
        '[',
        Punctuated<Delimited<'(', Once<CountInZoneBinding<'source>>, ')'>, ','>,
        ']',
    >,
    pub items: CommandArg<Delimited<'[', Punctuated<Action<'source>, ','>, ']'>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CountInZoneBinding<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub zone: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct RandomInteger;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomIntegerArgs<'source>(pub RandomNumberArgs<'source>);

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct RandomFloat;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomFloatArgs<'source>(pub RandomNumberArgs<'source>);

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomNumberArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub min: CommandArg<Expression<'source>>,
    pub max: CommandArg<Expression<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct RandomPool;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomPoolArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub ty: CommandArg<Spanned<Type>>,
    pub values: CommandArg<Delimited<'[', Punctuated<Expression<'source>, ','>, ']'>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct RandomFromPool;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomFromPoolArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub pool_identifier: CommandArg<Spanned<Identifier<'source>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum Annotation<'source> {
    Hidden(Spanned<Hidden>),
    Name(Spanned<Name>, CommandArgs<Spanned<&'source str>>),
    Category(Spanned<Category>, CommandArgs<Spanned<&'source str>>),
    Description(Spanned<Description>, CommandArgs<Spanned<&'source str>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Hidden;

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Name;

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Category;

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake_case")]
pub struct Description;
