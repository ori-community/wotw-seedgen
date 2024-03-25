use std::ops::Range;

// TODO documentation

pub trait Tokenize {
    type Token;

    fn tokenize(self, source: &str) -> TokenizeOutput<Self::Token>;
}

pub struct TokenizeOutput<Token> {
    pub tokens: Vec<(Token, Range<usize>)>,
    pub eof_token: Token,
}

#[cfg(feature = "logos")]
pub use logos_tokenizer::*;
#[cfg(feature = "logos")]
mod logos_tokenizer {
    use super::*;
    use logos::Logos;

    #[derive(Debug, Clone, Copy)]
    pub struct LogosTokenizer<Token> {
        error_token: Token,
        eof_token: Token,
    }
    impl<Token> LogosTokenizer<Token> {
        pub const fn new(error_token: Token, eof_token: Token) -> Self {
            Self {
                error_token,
                eof_token,
            }
        }
    }
    impl<Token> Tokenize for LogosTokenizer<Token>
    where
        for<'source> Token: Logos<'source, Source = str> + Clone,
        for<'source> <Token as Logos<'source>>::Extras: Default,
    {
        type Token = Token;

        fn tokenize(self, source: &str) -> TokenizeOutput<Self::Token> {
            TokenizeOutput {
                tokens: Token::lexer(source)
                    .spanned()
                    .map(|(token, span)| (token.unwrap_or_else(|_| self.error_token.clone()), span))
                    .collect(),
                eof_token: self.eof_token,
            }
        }
    }
}
