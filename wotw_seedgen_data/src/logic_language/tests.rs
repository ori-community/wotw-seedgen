use std::cmp::Ordering;

use crate::{
    assets::{AssetFileAccess, TestAccess},
    logic_language::{
        ast::Paths,
        output::{Graph, Node},
        token::{Token, Tokenizer},
    },
    DEFAULT_SPAWN,
};
use smallvec::smallvec;
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
    use crate::logic_language::ast::*;

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
        "paths.wotwl".to_string(),
        include_str!("../../../assets/logic/paths.wotwl").to_string(),
    );

    let paths = Paths::parse(&source.content)
        .eprint_errors(&source)
        .unwrap();

    let Some(graph) = Graph::compile(
        paths,
        TestAccess.loc_data().unwrap(),
        TestAccess.state_data().unwrap(),
        &[],
    )
    .eprint_errors(&source) else {
        panic!("Failed to parse paths.wotwl");
    };

    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    match &graph.nodes[spawn] {
        Node::Anchor(anchor) => {
            let adjacent = anchor
                .connections
                .iter()
                .map(|connection| graph.nodes[connection.to].identifier())
                .collect::<Vec<_>>();
            assert!(adjacent.contains(&"MarshSpawn.ToOpherBarrier"));
            assert!(adjacent.contains(&"MarshSpawn.GrappleHF"));
            assert!(adjacent.contains(&"Teleporters"));
        }
        _ => panic!(),
    }
}

#[test]
fn logical_cmp() {
    use crate::{
        logic_language::output::{Enemy::*, Requirement::*},
        Difficulty::*,
        Skill::*,
    };
    use Ordering::*;

    fn partial_ord_str(ord: Option<Ordering>) -> &'static str {
        match ord {
            None => "≠",
            Some(Less) => "<",
            Some(Equal) => "=",
            Some(Greater) => ">",
        }
    }

    macro_rules! test {
        (@ord "<") => { Some(Less) };
        (@ord "=") => { Some(Equal) };
        (@ord ">") => { Some(Greater) };
        (@ord "≠") => { None };

        ($left_req:expr, $ord:tt, $right_req:expr $(,)?) => {
            let actual = $left_req.logical_cmp(&$right_req);

            assert_eq!(
                actual,
                test!(@ord $ord),
                "expected {left_req} {expected} {right_req}, but got {left_req} {actual} {right_req}",
                left_req = $left_req,
                right_req = $right_req,
                expected = $ord,
                actual = partial_ord_str(actual),
            );
        };
    }

    test!(Free, "<", Impossible);

    test!(Difficulty(Kii), ">", Difficulty(Gorlek));

    test!(BreakWall((20.).into()), ">", BreakWall((16.).into()));

    test!(
        And(vec![
            EnergySkill(Bow, (1.).into()),
            EnergySkill(Shuriken, (1.).into())
        ]),
        "≠",
        And(vec![
            EnergySkill(Shuriken, (1.).into()),
            EnergySkill(Bow, (1.).into())
        ]),
    );

    test!(
        Combat(smallvec![(Bat, 1), (Lizard, 1)]),
        "=",
        Combat(smallvec![(Bat, 1), (Lizard, 1)]),
    );

    test!(
        Combat(smallvec![(Lizard, 1), (Bat, 1)]),
        "≠",
        Combat(smallvec![(Bat, 1), (Lizard, 1)]),
    );

    test!(
        Combat(smallvec![(Bat, 1), (Lizard, 1)]),
        "<",
        Combat(smallvec![(Bat, 1), (Lizard, 1), (Mantis, 1)]),
    );

    test!(
        And(vec![Skill(DoubleJump), Skill(Dash)]),
        "=",
        And(vec![Skill(Dash), Skill(DoubleJump)]),
    );

    test!(
        And(vec![Skill(DoubleJump), Skill(Launch)]),
        "≠",
        And(vec![Skill(Dash), Skill(DoubleJump)]),
    );

    test!(
        And(vec![Skill(DoubleJump), Skill(Launch), Skill(Dash)]),
        ">",
        And(vec![Skill(Dash), Skill(DoubleJump)]),
    );

    test!(And(vec![Skill(DoubleJump), Skill(Dash)]), ">", Skill(Dash));

    test!(
        Or(vec![Skill(DoubleJump), Skill(Dash)]),
        "=",
        Or(vec![Skill(Dash), Skill(DoubleJump)]),
    );

    test!(
        Or(vec![
            EnergySkill(Bow, (1.).into()),
            EnergySkill(Shuriken, (1.).into())
        ]),
        "=",
        Or(vec![
            EnergySkill(Shuriken, (1.).into()),
            EnergySkill(Bow, (1.).into())
        ]),
    );

    test!(
        Or(vec![Skill(DoubleJump), Skill(Launch), Skill(Dash)]),
        "<",
        Or(vec![Skill(Dash), Skill(DoubleJump)]),
    );

    test!(Or(vec![Skill(DoubleJump), Skill(Dash)]), "<", Skill(Dash));
}
