use crate::logic_language::token::{Token, Tokenizer};
use ordered_float::OrderedFloat;
use std::ops::Range;
use wotw_seedgen_parse::{
    parse_ast, Ast, ErrorKind, ErrorMode, Identifier, ParseResult, Parser, Recover, Recoverable,
    Separated, SeparatedNonEmpty, Span, SpanEnd, SpanStart, Spanned, SpannedOption, Symbol,
};

#[inline]
pub fn parse_logic_ast<'source, V>(source: &'source str) -> ParseResult<V>
where
    V: Ast<'source, Tokenizer>,
{
    parse_ast(source, Tokenizer)
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct Areas<'source> {
    pub contents: Separated<Recoverable<Content<'source>, RecoverDedent>, Newline>,
}

impl<'source> Areas<'source> {
    #[inline]
    pub fn parse(source: &'source str) -> ParseResult<Self> {
        parse_logic_ast(source)
    }
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub enum Content<'source> {
    Requirement(
        Spanned<RequirementKeyword>,
        Recoverable<Macro<'source>, RecoverDedent>,
    ),
    Region(
        Spanned<RegionKeyword>,
        Recoverable<Region<'source>, RecoverDedent>,
    ),
    Anchor(
        Spanned<AnchorKeyword>,
        Recoverable<Anchor<'source>, RecoverDedent>,
    ),
}

pub struct RecoverDedent;

impl<'source> Recover<'source, Tokenizer> for RecoverDedent {
    fn recover(parser: &mut Parser<'source, Tokenizer>) {
        let mut depth = None;
        loop {
            match parser.current().0 {
                Token::Newline => {
                    if depth.is_none() {
                        break;
                    }
                }
                Token::Indent => {
                    *depth.get_or_insert(0) += 1;
                }
                Token::Dedent => {
                    let depth = depth.get_or_insert(0);
                    if *depth > 1 {
                        *depth -= 1;
                    } else {
                        parser.step();
                        break;
                    }
                }
                Token::Eof => break,
                _ => {}
            }
            parser.step();
        }
    }
}

pub struct RecoverNewline;

impl<'source> Recover<'source, Tokenizer> for RecoverNewline {
    fn recover(parser: &mut Parser<'source, Tokenizer>) {
        parser.jump(
            parser.find(|token, _| matches!(token, Token::Newline | Token::Dedent | Token::Indent)),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Requirement)]
pub struct RequirementKeyword;

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct Macro<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub requirements: RequirementGroup<'source>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Region)]
pub struct RegionKeyword;

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct Region<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub requirements: RequirementGroup<'source>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Anchor)]
pub struct AnchorKeyword;

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct Anchor<'source> {
    pub identifier: Spanned<LogicIdentifier<'source>>,
    pub position: Option<AnchorPosition>,
    pub content: Group<SeparatedNonEmpty<AnchorContent<'source>, Newline>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicIdentifier<'source>(pub &'source str);

impl<'source> Ast<'source, Tokenizer> for LogicIdentifier<'source> {
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, Tokenizer>) -> Option<Self> {
        let (token, span) = parser.current();

        match token {
            Token::LogicIdentifier | Token::Identifier => {
                let slice = parser.slice(span.clone());
                parser.step();

                Some(Self(slice))
            }
            _ => E::none(|| parser.error(ErrorKind::ExpectedToken("LogicIdentifier".to_string()))),
        }
    }
}

impl<'source> LogicIdentifier<'source> {
    pub fn region(&self) -> Option<&'source str> {
        self.0.split_once('.').map(|(region, _)| region)
    }
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct AnchorPosition {
    pub at: Spanned<At>,
    pub position: Recoverable<Position, RecoverPosition>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::At)]
pub struct At;

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct Position {
    pub x: Spanned<OrderedFloat<f32>>,
    pub comma: Symbol<','>,
    pub y: Spanned<OrderedFloat<f32>>,
}

pub struct RecoverPosition;
impl<'source> Recover<'source, Tokenizer> for RecoverPosition {
    fn recover(parser: &mut Parser<'source, Tokenizer>) {
        parser.jump(parser.find(|token, span| {
            matches!(token, Token::Newline | Token::Indent) || parser.slice(span.clone()) == ":"
        }));
    }
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub enum AnchorContent<'source> {
    Door(Spanned<DoorKeyword>, Door<'source>),
    NoSpawn(Spanned<NoSpawn>),
    TpRestriction(
        Spanned<TpRestrictionKeyword>,
        SpannedOption<RequirementLineOrGroup<'source>>,
    ),
    Refill(
        Spanned<RefillKeyword>,
        Recoverable<Refill<'source>, RecoverDedent>,
    ),
    Connection(
        Spanned<ConnectionKeyword>,
        Recoverable<Connection<'source>, RecoverDedent>,
    ),
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Door)]
pub struct DoorKeyword;
pub type Door<'source> = Group<SeparatedNonEmpty<DoorContent<'source>, Newline>>;

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub enum DoorContent<'source> {
    Id(Spanned<Id>, Recoverable<DoorId, RecoverNewline>),
    Target(
        Spanned<Target>,
        Recoverable<DoorTarget<'source>, RecoverNewline>,
    ),
    Enter(
        Spanned<Enter>,
        Recoverable<RequirementLineOrGroup<'source>, RecoverNewline>,
    ),
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Id)]
pub struct Id;

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct DoorId {
    pub colon: Spanned<Symbol<':'>>,
    pub id: Spanned<i32>,
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct DoorTarget<'source> {
    pub colon: Spanned<Symbol<':'>>,
    pub target: Spanned<LogicIdentifier<'source>>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Target)]
pub struct Target;

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Enter)]
pub struct Enter;

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::NoSpawn)]
pub struct NoSpawn;

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::TpRestriction)]
pub struct TpRestrictionKeyword;

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Refill)]
pub struct RefillKeyword;

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct Refill<'source> {
    pub value: Spanned<RefillValue>,
    pub requirements: Option<RequirementLineOrGroup<'source>>,
}

impl Span for Refill<'_> {
    fn span(&self) -> Range<usize> {
        self.span_start()..self.span_end()
    }
}

impl SpanStart for Refill<'_> {
    fn span_start(&self) -> usize {
        self.value.span_start()
    }
}

impl SpanEnd for Refill<'_> {
    fn span_end(&self) -> usize {
        match &self.requirements {
            None => self.value.span_end(),
            Some(requirements) => requirements.span_end(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub enum RefillValue {
    Full,
    Checkpoint,
    Health(RefillHealth),
    Energy(RefillEnergy),
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct RefillHealth {
    pub identifier: Health,
    pub amount: Option<Amount>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct Health;

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct RefillEnergy {
    pub identifier: Energy,
    pub amount: Option<Amount>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct Energy;

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct Amount {
    pub equals: Spanned<Symbol<'='>>,
    pub value: Recoverable<Spanned<usize>, RecoverPass>,
}

pub struct RecoverPass;
impl<'source> Recover<'source, Tokenizer> for RecoverPass {
    fn recover(_parser: &mut Parser<'source, Tokenizer>) {}
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub enum ConnectionKeyword {
    #[ast(token = Token::State)]
    State,
    // TODO remove?
    #[ast(token = Token::Quest)]
    Quest,
    #[ast(token = Token::Pickup)]
    Pickup,
    #[ast(token = Token::Connection)]
    Anchor,
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct Connection<'source> {
    pub identifier: Spanned<LogicIdentifier<'source>>,
    pub requirements: RequirementLineOrGroup<'source>,
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct RequirementLineOrGroup<'source> {
    pub colon: Spanned<Symbol<':'>>,
    pub requirement: Recoverable<InlineRequirementOrGroup<'source>, RecoverDedent>,
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub enum InlineRequirementOrGroup<'source> {
    Inline(Spanned<Free>),
    Group(GroupContent<SeparatedNonEmpty<RequirementLine<'source>, Newline>>),
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(case = "lowercase")]
pub struct Free;
pub type RequirementGroup<'source> = Group<SeparatedNonEmpty<RequirementLine<'source>, Newline>>;

#[derive(Debug, Clone, PartialEq)]
pub struct RequirementLine<'source> {
    pub ands: Vec<(Requirement<'source>, And)>,
    pub ors: SeparatedNonEmpty<Requirement<'source>, Or>,
    pub group: Option<Box<RequirementGroup<'source>>>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub enum And {
    Comma(Symbol<','>),
    Colon(Symbol<':'>),
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Or)]
pub struct Or;
impl<'source> Ast<'source, Tokenizer> for RequirementLine<'source> {
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, Tokenizer>) -> Option<Self> {
        let before = parser.position();

        let result = (|| {
            let mut ands = vec![];

            loop {
                let last = Requirement::ast_impl::<E>(parser)?;

                if parser.current().0 == Token::Or {
                    let mut more = Vec::with_capacity(1);

                    loop {
                        parser.step();
                        more.push((Or, Requirement::ast_impl::<E>(parser)?));

                        if parser.current().0 != Token::Or {
                            let ors = SeparatedNonEmpty { first: last, more };

                            let group = (parser.current_slice() == ":")
                                .then(|| RequirementGroup::ast_no_errors(parser).unwrap())
                                .map(Box::new);

                            return Some(RequirementLine { ands, ors, group });
                        }
                    }
                } else if parser.current_slice() == "," {
                    parser.step();
                    ands.push((last, And::Comma(Symbol)));
                } else if let Some(symbol) = <Spanned<Symbol<':'>>>::ast_no_errors(parser) {
                    if matches!(
                        parser.current().0,
                        Token::Newline | Token::Indent | Token::Dedent
                    ) {
                        let group = Some(Box::new(RequirementGroup {
                            colon: symbol,
                            content: Recoverable::ast_impl::<E>(parser)?,
                        }));

                        let ors = SeparatedNonEmpty {
                            first: last,
                            more: vec![],
                        };

                        return Some(RequirementLine { ands, ors, group });
                    } else {
                        ands.push((last, And::Colon(symbol.data)));
                    }
                } else if matches!(parser.current().0, Token::Newline | Token::Dedent) {
                    return Some(RequirementLine {
                        ands,
                        ors: SeparatedNonEmpty {
                            first: last,
                            more: vec![],
                        },
                        group: None,
                    });
                } else {
                    return E::none(|| {
                        parser.error(ErrorKind::ExpectedToken(
                            "',' or \"OR\" or ':' or Newline".to_string(),
                        ))
                    });
                }
            }
        })();

        if result.is_none() {
            parser.jump(before);
        }

        result
    }
}

impl Span for RequirementLine<'_> {
    fn span(&self) -> Range<usize> {
        self.span_start()..self.span_end()
    }
}

impl SpanStart for RequirementLine<'_> {
    fn span_start(&self) -> usize {
        match self.ands.first() {
            None => self.ors.span_start(),
            Some((first, _)) => first.span_start(),
        }
    }
}

impl SpanEnd for RequirementLine<'_> {
    fn span_end(&self) -> usize {
        match &self.group {
            None => self.ors.span_end(),
            Some(group) => group.span_end(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub enum Requirement<'source> {
    Combat(CombatRequirement<'source>),
    Plain(PlainRequirement<'source>),
    State(Spanned<LogicIdentifier<'source>>),
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct CombatRequirement<'source> {
    pub keyword: Spanned<Combat>,
    pub equals: Symbol<'='>,
    pub enemies: Recoverable<SeparatedNonEmpty<Enemy<'source>, Symbol<'+'>>, RecoverRequirement>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct PlainRequirement<'source> {
    pub identifier: Spanned<Identifier<'source>>,
    pub amount: Option<Amount>,
}

impl Span for PlainRequirement<'_> {
    fn span(&self) -> Range<usize> {
        self.span_start()..self.span_end()
    }
}

impl SpanStart for PlainRequirement<'_> {
    fn span_start(&self) -> usize {
        self.identifier.span_start()
    }
}

impl SpanEnd for PlainRequirement<'_> {
    fn span_end(&self) -> usize {
        match &self.amount {
            None => self.identifier.span_end(),
            Some(amount) => amount.span_end(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct Combat;

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct Enemy<'source> {
    pub amount: Option<EnemyAmount>,
    pub identifier: Spanned<Identifier<'source>>,
}

impl Span for Enemy<'_> {
    fn span(&self) -> Range<usize> {
        self.span_start()..self.span_end()
    }
}

impl SpanStart for Enemy<'_> {
    fn span_start(&self) -> usize {
        match &self.amount {
            None => self.identifier.span_start(),
            Some(amount) => amount.span_start(),
        }
    }
}

impl SpanEnd for Enemy<'_> {
    fn span_end(&self) -> usize {
        self.identifier.span_end()
    }
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct EnemyAmount {
    pub value: Spanned<u8>,
    pub times: Spanned<Debug<Symbol<'x'>>>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
pub struct Debug<T>(T);
pub struct RecoverRequirement;
impl<'source> Recover<'source, Tokenizer> for RecoverRequirement {
    fn recover(parser: &mut Parser<'source, Tokenizer>) {
        parser.jump(parser.find(|token, span| {
            matches!(
                token,
                Token::Or | Token::Newline | Token::Indent | Token::Dedent
            ) || matches!(parser.slice(span.clone()), "," | ":")
        }));
    }
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct Group<T> {
    pub colon: Spanned<Symbol<':'>>,
    pub content: Recoverable<GroupContent<T>, RecoverDedent>,
}

#[derive(Debug, Clone, PartialEq, Ast, Span)]
pub struct GroupContent<T> {
    pub indent: Spanned<Indent>,
    pub content: T,
    pub dedent: Spanned<Dedent>,
}

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Newline)]
pub struct Newline;

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Indent)]
pub struct Indent;

#[derive(Debug, Clone, PartialEq, Ast)]
#[ast(token = Token::Dedent)]
pub struct Dedent;
