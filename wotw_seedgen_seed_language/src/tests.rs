use crate::{
    ast::{
        Expression, ExpressionValue, FunctionCall, Literal, UberIdentifier, UberIdentifierName,
        UberIdentifierNumeric,
    },
    compile::Compiler,
    output::IntermediateOutput,
    token::TOKENIZER,
};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use std::{
    ffi::OsStr,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};
use wotw_seedgen_assets::{SnippetAccess, Source};
use wotw_seedgen_parse::{
    parse_ast, Delimited, Identifier, Punctuated, Recoverable, Spanned, Symbol,
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
        member: Recoverable::new(Ok(Spanned {
            data: 786,
            span: 9..12,
        })),
    };
    let uber_identifier = parse_ast(source, TOKENIZER).parsed;
    assert_eq!(uber_identifier, Ok(expected.clone()));
    let uber_identifier = parse_ast(source, TOKENIZER).parsed;
    assert_eq!(uber_identifier, Ok(UberIdentifier::Numeric(expected)));

    let source = "   GladesTown.  TuleySpawned  ";
    let expected = UberIdentifierName {
        group: Spanned {
            data: Identifier("GladesTown"),
            span: 3..13,
        },
        period: Symbol::<'.'>,
        member: Recoverable::new(Ok(Spanned {
            data: Identifier("TuleySpawned"),
            span: 16..28,
        })),
    };
    let uber_identifier = parse_ast(source, TOKENIZER).parsed;
    assert_eq!(uber_identifier, Ok(expected.clone()));
    let uber_identifier = parse_ast(source, TOKENIZER).parsed;
    assert_eq!(uber_identifier, Ok(UberIdentifier::Name(expected)));

    let source = "   456  ";
    let error = parse_ast::<_, UberIdentifier>(source, TOKENIZER)
        .parsed
        .unwrap_err();
    assert_eq!(error.to_string(), "expected '|'".to_string());
    assert_eq!(error.span, 3..8);

    let source = "   GladesTown.  5TuleySpawned  ";
    match parse_ast::<_, UberIdentifier>(source, TOKENIZER)
        .parsed
        .unwrap()
    {
        UberIdentifier::Name(uber_identifier) => {
            let error = uber_identifier.member.result.unwrap_err();
            assert_eq!(error.to_string(), "expected identifier".to_string());
            assert_eq!(error.span, 16..17);
        }
        _ => panic!(),
    }

    let source = " $$  ";
    let error = parse_ast::<_, UberIdentifier>(source, TOKENIZER)
        .parsed
        .unwrap_err();
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
            content: Ok(Punctuated {
                items: vec![(
                    Expression::Value(ExpressionValue::Literal(Spanned {
                        data: Literal::UberIdentifier(UberIdentifier::Name(UberIdentifierName {
                            group: Spanned {
                                data: Identifier("TuleyShop"),
                                span: 4..13,
                            },
                            period: Symbol::<'.'>,
                            member: Recoverable::new(Ok(Spanned {
                                data: Identifier("BlueMoon"),
                                span: 14..22,
                            })),
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
            close: Ok(Spanned {
                data: Symbol::<')'>,
                span: 25..26,
            }),
        },
    };
    let function_call = parse_ast::<_, FunctionCall>(source, TOKENIZER).parsed;
    assert_eq!(function_call, Ok(expected));
}

struct ExampleFileAccess(&'static str);
impl SnippetAccess for ExampleFileAccess {
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
}

struct TestFileAccess;
impl SnippetAccess for TestFileAccess {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        let id = format!("{}/assets/snippets/{}.wotws", *WORKDIR, identifier);
        let content = fs::read_to_string(&id).map_err(|err| err.to_string())?;
        Ok(Source { id, content })
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        let mut full_path = PathBuf::from(*WORKDIR);
        full_path.push("assets/snippets");
        full_path.push(path);
        fs::read(full_path).map_err(|err| err.to_string())
    }

    fn available_snippets(&self) -> Vec<String> {
        unimplemented!()
    }
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

fn test_str(source: &'static str) {
    let snippet_access = ExampleFileAccess(source);
    let mut compiler = test_compiler(&snippet_access);
    compiler.compile_snippet("").unwrap();
    let (_, success) = compiler.finish().eprint_errors();
    assert!(success);
}

#[test]
fn coersions() {
    test_str("!state(float, Float)  on float > 5 {}");
}

#[test]
fn snippets() {
    // TODO remove test output
    fn write_test_output(filename: impl Display, output: &IntermediateOutput) {
        fs::create_dir_all(format!("{}/target/snippet-test", *WORKDIR)).unwrap();
        fs::write(
            format!("{}/target/snippet-test/{}", *WORKDIR, filename),
            format!("{:#?}", output),
        )
        .unwrap();
    }

    let test_with_config = [(
        "relics".to_string(),
        [("relic_count".to_string(), "5".to_string())]
            .into_iter()
            .collect::<FxHashMap<_, _>>(),
    )]
    .into_iter()
    .collect::<FxHashMap<_, _>>();

    let snippets = fs::read_dir(format!("{}/assets/snippets", *WORKDIR))
        .unwrap()
        .map(|snippet| snippet.unwrap().path())
        .filter(|path| path.extension() == Some(OsStr::new("wotws")))
        .map(|path| path.file_stem().unwrap().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    let mut compiler = test_compiler(&TestFileAccess);

    for identifier in &snippets {
        compiler.compile_snippet(identifier).unwrap();
    }
    let (output, success) = compiler.finish().eprint_errors();
    assert!(success);

    write_test_output("_final", &output);

    for identifier in &snippets {
        let mut compiler = test_compiler(&TestFileAccess);

        compiler.compile_snippet(identifier).unwrap();

        let (output, success) = compiler.finish().eprint_errors();
        assert!(success);

        write_test_output(identifier, &output);
    }

    for identifier in test_with_config.keys() {
        let mut compiler = test_compiler_with_config(&TestFileAccess, test_with_config.clone());

        compiler.compile_snippet(identifier).unwrap();

        let (output, success) = compiler.finish().eprint_errors();
        assert!(success);

        write_test_output(format!("{identifier} (alternate config)"), &output);
    }
}
