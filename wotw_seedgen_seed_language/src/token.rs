use logos::Logos;
use wotw_seedgen_parse::{
    LogosTokenizer, ParseBoolToken, ParseFloatToken, ParseIdentToken, ParseIntToken,
    ParseStringToken, TokenDisplay,
};

#[derive(Logos, Debug, Clone, Copy, Hash, PartialEq, Eq, TokenDisplay)]
#[logos(skip r"\s+|//[^\n]*")]
pub enum Token {
    #[token("on")]
    On,
    #[token("change")]
    Change,
    #[token("fun")]
    Fun,
    #[token("if")]
    If,
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Boolean(bool),
    #[regex(r"[_a-zA-Z]\w*")]
    Identifier,
    #[regex(r"-?\d+\.\d*")]
    Float,
    #[regex(r"-?\d+")]
    Integer,
    #[regex(r#""[^"]*""#)]
    String,
    #[token("+")]
    Add,
    #[token("-")]
    Subtract,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<=")]
    LessOrEqual,
    #[token("<")]
    Less,
    #[token(">=")]
    GreaterOrEqual,
    #[token(">")]
    Greater,
    #[token("::")]
    Variant,
    #[regex(r".", priority = 0)]
    Symbol,
    Eof,
}

impl ParseBoolToken for Token {
    fn bool(&self) -> Option<bool> {
        match self {
            Token::Boolean(value) => Some(*value),
            _ => None,
        }
    }
}

impl ParseIntToken for Token {
    fn is_int(&self) -> bool {
        matches!(self, Token::Integer)
    }
}

impl ParseFloatToken for Token {
    fn is_float(&self) -> bool {
        matches!(self, Token::Float | Token::Integer)
    }
}

impl ParseStringToken for Token {
    fn is_string(&self) -> bool {
        matches!(self, Token::String)
    }
}

impl ParseIdentToken for Token {
    fn is_ident(&self) -> bool {
        matches!(self, Token::Identifier)
    }
}

pub type Tokenizer = LogosTokenizer<Token>;
pub const TOKENIZER: Tokenizer = Tokenizer::new(Token::Symbol, Token::Eof);
