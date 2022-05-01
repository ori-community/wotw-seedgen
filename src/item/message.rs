use std::fmt;

use decorum::R32;
use rustc_hash::FxHashMap;
use seedgen_derive::VVariant;

use crate::VItem;
use crate::header::{VResolve, VString, parser::Parser, tokenizer::TokenKind};

#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub struct Message {
    #[VType] pub message: String,
    pub frames: Option<u32>,
    pub pos: Option<R32>,
    pub mute: bool,
    pub instant: bool,
    pub quiet: bool,
    pub noclear: bool,
}

impl Message {
    pub(crate) fn new(message: String) -> Message {
        Message {
            message,
            frames: None,
            pos: None,
            mute: false,
            instant: false,
            quiet: false,
            noclear: false,
        }
    }

    pub fn code(&self) -> String {
        let mut code = self.message.clone();
        if let Some(frames) = self.frames {
            code.push_str(&format!("f={frames}"));
        }
        if let Some(pos) = self.pos {
            code.push_str(&format!("p={pos}"));
        }
        if self.mute { code.push_str("|mute") }
        if self.instant { code.push_str("|instant") }
        if self.quiet { code.push_str("|quiet") }
        if self.noclear { code.push_str("|noclear") }
        code
    }
}
impl VMessage {
    pub(crate) fn new(message: VString) -> VMessage {
        VMessage {
            message,
            frames: None,
            pos: None,
            mute: false,
            instant: false,
            quiet: false,
            noclear: false,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parser = Parser::new(&self.message);
        let mut replacements = vec![];
        loop {
            parser.skip_while(|kind| kind != TokenKind::Dollar);
            let token = parser.next_token();
            if token.kind == TokenKind::Eof { break }
            let start = token.range.start;
            let token = parser.next_token();
            if token.kind != TokenKind::OpenBracket { continue }
            if let Ok(item) = VItem::parse(&mut parser) {
                if let Ok(item) = item.resolve(&FxHashMap::default()) {
                    let token = parser.next_token();
                    if token.kind == TokenKind::CloseBracket {
                        let end = token.range.end;
                        replacements.push((start..end, item.to_string()));
                    }
                }
            }
        }

        let mut message = self.message.clone();
        for (range, replace_with) in replacements {
            message.replace_range(range, &replace_with);
        }

        write!(f, "{message}")
    }
}
