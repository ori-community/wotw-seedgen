use std::fmt;

use decorum::R32;
use rustc_hash::FxHashMap;
use wotw_seedgen_derive::VVariant;

use crate::header::{parser, CodeDisplay, VResolve, VString};
use crate::languages::TokenKind;
use crate::VItem;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
pub struct Message {
    #[VType]
    pub message: String,
    pub frames: Option<u32>,
    pub pos: Option<R32>,
    pub mute: bool,
    pub instant: bool,
    pub quiet: bool,
    pub noclear: bool,
}

macro_rules! write_part {
    ($first:ident, $dst:expr, $($arg:tt)*) => {
        {
            #[allow(unused_assignments)]
            if $first {
                $first = false;
            } else {
                write!($dst, "|")?;
            }
            write!($dst, $($arg)*)
        }
    };
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

    pub fn code(&self) -> CodeDisplay<Message> {
        CodeDisplay::new(self, |s, f| {
            let mut first = true;

            if !s.message.is_empty() {
                write_part!(first, f, "{}", s.message)?;
            }
            if let Some(frames) = s.frames {
                write_part!(first, f, "f={frames}")?;
            }
            if let Some(pos) = s.pos {
                write_part!(first, f, "p={pos}")?;
            }
            if s.mute {
                write_part!(first, f, "mute")?;
            }
            if s.instant {
                write_part!(first, f, "instant")?;
            }
            if s.quiet {
                write_part!(first, f, "quiet")?;
            }
            if s.noclear {
                write_part!(first, f, "noclear")?;
            }
            Ok(())
        })
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
        let mut message = self.message.clone();
        replace_item_syntax(&mut message);
        write!(f, "{message}")
    }
}
impl fmt::Display for VMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut message = self.message.0.clone();
        replace_item_syntax(&mut message);
        write!(f, "{message}")
    }
}

fn replace_item_syntax(message: &mut String) {
    let mut parser = parser::new(message);
    let mut replacements = vec![];
    loop {
        parser.skip_while(|kind| kind != TokenKind::Dollar);
        let token = parser.next_token();
        if token.kind == TokenKind::Eof {
            break;
        }
        let start = token.range.start;
        let token = parser.next_token();
        if token.kind != TokenKind::OpenBracket {
            continue;
        }
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

    for (range, replace_with) in replacements {
        message.replace_range(range, &replace_with);
    }
}
