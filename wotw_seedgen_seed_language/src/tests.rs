use crate::{
    ast::{
        parse_seed_ast, ClientEvent, ConstantDiscriminants, Expression, ExpressionValue,
        FunctionCall, Literal, UberIdentifier, UberIdentifierName, UberIdentifierNumeric,
    },
    compile::{Compiler, PRIVATE_MEMORY},
    output::{CommandVoid, IntermediateOutput},
};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use std::{fmt::Display, path::Path};
use strum::VariantArray;
use wotw_seedgen_assets::{FileAccess, SnippetAccess, Source};
use wotw_seedgen_data::{
    Alignment, CoordinateSystem, EquipSlot, Equipment, GromIcon, HorizontalAnchor, LupoIcon,
    MapIcon, OpherIcon, ScreenPosition, Shard, Skill, Teleporter, TuleyIcon, VerticalAnchor,
    WeaponUpgrade, WheelBind, WheelItemPosition, Zone,
};
use wotw_seedgen_parse::{
    Delimited, Identifier, Punctuated, Recoverable, Spanned, SpannedOption, Symbol,
};
use wotw_seedgen_static_assets::UBER_STATE_DATA;

#[test]
fn uber_identifier() {
    let source = "   456 | 786  ";
    let expected = UberIdentifierNumeric {
        group: Spanned {
            data: 456,
            span: 3..6,
        },
        separator: Symbol::<'|'>,
        member: Recoverable::some(Spanned {
            data: 786,
            span: 9..12,
        }),
    };
    let uber_identifier = parse_seed_ast(source).parsed;
    assert_eq!(uber_identifier, Some(expected.clone()));
    let uber_identifier = parse_seed_ast(source).parsed;
    assert_eq!(uber_identifier, Some(UberIdentifier::Numeric(expected)));

    let source = "   GladesTown.  TuleySpawned  ";
    let expected = UberIdentifierName {
        group: Spanned {
            data: Identifier("GladesTown"),
            span: 3..13,
        },
        period: Symbol::<'.'>,
        member: Recoverable::some(Spanned {
            data: Identifier("TuleySpawned"),
            span: 16..28,
        }),
    };
    let uber_identifier = parse_seed_ast(source).parsed;
    assert_eq!(uber_identifier, Some(expected.clone()));
    let uber_identifier = parse_seed_ast(source).parsed;
    assert_eq!(uber_identifier, Some(UberIdentifier::Name(expected)));

    let source = "   456  ";
    let error = parse_seed_ast::<UberIdentifier>(source)
        .errors
        .pop()
        .unwrap();
    assert_eq!(error.to_string(), "expected '|'".to_string());
    assert_eq!(error.span, 3..8);

    let source = "   GladesTown.  5TuleySpawned  ";
    let result = parse_seed_ast::<UberIdentifier>(source);
    assert_eq!(
        result.parsed,
        Some(UberIdentifier::Name(UberIdentifierName {
            group: Spanned {
                data: Identifier("GladesTown"),
                span: 3..13
            },
            period: Symbol,
            member: Recoverable::none(16..17),
        }))
    );
    let error = result.errors.into_iter().next().unwrap();
    assert_eq!(error.to_string(), "expected identifier".to_string());
    assert_eq!(error.span, 16..17);

    let source = " $$  ";
    let error = parse_seed_ast::<UberIdentifier>(source)
        .errors
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(
        error.to_string(),
        "expected identifier or integer".to_string()
    );
    assert_eq!(error.span, 1..2);
}

#[test]
fn function_call() {
    let source = "set(TuleyShop.BlueMoon, 3)";
    let expected = FunctionCall {
        identifier: Spanned {
            data: Identifier("set"),
            span: 0..3,
        },
        parameters: Delimited {
            open: Spanned {
                data: Symbol::<'('>,
                span: 3..4,
            },
            content: Some(Punctuated {
                items: vec![(
                    Expression::Value(ExpressionValue::Literal(Spanned {
                        data: Literal::UberIdentifier(UberIdentifier::Name(UberIdentifierName {
                            group: Spanned {
                                data: Identifier("TuleyShop"),
                                span: 4..13,
                            },
                            period: Symbol::<'.'>,
                            member: Recoverable::some(Spanned {
                                data: Identifier("BlueMoon"),
                                span: 14..22,
                            }),
                        })),
                        span: 4..22,
                    })),
                    Symbol::<','>,
                )],
                last: Some(Expression::Value(ExpressionValue::Literal(Spanned {
                    data: Literal::Integer(3),
                    span: 24..25,
                }))),
            }),
            close: SpannedOption::Some(Spanned {
                data: Symbol::<')'>,
                span: 25..26,
            }),
        },
    };
    let function_call = parse_seed_ast::<FunctionCall>(source).parsed;
    assert_eq!(function_call, Some(expected));
}

struct ExampleFileAccess<'a>(&'a str);
impl SnippetAccess for ExampleFileAccess<'_> {
    fn read_snippet(&self, _identifier: &str) -> Result<Source, String> {
        Ok(Source {
            id: String::new(),
            content: self.0.to_string(),
        })
    }

    fn read_file(&self, _path: &Path) -> Result<Vec<u8>, String> {
        unimplemented!()
    }

    fn available_snippets(&self) -> Vec<String> {
        unimplemented!()
    }
}

lazy_static! {
    // works while debugging, but doesn't work to jump into code from errors
    // static ref WORKDIR: String = {
    //     let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     path.pop();
    //     path.to_string_lossy().to_string()
    // };

    // works to jump into code from errors, but doesn't work while debugging
    static ref WORKDIR: &'static str = "..";

    static ref TEST_FILE_ACCESS: FileAccess = FileAccess::new([
        format!("{}/assets/snippets", *WORKDIR),
        format!("{}/assets/toolseeds", *WORKDIR)
    ]);
}

fn test_compiler<'snippets, F: SnippetAccess>(
    snippet_access: &'snippets F,
) -> Compiler<'snippets, 'static> {
    test_compiler_with_config(snippet_access, Default::default())
}

fn test_compiler_with_config<'snippets, F: SnippetAccess>(
    snippet_access: &'snippets F,
    config: FxHashMap<String, FxHashMap<String, String>>,
) -> Compiler<'snippets, 'static> {
    Compiler::new(
        &mut rand::thread_rng(),
        snippet_access,
        &UBER_STATE_DATA,
        config,
        false,
    )
}

fn test_str(source: &str) -> IntermediateOutput {
    eprintln!("testing snippet:\n{source}");

    let snippet_access = ExampleFileAccess(source);
    let mut compiler = test_compiler(&snippet_access);

    compiler.compile_snippet("").unwrap();

    let output = compiler.finish().eprint_errors().unwrap();

    output
}

#[test]
fn coersions() {
    test_str("on spawn store(player.spiritLight, player.spiritLight + player.spiritLight)");
    test_str("!state(float, Float)  on float > 5 {}");
    test_str("on spawn item_message(player.spiritLight - 1)");
    test_str("on spawn item_message((player.spiritLight - player.gorlekOre) + \"SL/Ore\")");

    fn test_variants_with_prefix<T: VariantArray + Display>(prefix: &str, f: fn(&T) -> String) {
        let variants = T::VARIANTS.iter().map(f).collect::<String>();

        test_str(&format!("{prefix} {{{}}}", variants));
    }

    fn test_variants<T: VariantArray + Display>(f: fn(&T) -> String) {
        test_variants_with_prefix("on spawn", f)
    }

    for kind in ConstantDiscriminants::VARIANTS {
        match kind {
            ConstantDiscriminants::ClientEvent => test_variants::<ClientEvent>(|client_event| {
                format!("trigger_client_event({client_event})")
            }),
            ConstantDiscriminants::Skill => {
                test_variants::<Skill>(|skill| format!("skill({skill})"))
            }
            ConstantDiscriminants::Shard => {
                test_variants::<Shard>(|shard| format!("shard({shard})"))
            }
            ConstantDiscriminants::Teleporter => {
                // TODO maybe the TP suffix shouldn't be in the Teleporter Display impl
                test_variants::<Teleporter>(|teleporter| format!("teleporter({teleporter:?})"))
            }
            ConstantDiscriminants::WeaponUpgrade => {
                test_variants::<WeaponUpgrade>(|weapon_upgrade| {
                    format!("weapon_upgrade({weapon_upgrade})")
                })
            }
            ConstantDiscriminants::Equipment => {
                test_variants::<Equipment>(|equipment| format!("unequip({equipment})"))
            }
            ConstantDiscriminants::Zone => {
                test_variants::<Zone>(|zone| format!("if current_zone() == {zone} {{}}"))
            }
            ConstantDiscriminants::OpherIcon => test_variants::<OpherIcon>(|opher_icon| {
                format!("set_shop_item_icon(OpherShop.Blaze, {opher_icon})")
            }),
            ConstantDiscriminants::LupoIcon => test_variants::<LupoIcon>(|lupo_icon| {
                format!("set_shop_item_icon(OpherShop.Blaze, {lupo_icon})")
            }),
            ConstantDiscriminants::GromIcon => test_variants::<GromIcon>(|grom_icon| {
                format!("set_shop_item_icon(OpherShop.Blaze, {grom_icon})")
            }),
            ConstantDiscriminants::TuleyIcon => test_variants::<TuleyIcon>(|tuley_icon| {
                format!("set_shop_item_icon(OpherShop.Blaze, {tuley_icon})")
            }),
            ConstantDiscriminants::MapIcon => {
                test_variants_with_prefix::<MapIcon>("!if true", |map_icon| {
                    format!("!item_data_map_icon(item_message(\"{map_icon}\"), {map_icon})")
                })
            }
            ConstantDiscriminants::EquipSlot => {
                test_variants::<EquipSlot>(|equip_slot| format!("equip({equip_slot}, Bow)"))
            }
            ConstantDiscriminants::WheelItemPosition => {
                test_variants::<WheelItemPosition>(|wheel_item_position| {
                    format!("destroy_wheel_item(\"root\", {wheel_item_position})")
                })
            }
            ConstantDiscriminants::WheelBind => test_variants::<WheelBind>(|wheel_bind| {
                format!("set_wheel_item_action(\"root\", Top, {wheel_bind}, {{}})")
            }),
            ConstantDiscriminants::Alignment => test_variants::<Alignment>(|alignment| {
                format!("set_message_alignment(\"\", {alignment})")
            }),
            ConstantDiscriminants::HorizontalAnchor => {
                test_variants::<HorizontalAnchor>(|horizontal_anchor| {
                    format!("set_message_horizontal_anchor(\"\", {horizontal_anchor})")
                })
            }
            ConstantDiscriminants::VerticalAnchor => {
                test_variants::<VerticalAnchor>(|vertical_anchor| {
                    format!("set_message_vertical_anchor(\"\", {vertical_anchor})")
                })
            }
            ConstantDiscriminants::ScreenPosition => {
                test_variants::<ScreenPosition>(|screen_position| {
                    format!("set_message_screen_position(\"\", {screen_position})")
                })
            }
            ConstantDiscriminants::CoordinateSystem => {
                test_variants::<CoordinateSystem>(|coordinate_system| {
                    format!("set_message_coordinate_system(\"\", {coordinate_system})")
                })
            }
        }
    }
}

#[test]
fn operator_precedence() {
    fn test_precedence(term: &str, value: i32) {
        let output = test_str(&format!("on spawn set_integer(\"oriLurk\", {term})"));
        assert_eq!(
            output.events[0].command,
            CommandVoid::SetInteger {
                id: PRIVATE_MEMORY,
                value: value.into()
            }
        );
    }

    test_precedence("3 - 2 - 1", 0);
    test_precedence("4 / 2 / 2", 1);
    test_precedence("4 / 2 + 2", 4);
    test_precedence("4 / (2 + 2)", 1);
}

#[test]
fn snippets() {
    let snippets = TEST_FILE_ACCESS.available_snippets();

    let mut compiler = test_compiler(&*TEST_FILE_ACCESS);

    for identifier in &snippets {
        compiler.compile_snippet(identifier).unwrap();
    }

    compiler.finish().eprint_errors().unwrap();

    for identifier in &snippets {
        let mut compiler = test_compiler(&*TEST_FILE_ACCESS);

        compiler.compile_snippet(identifier).unwrap();

        compiler.finish().eprint_errors().unwrap();
    }

    let test_with_config = [(
        "relics".to_string(),
        [("relic_count".to_string(), "5".to_string())]
            .into_iter()
            .collect::<FxHashMap<_, _>>(),
    )]
    .into_iter()
    .collect::<FxHashMap<_, _>>();

    for identifier in test_with_config.keys() {
        let mut compiler = test_compiler_with_config(&*TEST_FILE_ACCESS, test_with_config.clone());

        compiler.compile_snippet(identifier).unwrap();

        compiler.finish().eprint_errors().unwrap();
    }
}
