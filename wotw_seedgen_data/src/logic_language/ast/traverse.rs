use wotw_seedgen_parse::{
    Identifier, Recoverable, Separated, SeparatedNonEmpty, Spanned, SpannedOption,
};

use crate::logic_language::ast::{
    Anchor, AnchorContent, And, Paths, Connection, ConnectionKeyword, Content, EntranceContent,
    EntranceTarget, Group, GroupContent, InlineRequirementOrGroup, LogicIdentifier, Macro,
    PlainRequirement, Refill, Region, Requirement, RequirementLine, RequirementLineOrGroup,
};

pub trait Traverse<'ast, 'source, H: Handler<'ast, 'source>> {
    fn traverse(&'ast self, handler: &mut H);
}

impl<'ast, 'source, H, T> Traverse<'ast, 'source, H> for Box<T>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        (**self).traverse(handler);
    }
}

impl<'ast, 'source, H, T> Traverse<'ast, 'source, H> for Option<T>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        if let Some(t) = self {
            t.traverse(handler);
        }
    }
}

impl<'ast, 'source, H, T> Traverse<'ast, 'source, H> for SpannedOption<T>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        if let SpannedOption::Some(t) = self {
            t.traverse(handler);
        }
    }
}

impl<'ast, 'source, H, T1, T2> Traverse<'ast, 'source, H> for (T1, T2)
where
    H: Handler<'ast, 'source>,
    T1: Traverse<'ast, 'source, H>,
    T2: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        self.0.traverse(handler);
        self.1.traverse(handler);
    }
}

impl<'ast, 'source, H, T> Traverse<'ast, 'source, H> for Vec<T>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        for t in self {
            t.traverse(handler);
        }
    }
}

impl<'ast, 'source, H, T, Separator> Traverse<'ast, 'source, H> for Separated<T, Separator>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        for t in self {
            t.traverse(handler);
        }
    }
}

impl<'ast, 'source, H, T, Separator> Traverse<'ast, 'source, H> for SeparatedNonEmpty<T, Separator>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        for t in self {
            t.traverse(handler);
        }
    }
}

impl<'ast, 'source, H, T, R> Traverse<'ast, 'source, H> for Recoverable<T, R>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        self.value.traverse(handler);
    }
}

impl<'ast, 'source, H, T> Traverse<'ast, 'source, H> for Group<T>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        self.content.traverse(handler);
    }
}

impl<'ast, 'source, H, T> Traverse<'ast, 'source, H> for GroupContent<T>
where
    H: Handler<'ast, 'source>,
    T: Traverse<'ast, 'source, H>,
{
    fn traverse(&'ast self, handler: &mut H) {
        self.content.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H>
    for RequirementLineOrGroup<'source>
{
    fn traverse(&'ast self, handler: &mut H) {
        self.requirement.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H>
    for InlineRequirementOrGroup<'source>
{
    fn traverse(&'ast self, handler: &mut H) {
        match self {
            Self::Inline(_) => {}
            Self::Group(group) => group.traverse(handler),
        }
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H>
    for RequirementLine<'source>
{
    fn traverse(&'ast self, handler: &mut H) {
        self.ands.traverse(handler);
        self.ors.traverse(handler);
        self.group.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Requirement<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        match self {
            Self::Combat(_) => {}
            Self::Plain(plain) => handler.plain_requirement(plain),
            Self::State(identifier) => handler.state_use(identifier),
        }
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for And {
    fn traverse(&'ast self, _handler: &mut H) {}
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Paths<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        self.contents.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Content<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        match self {
            Self::Requirement(_, requirement) => requirement.traverse(handler),
            Self::Region(_, region) => region.traverse(handler),
            Self::Anchor(_, anchor) => anchor.traverse(handler),
        }
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Macro<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        handler.macro_def(&self.identifier);
        self.requirements.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Region<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        self.requirements.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Anchor<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        handler.anchor_def(&self.identifier);
        self.content.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H>
    for AnchorContent<'source>
{
    fn traverse(&'ast self, handler: &mut H) {
        match self {
            Self::Entrance(_, entrance) => entrance.traverse(handler),
            Self::NoSpawn(_) => {}
            Self::TpRestriction(_, tprestriction) => tprestriction.traverse(handler),
            Self::Refill(_, refill) => refill.traverse(handler),
            Self::Connection(keyword, connection) => {
                if let SpannedOption::Some(connection) = &connection.value {
                    match keyword.data {
                        ConnectionKeyword::State
                        | ConnectionKeyword::Quest
                        | ConnectionKeyword::Pickup => handler.state_def(&connection.identifier),
                        ConnectionKeyword::Anchor => handler.anchor_use(&connection.identifier),
                    }

                    connection.traverse(handler);
                }
            }
        }
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H>
    for EntranceContent<'source>
{
    fn traverse(&'ast self, handler: &mut H) {
        match self {
            Self::Id(_, _) => {}
            Self::Target(_, target) => target.traverse(handler),
            Self::Enter(_, enter) => enter.traverse(handler),
        }
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H>
    for EntranceTarget<'source>
{
    fn traverse(&'ast self, handler: &mut H) {
        handler.anchor_use(&self.target);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Refill<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        self.requirements.traverse(handler);
    }
}

impl<'ast, 'source, H: Handler<'ast, 'source>> Traverse<'ast, 'source, H> for Connection<'source> {
    fn traverse(&'ast self, handler: &mut H) {
        self.requirements.traverse(handler);
    }
}

pub trait Handler<'ast, 'source> {
    fn macro_def(&mut self, identifier: &'ast Spanned<Identifier<'source>>) {
        let _ = identifier;
    }

    fn anchor_def(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        let _ = identifier;
    }

    fn anchor_use(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        let _ = identifier;
    }

    fn state_def(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        let _ = identifier;
    }

    fn state_use(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        let _ = identifier;
    }

    fn plain_requirement(&mut self, requirement: &'ast PlainRequirement<'source>) {
        let _ = requirement;
    }
}
