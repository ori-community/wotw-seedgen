use crate::{
    token::{Token, Tokenizer, TOKENIZER},
    types::Type,
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use strum::Display;
use wotw_seedgen_parse::{parse_ast, Separated};

pub use wotw_seedgen_parse::{
    Ast, Identifier, NoTrailingInput, Once, Parser, Recover, Recoverable, Result,
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
    Binding(Spanned<Change>, TriggerBinding<'source>),
    Condition(Expression<'source>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ast, Display, Serialize, Deserialize)]
#[ast(case = "snake")]
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
    /// Trigger on Teleport
    Teleport,
    /// Trigger on the Show Progress keybind
    ProgressMessage,
    /// Trigger every frame
    Tick,
    /// Trigger when the Inkwater trial reward text should be updated
    InkwaterTrialTextRequest,
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
    LumaTrialTextRequest,
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
    fn ast(parser: &mut Parser<'source, Tokenizer>) -> Result<Self> {
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
        SeparatedNonEmpty::ast(parser).map(resolve_precedence)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ast)]
pub enum Operator {
    Arithmetic(ArithmeticOperator),
    Logic(LogicOperator),
    Comparator(Comparator),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ast)]
pub enum ArithmeticOperator {
    #[ast(token = Token::Add)]
    Add,
    #[ast(token = Token::Subtract)]
    Subtract,
    #[ast(token = Token::Multiply)]
    Multiply,
    #[ast(token = Token::Divide)]
    Divide,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ast)]
pub enum LogicOperator {
    #[ast(token = Token::And)]
    And,
    #[ast(token = Token::Or)]
    Or,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ast)]
pub enum Comparator {
    #[ast(token = Token::Equal)]
    Equal,
    #[ast(token = Token::NotEqual)]
    NotEqual,
    #[ast(token = Token::LessOrEqual)]
    LessOrEqual,
    #[ast(token = Token::Less)]
    Less,
    #[ast(token = Token::GreaterOrEqual)]
    GreaterOrEqual,
    #[ast(token = Token::Greater)]
    Greater,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast)]
pub enum Literal<'source> {
    UberIdentifier(UberIdentifier<'source>),
    Boolean(bool),
    Integer(i32),
    Float(OrderedFloat<f32>),
    String(&'source str),
    Constant(Constant<'source>),
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
    pub member: Spanned<i32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct UberIdentifierName<'source> {
    pub group: Spanned<Identifier<'source>>,
    pub period: Symbol<'.'>,
    pub member: Spanned<Identifier<'source>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct Constant<'source> {
    pub kind: Spanned<Identifier<'source>>,
    pub separator: Variant,
    pub variant: Spanned<Identifier<'source>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(token = Token::Variant)]
pub struct Variant;

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum Command<'source> {
    // TODO have include be able to change the default config?
    Include(Spanned<Include>, CommandArgs<IncludeArgs<'source>>),
    BundleIcon(Spanned<BundleIcon>, CommandArgs<BundleIconArgs<'source>>),
    BuiltinIcon(Spanned<BuiltinIcon>, CommandArgs<BuiltinIconArgs<'source>>),
    Event(Spanned<EventIdent>, CommandArgs<EventArgs<'source>>),
    OnEvent(Spanned<OnEvent>, CommandArgs<OnEventArgs<'source>>),
    Export(Spanned<Export>, CommandArgs<ExportArgs<'source>>),
    Spawn(Spanned<Spawn>, CommandArgs<SpawnArgs<'source>>),
    Tags(
        Spanned<Tags>,
        CommandArgsCollection<SeparatedNonEmpty<TagsArg<'source>, Symbol<','>>>,
    ),
    Config(Spanned<Config>, CommandArgs<ConfigArgs<'source>>),
    State(Spanned<State>, CommandArgs<StateArgs<'source>>),
    Timer(Spanned<Timer>, CommandArgs<TimerArgs<'source>>),
    Let(Spanned<Let>, CommandArgs<LetArgs<'source>>),
    If(Spanned<If>, CommandIf<'source>),
    Repeat(Spanned<Repeat>, CommandRepeat<'source>),
    Add(Spanned<Add>, CommandArgs<AddArgs<'source>>),
    Remove(Spanned<Remove>, CommandArgs<AddArgs<'source>>),
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
    // TODO ItemDataMapIcon
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
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Include;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct IncludeArgs<'source> {
    pub path: Spanned<&'source str>,
    pub imports: Spanned<Option<(Symbol<','>, IncludeArgsImports<'source>)>>,
}
pub type IncludeArgsImports<'source> = Separated<Spanned<Identifier<'source>>, Symbol<','>>;
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct BundleIcon;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct BundleIconArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub path: Spanned<&'source str>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct BuiltinIcon;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct BuiltinIconArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub path: Spanned<&'source str>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(rename = "event")]
pub struct EventIdent;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct EventArgs<'source>(pub Spanned<Identifier<'source>>);
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct OnEvent;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct OnEventArgs<'source> {
    pub snippet_name: Spanned<&'source str>,
    pub comma: Symbol<','>,
    pub identifier: Spanned<Identifier<'source>>,
    pub comma_2: Symbol<','>,
    pub action: Action<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Export;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ExportArgs<'source>(pub Spanned<Identifier<'source>>);
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Spawn;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct SpawnArgs<'source> {
    pub x: Expression<'source>,
    pub comma: Symbol<','>,
    pub y: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Tags;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct TagsArg<'source>(pub Expression<'source>);
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Config;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ConfigArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub description: Expression<'source>,
    pub comma_2: Symbol<','>,
    pub ty: Spanned<UberStateType>,
    pub comma_3: Symbol<','>,
    pub default: Expression<'source>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ast, Display)]
pub enum UberStateType {
    Boolean,
    Integer,
    Float,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct State;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct StateArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub ty: Spanned<UberStateType>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Timer;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct TimerArgs<'source> {
    pub toggle_identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub timer_identifier: Spanned<Identifier<'source>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Let;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct LetArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub value: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CommandIf<'source> {
    pub condition: Expression<'source>,
    pub contents: Delimited<'{', Vec<Recoverable<Content<'source>, RecoverContent>>, '}'>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Repeat;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CommandRepeat<'source> {
    pub amount: Expression<'source>,
    pub contents: Delimited<'{', Vec<Recoverable<Content<'source>, RecoverContent>>, '}'>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Add;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct AddArgs<'source>(pub ChangeItemPoolArgs<'source>);
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Remove;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RemoveArgs<'source>(pub ChangeItemPoolArgs<'source>);
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ChangeItemPoolArgs<'source> {
    pub item: Action<'source>,
    pub comma: Symbol<','>,
    pub amount: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct ItemData;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataArgs<'source> {
    pub item: Action<'source>,
    pub comma: Symbol<','>,
    pub name: Expression<'source>,
    pub comma_2: Symbol<','>,
    pub price: Expression<'source>, // TODO why isn't the description after the name?
    pub comma_3: Symbol<','>,
    pub description: Expression<'source>,
    pub comma_4: Symbol<','>,
    pub icon: Expression<'source>,
    pub comma_5: Symbol<','>,
    pub map_icon: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct ItemDataName;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataNameArgs<'source> {
    pub item: Action<'source>,
    pub comma: Symbol<','>,
    pub name: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct ItemDataPrice;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataPriceArgs<'source> {
    pub item: Action<'source>,
    pub comma: Symbol<','>,
    pub price: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct ItemDataDescription;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataDescriptionArgs<'source> {
    pub item: Action<'source>,
    pub comma: Symbol<','>,
    pub description: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct ItemDataIcon;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemDataIconArgs<'source> {
    pub item: Action<'source>,
    pub comma: Symbol<','>,
    pub icon: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct SetLogicState;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct SetLogicStateArgs<'source>(pub Spanned<&'source str>);
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Preplace;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct PreplaceArgs<'source> {
    pub item: Action<'source>,
    pub comma: Symbol<','>,
    pub zone: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct ZoneOf;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ZoneOfArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub item: Action<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct ItemOn;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct ItemOnArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub trigger: Trigger<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct CountInZone;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CountInZoneArgs<'source> {
    pub zone_bindings: Delimited<
        '[',
        Punctuated<Delimited<'(', Once<CountInZoneBinding<'source>>, ')'>, ','>,
        ']',
    >,
    pub comma: Symbol<','>,
    pub items: Delimited<'[', Punctuated<Action<'source>, ','>, ']'>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct CountInZoneBinding<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub zone: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct RandomInteger;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomIntegerArgs<'source>(pub RandomNumberArgs<'source>);
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct RandomFloat;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomFloatArgs<'source>(pub RandomNumberArgs<'source>);
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomNumberArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub min: Expression<'source>,
    pub comma_2: Symbol<','>,
    pub max: Expression<'source>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct RandomPool;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomPoolArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub ty: Spanned<Type>,
    pub comma_2: Symbol<','>,
    pub values: Delimited<'[', Punctuated<Expression<'source>, ','>, ']'>,
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct RandomFromPool;
#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub struct RandomFromPoolArgs<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub comma: Symbol<','>,
    pub pool_identifier: Spanned<Identifier<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ast, Span)]
pub enum Annotation<'source> {
    Hidden(Spanned<Hidden>),
    Name(Spanned<Name>, CommandArgs<Spanned<&'source str>>),
    Category(Spanned<Category>, CommandArgs<Spanned<&'source str>>),
    Description(Spanned<Description>, CommandArgs<Spanned<&'source str>>),
}
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Hidden;
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Name;
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Category;
#[derive(Debug, Clone, PartialEq, Eq, Ast)]
#[ast(case = "snake")]
pub struct Description;
