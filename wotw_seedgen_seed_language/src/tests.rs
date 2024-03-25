use crate::{
    ast::{
        Expression, ExpressionValue, FunctionCall, Literal, UberIdentifier, UberIdentifierName,
        UberIdentifierNumeric,
    },
    compile::Compiler,
    output::CompilerOutput,
    token::TOKENIZER,
};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use std::{
    ffi::OsStr,
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};
use wotw_seedgen_assets::{SnippetAccess, Source};
use wotw_seedgen_parse::{parse_ast, Delimited, Identifier, Punctuated, Spanned, Symbol};
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
        member: Spanned {
            data: 786,
            span: 9..12,
        },
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
        member: Spanned {
            data: Identifier("TuleySpawned"),
            span: 16..28,
        },
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
    assert_eq!(error.span, 8..8);

    let source = "   GladesTown.  5TuleySpawned  ";
    let error = parse_ast::<_, UberIdentifier>(source, TOKENIZER)
        .parsed
        .unwrap_err();
    assert_eq!(error.to_string(), "expected identifier".to_string());
    assert_eq!(error.span, 16..17);

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
                            member: Spanned {
                                data: Identifier("BlueMoon"),
                                span: 14..22,
                            },
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

#[test]
fn snippets() {
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
    }

    // TODO delete temporary test stuff
    let mut compiler = Compiler::new(
        &mut rand::thread_rng(),
        &TestFileAccess,
        &*UBER_STATE_DATA,
        Default::default(),
    );
    compiler.compile_snippet("rapid_hammer_core").unwrap();
    let output = compiler.finish(&mut &mut io::stderr()).unwrap();
    assert!(output.success);

    fn write_test_output(filename: impl Display, output: &CompilerOutput) {
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

    let mut compiler = Compiler::new(
        &mut rand::thread_rng(),
        &TestFileAccess,
        &*UBER_STATE_DATA,
        Default::default(),
    );

    for identifier in &snippets {
        compiler.compile_snippet(identifier).unwrap();
    }
    let output = compiler.finish(&mut io::stderr()).unwrap();
    assert!(output.success);

    write_test_output("_final", &output);

    snippets
        .iter()
        .map(|identifier| {
            let mut compiler = Compiler::new(
                &mut rand::thread_rng(),
                &TestFileAccess,
                &*UBER_STATE_DATA,
                Default::default(),
            );

            compiler.compile_snippet(identifier).unwrap();

            let output = compiler.finish(&mut io::stderr()).unwrap();

            write_test_output(identifier, &output);

            output.success.then_some(())
        })
        .chain(test_with_config.keys().map(|identifier| {
            let mut compiler = Compiler::new(
                &mut rand::thread_rng(),
                &TestFileAccess,
                &*UBER_STATE_DATA,
                test_with_config.clone(),
            );

            compiler.compile_snippet(identifier).unwrap();

            let output = compiler.finish(&mut io::stderr()).unwrap();

            write_test_output(format!("{identifier} (alternate config)"), &output);

            output.success.then_some(())
        }))
        .collect::<Vec<_>>()
        .into_iter()
        .try_for_each(|t| t)
        .unwrap();
}

// TODO delete
#[test]
fn dangerous() {
    struct TestFileAccess;
    impl SnippetAccess for TestFileAccess {
        fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
            let mut id = format!("{}/assets/dangerous/{}.wotws", *WORKDIR, identifier);
            let content = fs::read_to_string(&id)
                .or_else(|_| {
                    id = format!("{}/assets/snippets/{}.wotws", *WORKDIR, identifier);
                    fs::read_to_string(&id)
                })
                .map_err(|err| err.to_string())?;
            Ok(Source { id, content })
        }
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
            let mut full_path = PathBuf::from(*WORKDIR);
            full_path.push("assets/dangerous");
            full_path.push(path);
            fs::read(full_path).map_err(|err| err.to_string())
        }
    }

    let mut compiler = Compiler::new(
        &mut rand::thread_rng(),
        &TestFileAccess,
        &*UBER_STATE_DATA,
        Default::default(),
    );
    compiler.compile_snippet("main").unwrap();
    let output = compiler.finish(&mut io::stderr()).unwrap();
    assert!(output.success);
}
