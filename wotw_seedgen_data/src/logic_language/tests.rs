use crate::{
    assets::{AssetFileAccess, TestAccess},
    logic_language::{
        ast::{
            Amount, And, Areas, Content, Dedent, GroupContent, Indent, LogicIdentifier, Or,
            PlainRequirement, Region, RegionKeyword, Requirement, RequirementGroup,
            RequirementLine,
        },
        output::{Graph, Node},
        token::{Token, Tokenizer},
    },
    DEFAULT_SPAWN,
};
use wotw_seedgen_parse::{
    Ast, Identifier, Parser, Recoverable, SeparatedNonEmpty, Source, Spanned, Symbol, Tokenize,
};

#[test]
fn tokenization() {
    use Token::*;

    let source = "
anchor My.Anchor at -420, 69:
  nospawn

  conn Other.Anchor:
    moki: Sword OR Combat=2xBee
";

    assert_eq!(
        Tokenizer
            .tokenize(source)
            .tokens
            .into_iter()
            .map(|(token, _)| token)
            .collect::<Vec<_>>(),
        vec![
            Anchor,
            LogicIdentifier,
            At,
            Integer,
            Symbol,
            Integer,
            Symbol,
            Indent,
            NoSpawn,
            Newline,
            Connection,
            LogicIdentifier,
            Symbol,
            Indent,
            Identifier,
            Symbol,
            Identifier,
            Or,
            Identifier,
            Symbol,
            Integer,
            Symbol,
            Identifier,
            Dedent,
            Dedent,
        ]
    );
}

#[test]
fn ast() {
    let source = "region GorlekMines:\n    moki: GorlekMines.ElevatorFixed OR Shuriken=1\n";
    let mut parser = Parser::new(source, Tokenizer);
    assert_eq!(
        Content::ast(&mut parser).unwrap(),
        Content::Region(
            Spanned {
                data: RegionKeyword,
                span: 0..6
            },
            Recoverable::some(Region {
                identifier: Spanned {
                    data: Identifier("GorlekMines"),
                    span: 7..18
                },
                requirements: RequirementGroup {
                    colon: Spanned {
                        data: Symbol,
                        span: 18..19
                    },
                    content: Recoverable::some(GroupContent {
                        indent: Spanned {
                            data: Indent,
                            span: 19..24
                        },
                        content: SeparatedNonEmpty {
                            first: RequirementLine {
                                ands: vec![(
                                    Requirement::Plain(PlainRequirement {
                                        identifier: Spanned {
                                            data: Identifier("moki"),
                                            span: 24..28,
                                        },
                                        amount: None
                                    }),
                                    And::Colon(Symbol)
                                )],
                                ors: SeparatedNonEmpty {
                                    first: Requirement::State(Spanned {
                                        data: LogicIdentifier("GorlekMines.ElevatorFixed"),
                                        span: 30..55,
                                    }),
                                    more: vec![(
                                        Or,
                                        Requirement::Plain(PlainRequirement {
                                            identifier: Spanned {
                                                data: Identifier("Shuriken"),
                                                span: 59..67
                                            },
                                            amount: Some(Amount {
                                                equals: Spanned {
                                                    data: Symbol,
                                                    span: 67..68
                                                },
                                                value: Recoverable::some(Spanned {
                                                    data: 1,
                                                    span: 68..69
                                                })
                                            })
                                        })
                                    )]
                                },
                                group: None
                            },
                            more: vec![]
                        },
                        dedent: Spanned {
                            data: Dedent,
                            span: 69..source.len()
                        }
                    }),
                }
            }),
        ),
    );
}

#[test]
fn compile() {
    let source = Source::new(
        "areas.wotw".to_string(),
        include_str!("../../../wotw_seedgen/areas.wotw").to_string(),
    );

    let areas = Areas::parse(&source.content)
        .eprint_errors(&source)
        .unwrap();

    let Some(graph) = Graph::compile(
        areas,
        TestAccess.loc_data().unwrap(),
        TestAccess.state_data().unwrap(),
        &[],
    )
    .eprint_errors(&source) else {
        panic!("Failed to parse areas.wotw");
    };

    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    match &graph.nodes[spawn] {
        Node::Anchor(anchor) => {
            let adjacent = anchor
                .connections
                .iter()
                .map(|connection| graph.nodes[connection.to].identifier())
                .collect::<Vec<_>>();
            assert!(adjacent.contains(&"NonGladesTeleporter"));
            assert!(adjacent.contains(&"MarshSpawn.GrappleHC"));
            assert!(adjacent.contains(&"Teleporters"));
        }
        _ => panic!(),
    }
}
