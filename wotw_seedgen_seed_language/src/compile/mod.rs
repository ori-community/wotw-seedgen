mod command;
mod content;
mod evaluate;
mod expression;
mod function;
mod literal;
mod preprocess;

pub use function::{
    clean_water, energy_fragment, gorlek_ore, health_fragment, keystone, shard, shard_slot, skill,
    spirit_light, teleporter, weapon_upgrade,
};

pub(crate) use function::FunctionIdentifier;

use self::preprocess::{Preprocessor, PreprocessorOutput};
use crate::{
    ast::{self, UberStateType},
    output::{
        intermediate::Literal, ArithmeticOperator, CommandBoolean, CommandFloat, CommandInteger,
        CommandVoid, IntermediateOutput, Operation, SnippetDebugOutput,
    },
    token::TOKENIZER,
    types::uber_state_type,
};
use derivative::Derivative;
use ordered_float::OrderedFloat;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    fmt::Debug,
    io::{self, Write},
};
use wotw_seedgen_assets::{SnippetAccess, Source, UberStateData};
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_parse::{
    parse_ast, Delimited, Error, Identifier, Once, Punctuated, Recoverable, Result,
    SeparatedNonEmpty, Span, Spanned,
};

#[derive(Debug)]
pub struct Compiler<'snippets, 'uberstates> {
    rng: Pcg64Mcg,
    global: GlobalCompilerData<'snippets, 'uberstates>,
    compiled_snippets: FxHashSet<String>,
    errors: Vec<(Source, Vec<Error>)>,
}

/// How many memory slots to reserve for generated calculations
// TODO how much is needed
pub const RESERVED_MEMORY: usize = 10;

pub const fn set_boolean(uber_identifier: UberIdentifier, value: CommandBoolean) -> CommandVoid {
    CommandVoid::StoreBoolean {
        uber_identifier,
        value,
        trigger_events: true,
    }
}
pub const fn set_boolean_value(uber_identifier: UberIdentifier, value: bool) -> CommandVoid {
    set_boolean(uber_identifier, CommandBoolean::Constant { value })
}
pub const fn set_integer(uber_identifier: UberIdentifier, value: CommandInteger) -> CommandVoid {
    CommandVoid::StoreInteger {
        uber_identifier,
        value,
        trigger_events: true,
    }
}
pub const fn set_integer_value(uber_identifier: UberIdentifier, value: i32) -> CommandVoid {
    set_integer(uber_identifier, CommandInteger::Constant { value })
}
pub fn add_integer(uber_identifier: UberIdentifier, amount: CommandInteger) -> CommandVoid {
    CommandVoid::StoreInteger {
        uber_identifier,
        value: CommandInteger::Arithmetic {
            operation: Box::new(Operation {
                left: CommandInteger::FetchInteger { uber_identifier },
                operator: ArithmeticOperator::Add,
                right: amount,
            }),
        },
        trigger_events: true,
    }
}
pub fn add_integer_value(uber_identifier: UberIdentifier, value: i32) -> CommandVoid {
    add_integer(uber_identifier, CommandInteger::Constant { value })
}
pub const fn set_float(uber_identifier: UberIdentifier, value: CommandFloat) -> CommandVoid {
    CommandVoid::StoreFloat {
        uber_identifier,
        value,
        trigger_events: true,
    }
}
pub const fn set_float_value(
    uber_identifier: UberIdentifier,
    value: OrderedFloat<f32>,
) -> CommandVoid {
    set_float(uber_identifier, CommandFloat::Constant { value })
}
pub fn add_float(uber_identifier: UberIdentifier, amount: CommandFloat) -> CommandVoid {
    CommandVoid::StoreFloat {
        uber_identifier,
        value: CommandFloat::Arithmetic {
            operation: Box::new(Operation {
                left: CommandFloat::FetchFloat { uber_identifier },
                operator: ArithmeticOperator::Add,
                right: amount,
            }),
        },
        trigger_events: true,
    }
}
pub fn add_float_value(uber_identifier: UberIdentifier, value: OrderedFloat<f32>) -> CommandVoid {
    add_float(uber_identifier, CommandFloat::Constant { value })
}

pub(crate) trait Compile<'source> {
    type Output;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output;
}

impl<'source, T: Compile<'source>> Compile<'source> for Spanned<T> {
    type Output = T::Output;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.data.compile(compiler)
    }
}
impl<'source, T: Compile<'source>> Compile<'source> for Result<T> {
    type Output = Option<T::Output>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let compiled = self.map(|t| t.compile(compiler));
        compiler.consume_result(compiled)
    }
}
impl<'source, T: Compile<'source>, R> Compile<'source> for Recoverable<T, R> {
    type Output = Option<T::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.result.compile(compiler)
    }
}
impl<'source, T: Compile<'source>> Compile<'source> for Vec<T> {
    type Output = Vec<T::Output>; // TODO experiment with returning iterators instead of vectors from collection compile implementations

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}
impl<'source, Open, Content: Compile<'source>, Close> Compile<'source>
    for Delimited<Open, Content, Close>
{
    type Output = Option<Content::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.content.compile(compiler)
    }
}
impl<'source, T: Compile<'source>> Compile<'source> for Once<T> {
    type Output = T::Output;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.0.compile(compiler)
    }
}
impl<'source, Item: Compile<'source>, Punctuation> Compile<'source>
    for Punctuated<Item, Punctuation>
{
    type Output = Vec<Item::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}
impl<'source, Item: Compile<'source>, Separator> Compile<'source>
    for SeparatedNonEmpty<Item, Separator>
{
    type Output = Vec<Item::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}
impl<'source> Compile<'source> for ast::Snippet<'source> {
    type Output = ();

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.contents.compile(compiler);
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct GlobalCompilerData<'snippets, 'uberstates> {
    pub output: IntermediateOutput,
    pub uber_state_data: &'uberstates UberStateData,
    #[derivative(Debug = "ignore")]
    pub snippet_access: &'snippets dyn SnippetAccess,
    pub events: FxHashMap<String, FxHashMap<String, usize>>,
    pub exported_values: FxHashMap<String, FxHashMap<String, ExportedValue>>,
    pub boolean_ids: IdProvider,
    pub integer_ids: IdProvider,
    pub float_ids: IdProvider,
    pub string_ids: IdProvider,
    pub boolean_state_id: usize,
    pub integer_state_id: usize,
    pub float_state_id: usize,
    pub message_ids: IdProvider,
    pub wheel_ids: IdProvider,
    pub warp_icon_ids: IdProvider,
    // TODO could be a reference
    pub config: FxHashMap<String, FxHashMap<String, String>>,
}
#[derive(Debug)]
pub(crate) enum ExportedValue {
    Function(usize),
    Literal(Literal),
}
impl<'snippets, 'uberstates> GlobalCompilerData<'snippets, 'uberstates> {
    pub(crate) fn new(
        uber_state_data: &'uberstates UberStateData,
        snippet_access: &'snippets dyn SnippetAccess,
        config: FxHashMap<String, FxHashMap<String, String>>,
        debug: bool,
    ) -> Self {
        Self {
            output: IntermediateOutput::new(debug),
            uber_state_data,
            snippet_access,
            events: Default::default(),
            exported_values: Default::default(),
            boolean_ids: IdProvider::new(RESERVED_MEMORY),
            integer_ids: IdProvider::new(RESERVED_MEMORY),
            float_ids: IdProvider::new(RESERVED_MEMORY),
            string_ids: IdProvider::new(RESERVED_MEMORY + 1), // 1 reserved for spirit light strings
            boolean_state_id: 100,
            integer_state_id: 0,
            float_state_id: 150,
            message_ids: IdProvider::new(0),
            wheel_ids: IdProvider {
                offset: 0,
                ids: FxHashMap::from_iter([("root".to_string(), 0)]),
            },
            warp_icon_ids: IdProvider::new(0),
            config,
        }
    }
}
#[derive(Debug)]
pub(crate) struct IdProvider {
    offset: usize,
    ids: FxHashMap<String, usize>,
}
impl IdProvider {
    pub fn new(offset: usize) -> Self {
        Self {
            offset,
            ids: Default::default(),
        }
    }
    pub fn id(&mut self, id: String) -> usize {
        match self.ids.get(&id) {
            None => {
                let len = self.ids.len() + self.offset;
                self.ids.insert(id, len);
                len
            }
            Some(id) => *id,
        }
    }
}
// TODO not sure if all these fields are used anymore since pulling some stuff out into global
pub(crate) struct SnippetCompiler<'compiler, 'source, 'snippets, 'uberstates> {
    pub rng: Pcg64Mcg,
    pub identifier: String, // TODO could be a reference
    pub global: &'compiler mut GlobalCompilerData<'snippets, 'uberstates>,
    pub preprocessed: PreprocessorOutput,
    pub function_indices: FxHashMap<String, usize>, // TODO could maybe be a reference too?
    pub function_imports: FxHashMap<String, String>, // TODO could maybe be a reference too?
    pub variables: FxHashMap<Identifier<'source>, Literal>,
    pub random_pools: FxHashMap<String, Vec<Literal>>, // TODO could maybe be a reference too?
    pub errors: Vec<Error>,
}
const SEED_FAILED_MESSAGE: &str = "Failed to seed child RNG";
impl<'compiler, 'source, 'snippets, 'uberstates>
    SnippetCompiler<'compiler, 'source, 'snippets, 'uberstates>
{
    // TODO weird api
    pub(crate) fn compile<R: Rng>(
        ast: ast::Snippet<'source>,
        rng: &mut R,
        identifier: String,
        global: &'compiler mut GlobalCompilerData<'snippets, 'uberstates>,
        preprocessed: PreprocessorOutput,
    ) -> Self {
        let function_indices = preprocessed
            .functions
            .iter()
            .cloned()
            .zip(global.output.command_lookup.len()..)
            .collect();
        global.output.command_lookup.extend(vec![
            CommandVoid::Multi { commands: vec![] }; // Fill with placeholders for all the functions, this also ensures a sane result if some of the functions fail to compile
            preprocessed.functions.len()
        ]);
        let mut compiler = Self {
            rng: Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE),
            identifier,
            global,
            preprocessed,
            function_indices,
            function_imports: Default::default(),
            variables: Default::default(),
            random_pools: Default::default(),
            errors: Default::default(),
        };
        ast.compile(&mut compiler);
        // TODO feature gate debug?
        if let Some(debug) = &mut compiler.global.output.debug {
            // TODO now it's inefficient that we're returning the whole compiler, could save some clones here
            // ... on the other hand, the things we're cloning are probably supposed to be references anyway
            debug.snippets.insert(
                compiler.identifier.clone(),
                SnippetDebugOutput {
                    variables: compiler
                        .variables
                        .iter()
                        .map(|(k, v)| (k.to_string(), format!("{v:?}")))
                        .collect(),
                    function_indices: compiler.function_indices.clone(),
                },
            );
        }
        compiler
    }

    pub(crate) fn resolve<'a>(
        &'a mut self,
        identifier: &'a Spanned<Identifier>,
    ) -> Option<&'a Literal> {
        let literal = self.variables.get(&identifier.data);
        if literal.is_none() {
            self.errors.push(Error::custom(
                "unknown identifier".to_string(),
                identifier.span(),
            ))
        }
        literal
    }

    pub(crate) fn consume_result<T>(&mut self, result: Result<T>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(err) => {
                self.errors.push(err);
                None
            }
        }
    }

    pub(crate) fn uber_state_type<S: Span>(
        &mut self,
        uber_identifier: UberIdentifier,
        span: S,
    ) -> Option<UberStateType> {
        let ty = uber_state_type(self.global.uber_state_data, uber_identifier);
        if ty.is_none() {
            self.errors
                .push(Error::custom("Unknown UberState".to_string(), span.span()))
        }
        ty
    }
}

impl<'snippets, 'uberstates> Compiler<'snippets, 'uberstates> {
    pub fn new<R: Rng, F: SnippetAccess>(
        rng: &mut R,
        snippet_access: &'snippets F,
        uber_state_data: &'uberstates UberStateData,
        config: FxHashMap<String, FxHashMap<String, String>>,
        debug: bool,
    ) -> Self {
        Self {
            rng: Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE),
            global: GlobalCompilerData::new(uber_state_data, snippet_access, config, debug),
            compiled_snippets: Default::default(),
            errors: Default::default(),
        }
    }

    pub fn compile_snippet(&mut self, identifier: &str) -> std::result::Result<(), String> {
        if !self.compiled_snippets.insert(identifier.to_string()) {
            return Ok(());
        }

        let source = self.global.snippet_access.read_snippet(identifier)?;
        self.global
            .events
            .insert(identifier.to_string(), Default::default());
        self.global
            .exported_values
            .insert(identifier.to_string(), Default::default());
        let mut errors = vec![];

        let ast = parse_ast(&source.content, TOKENIZER);
        // TODO this pattern seems inconvenient, maybe a result with multiple errors and then use extend instead?
        if let Err(err) = ast.trailing {
            errors.push(err);
        }
        match ast.parsed {
            Err(err) => errors.push(err),
            Ok(ast) => {
                let preprocessor = Preprocessor::preprocess(&ast);
                errors.extend(preprocessor.errors);

                for include in &preprocessor.output.includes {
                    if let Err(err) = self.compile_snippet(&include.data) {
                        errors.push(Error::custom(
                            format!("Failed to read snippet: {err}"),
                            include.span.clone(),
                        ));
                    }
                }

                let compiler = SnippetCompiler::compile(
                    ast,
                    &mut self.rng,
                    identifier.to_string(),
                    &mut self.global,
                    preprocessor.output,
                );

                errors.extend(compiler.errors);
            }
        }

        self.errors.push((source, errors));

        Ok(())
    }

    pub fn finish(self) -> CompileResult {
        let mut output = self.global.output;
        if let Some(debug) = &mut output.debug {
            debug.events = self.global.events;
        }

        CompileResult {
            output,
            errors: self.errors,
        }
    }
}

pub struct CompileResult {
    pub output: IntermediateOutput,
    pub errors: Vec<(Source, Vec<Error>)>,
}
impl CompileResult {
    pub fn into_result(self) -> std::result::Result<IntermediateOutput, String> {
        for (source, errors) in self.errors {
            if let Some(err) = errors.into_iter().next() {
                return Err(err.with_source(&source).to_string());
            }
        }
        Ok(self.output)
    }

    pub fn eprint_errors(self) -> (IntermediateOutput, bool) {
        let mut stderr = io::stderr().lock();

        let mut error_count = 0;

        for (source, errors) in self.errors {
            for error in errors {
                error_count += 1;
                error.write_pretty(&source, &mut stderr).unwrap();
            }
        }

        let success = error_count == 0;
        if !success {
            writeln!(
                &mut stderr,
                "Failed to compile Snippets with {error_count} errors."
            )
            .unwrap();
        }

        (self.output, success)
    }
}
