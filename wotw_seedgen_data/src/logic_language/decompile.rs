use std::fmt::{self, Display};

use crate::{
    logic_language::output::{Anchor, Connection, Entrance, Graph, Node, Refill, Requirement},
    Position,
};

impl Graph {
    pub fn decompile(&self) -> impl Display + use<'_> {
        GraphDecompiler::new(self)
    }
}

struct GraphDecompiler<'g> {
    graph: &'g Graph,
}

impl<'g> GraphDecompiler<'g> {
    fn new(graph: &'g Graph) -> Self {
        Self { graph }
    }
}

impl Display for GraphDecompiler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for anchor in self.graph.nodes.iter().filter_map(Node::try_as_anchor_ref) {
            AnchorDecompiler::new(self.graph, anchor).fmt(f)?;
        }

        Ok(())
    }
}

struct AnchorDecompiler<'g> {
    graph: &'g Graph,
    anchor: &'g Anchor,
}

impl<'g> AnchorDecompiler<'g> {
    fn new(graph: &'g Graph, anchor: &'g Anchor) -> Self {
        Self { graph, anchor }
    }
}

impl Display for AnchorDecompiler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Anchor {
            identifier,
            position,
            entrance,
            can_spawn,
            teleport_restriction,
            refills,
            connections,
        } = self.anchor;

        write!(f, "anchor {identifier}")?;

        if let Some(position) = position {
            PositionDecompiler::new(position).fmt(f)?;
        }

        f.write_str(":\n")?;

        if let Some(entrance) = entrance {
            EntranceDecompiler::new(self.graph, entrance).fmt(f)?;

            f.write_str("\n")?;
        }

        if !can_spawn {
            f.write_str("  nospawn\n\n")?;
        }

        if !matches!(teleport_restriction, Requirement::Free) {
            write!(
                f,
                "  tprestriction:{}\n\n",
                RequirementDecompiler::new(self.graph, teleport_restriction, 4, true)
            )?;
        }

        for refill in refills {
            RefillDecompiler::new(self.graph, refill).fmt(f)?;
        }

        if !refills.is_empty() {
            f.write_str("\n")?;
        }

        for connection in connections {
            if connection.implicitly_generated {
                continue;
            }

            ConnectionDecompiler::new(self.graph, connection).fmt(f)?;

            f.write_str("\n")?;
        }

        f.write_str("\n")
    }
}

struct PositionDecompiler<'g> {
    position: &'g Position,
}

impl<'g> PositionDecompiler<'g> {
    fn new(position: &'g Position) -> Self {
        Self { position }
    }
}

impl Display for PositionDecompiler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Position { x, y } = self.position;

        write!(f, " at {x}, {y}")
    }
}

struct EntranceDecompiler<'g> {
    graph: &'g Graph,
    entrance: &'g Entrance,
}

impl<'g> EntranceDecompiler<'g> {
    fn new(graph: &'g Graph, entrance: &'g Entrance) -> Self {
        Self { graph, entrance }
    }
}

impl Display for EntranceDecompiler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Entrance {
            id,
            target,
            requirement,
        } = self.entrance;

        write!(
            f,
            concat!(
                "  entrance:\n",
                "    id: {}\n",
                "    target: {}\n",
                "    enter:{}\n",
            ),
            id,
            target,
            RequirementDecompiler::new(self.graph, requirement, 6, true),
        )
    }
}

struct RefillDecompiler<'g> {
    graph: &'g Graph,
    refill: &'g Refill,
}

impl<'g> RefillDecompiler<'g> {
    fn new(graph: &'g Graph, refill: &'g Refill) -> Self {
        Self { graph, refill }
    }
}

impl Display for RefillDecompiler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Refill { value, requirement } = self.refill;

        write!(f, "  refill {value}")?;

        if !matches!(requirement, Requirement::Free) {
            write!(
                f,
                ":{}",
                RequirementDecompiler::new(self.graph, requirement, 4, true)
            )?;
        }

        f.write_str("\n")
    }
}

struct ConnectionDecompiler<'g> {
    graph: &'g Graph,
    connection: &'g Connection,
}

impl<'g> ConnectionDecompiler<'g> {
    fn new(graph: &'g Graph, connection: &'g Connection) -> Self {
        Self { graph, connection }
    }
}

impl Display for ConnectionDecompiler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Connection {
            to,
            requirement,
            implicitly_generated: _,
        } = self.connection;

        match &self.graph.nodes[*to] {
            Node::Anchor(anchor) => write!(f, "  conn {}", anchor.identifier)?,
            Node::Pickup(loc_data_entry) => write!(f, "  pickup {}", loc_data_entry.identifier)?,
            Node::State(state_data_entry) => write!(f, "  state {}", state_data_entry.identifier)?,
            Node::LogicalState(identifier) => write!(f, "  state {identifier}")?,
        }

        write!(
            f,
            ":{}",
            RequirementDecompiler::new(self.graph, requirement, 4, true)
        )
    }
}

struct RequirementDecompiler<'g> {
    graph: &'g Graph,
    requirement: &'g Requirement,
    indent: usize,
    start_with_new_line: bool,
}

impl<'g> RequirementDecompiler<'g> {
    fn new(
        graph: &'g Graph,
        requirement: &'g Requirement,
        indent: usize,
        start_with_new_line: bool,
    ) -> Self {
        Self {
            graph,
            requirement,
            indent,
            start_with_new_line,
        }
    }

    fn fork(&self, requirement: &'g Requirement) -> Self {
        Self::new(self.graph, requirement, self.indent, false)
    }

    fn fork_indent(&self, requirement: &'g Requirement, indent: usize) -> Self {
        Self::new(self.graph, requirement, indent, false)
    }
}

impl Display for RequirementDecompiler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start_with_new_line {
            write_new_line(f, self.indent)?;
        }

        match self.requirement {
            Requirement::State(index) => self.graph.nodes[*index].identifier().fmt(f),
            Requirement::And(ands) => {
                let mut indent = self.indent;
                let mut next_or_kind = or_kind(ands.first().unwrap());

                for [and, next_and] in ands.array_windows() {
                    let current_or_kind = next_or_kind;
                    next_or_kind = or_kind(next_and);

                    match current_or_kind {
                        OrKind::Lines => {
                            f.write_str("(")?;
                            write_new_line(f, indent + 2)?;
                            self.fork_indent(and, indent + 2).fmt(f)?;
                            write_new_line(f, indent)?;
                            f.write_str(")")?;
                        }
                        OrKind::Inline | OrKind::NotAnOr => self.fork_indent(and, indent).fmt(f)?,
                    }

                    match (current_or_kind, next_or_kind) {
                        (OrKind::Inline, _) | (_, OrKind::Lines) => {
                            f.write_str(":")?;
                            indent += 2;
                            write_new_line(f, indent)?;
                        }
                        _ => f.write_str(", ")?,
                    }
                }

                self.fork_indent(ands.last().unwrap(), indent).fmt(f)?;

                Ok(())
            }
            Requirement::Or(ors) => {
                let mut iter = ors.iter();

                let first = iter.next().unwrap();
                self.fork(first).fmt(f)?;

                if ors.iter().any(|or| matches!(or, Requirement::And(_))) {
                    for or in iter {
                        write_new_line(f, self.indent)?;
                        self.fork(or).fmt(f)?;
                    }
                } else {
                    for or in iter {
                        f.write_str(" OR ")?;
                        self.fork(or).fmt(f)?;
                    }
                }

                Ok(())
            }
            other => other.fmt(f),
        }
    }
}

fn write_new_line(f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    write!(f, "\n{:<indent$}", "")
}

fn or_kind(requirement: &Requirement) -> OrKind {
    if let Requirement::Or(ors) = requirement {
        if ors.iter().any(|or| matches!(or, Requirement::And(_))) {
            OrKind::Lines
        } else {
            OrKind::Inline
        }
    } else {
        OrKind::NotAnOr
    }
}

#[derive(Clone, Copy)]
enum OrKind {
    Lines,
    Inline,
    NotAnOr,
}
