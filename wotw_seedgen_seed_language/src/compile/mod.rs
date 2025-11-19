mod command;
mod content;
mod evaluate;
mod expression;
mod function;
mod helpers;
mod literal;
mod preprocess;

pub use function::{
    clean_water, energy_fragment, gorlek_ore, health_fragment, keystone, shard, shard_slot, skill,
    spirit_light, teleporter, weapon_upgrade, FunctionArg, FunctionIdentifier, FunctionSignature,
};
pub use helpers::{add_float, add_integer, store_boolean, store_float, store_integer};

use self::preprocess::{Preprocessor, PreprocessorOutput};
use crate::{
    ast::{self, Expression, Snippet, UberStateType},
    output::{CommandVoid, IntermediateOutput, Literal, SnippetDebugOutput},
    types::{uber_state_type, InferType, Type},
};
use derivative::Derivative;
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
    Delimited, Error, Identifier, Once, Punctuated, Recoverable, Result, SeparatedNonEmpty, Span,
    SpanEnd, SpanStart, Spanned, SpannedOption,
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
pub const RESERVED_MEMORY: usize = 20;
/// Memory slot for hardcoded calculations
// TODO are there off by one errors here? RESERVED_MEMORY seems to exclude its value as index,
// so maybe the PRIVATE_MEMORY index is supposed to be the value of RESERVED_MEMORY itself.
pub const PRIVATE_MEMORY: usize = RESERVED_MEMORY + 1;

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

impl<'source, T: Compile<'source>> Compile<'source> for Option<T> {
    type Output = Option<T::Output>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.map(|t| t.compile(compiler))
    }
}

impl<'source, T: Compile<'source>> Compile<'source> for SpannedOption<T> {
    type Output = Option<T::Output>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.into_option().map(|t| t.compile(compiler))
    }
}

impl<'source, T: Compile<'source>, R> Compile<'source> for Recoverable<T, R> {
    type Output = Option<T::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        self.value.compile(compiler)
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
    pub exported_values: FxHashMap<String, FxHashMap<String, ExportedValue>>,
    pub boolean_ids: IdProvider,
    pub integer_ids: IdProvider,
    pub float_ids: IdProvider,
    pub string_ids: IdProvider,
    pub boolean_state_id: i32,
    pub integer_state_id: i32,
    pub float_state_id: i32,
    pub message_ids: IdProvider,
    pub box_trigger_ids: IdProvider,
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
            exported_values: Default::default(),
            // 1 additional reserved for hardcoded calculations
            boolean_ids: IdProvider::new(PRIVATE_MEMORY),
            integer_ids: IdProvider::new(PRIVATE_MEMORY),
            float_ids: IdProvider::new(PRIVATE_MEMORY),
            string_ids: IdProvider::new(PRIVATE_MEMORY),
            boolean_state_id: 0,
            integer_state_id: 0,
            float_state_id: 0,
            message_ids: IdProvider::new(0),
            box_trigger_ids: IdProvider::new(0),
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

    pub(crate) fn resolve_function(&mut self, identifier: &Spanned<Identifier>) -> Option<usize> {
        let function = self.function_indices.get(identifier.data.0);

        if function.is_none() {
            self.errors.push(Error::custom(
                "unknown function".to_string(),
                identifier.span(),
            ))
        }

        function.copied()
    }

    pub(crate) fn check_snippet_included(&mut self, snippet_name: &Spanned<&str>) -> bool {
        let included = self.preprocessed.snippet_included(snippet_name.data);

        if !included {
            self.errors.push(
                Error::custom("unknown snippet".to_string(), snippet_name.span.clone())
                    .with_help(format!("try !include(\"{}\")", snippet_name.data)),
            );
        }

        included
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

    pub(crate) fn common_type(&mut self, left: &Expression, right: &Expression) -> Option<Type> {
        let left_ty = left.infer_type(self);
        let right_ty = right.infer_type(self);

        let (left_ty, right_ty) = (left_ty?, right_ty?);

        match (left_ty, right_ty) {
            (Type::UberIdentifier, Type::UberIdentifier) => {
                let left_ty = left.uber_state_type(self);
                let right_ty = right.uber_state_type(self);

                let (left_ty, right_ty) = (left_ty?.into(), right_ty?.into());

                match (left_ty, right_ty) {
                    (Type::Boolean, Type::Boolean) => Some(Type::Boolean),
                    (Type::Float, _) | (_, Type::Float) => Some(Type::Float),
                    (Type::Integer, Type::Integer) => Some(Type::Integer),
                    _ => {
                        self.errors
                            .push(operand_error(left_ty, right_ty, left, right));

                        None
                    }
                }
            }
            (left, right) if left == right => Some(left),
            (Type::UberIdentifier, ty @ (Type::Boolean | Type::Float))
            | (ty @ (Type::Boolean | Type::Float), Type::UberIdentifier) => Some(ty),
            (Type::UberIdentifier, Type::Integer) => left.uber_state_type(self).map(Type::from),
            (Type::Integer, Type::UberIdentifier) => right.uber_state_type(self).map(Type::from),
            (Type::Integer, Type::Float) | (Type::Float, Type::Integer) => Some(Type::Float),
            (Type::Skill, Type::Equipment) | (Type::Equipment, Type::Skill) => {
                Some(Type::Equipment)
            }
            (Type::Teleporter, Type::Zone) | (Type::Zone, Type::Teleporter) => Some(Type::Zone),
            (Type::Skill | Type::WeaponUpgrade, Type::OpherIcon)
            | (Type::OpherIcon, Type::Skill | Type::WeaponUpgrade) => Some(Type::OpherIcon),
            (Type::EquipSlot, Type::WheelBind) | (Type::WheelBind, Type::EquipSlot) => {
                Some(Type::WheelBind)
            }
            (Type::WheelItemPosition, Type::Alignment)
            | (Type::Alignment, Type::WheelItemPosition) => Some(Type::Alignment),
            (Type::WheelItemPosition | Type::Alignment, Type::HorizontalAnchor)
            | (Type::HorizontalAnchor, Type::WheelItemPosition | Type::Alignment) => {
                Some(Type::HorizontalAnchor)
            }
            (Type::WheelItemPosition, Type::VerticalAnchor)
            | (Type::VerticalAnchor, Type::WheelItemPosition) => Some(Type::VerticalAnchor),
            (Type::WheelItemPosition, Type::ScreenPosition)
            | (Type::ScreenPosition, Type::WheelItemPosition) => Some(Type::ScreenPosition),
            (_, Type::String) | (Type::String, _) => Some(Type::String),
            _ => {
                self.errors
                    .push(operand_error(left_ty, right_ty, left, right));

                None
            }
        }
    }
}

fn operand_error(left_ty: Type, right_ty: Type, left: &Expression, right: &Expression) -> Error {
    Error::custom(
        format!("Cannot perform operation on {left_ty} and {right_ty}"),
        left.span_start()..right.span_end(),
    )
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
            .exported_values
            .insert(identifier.to_string(), Default::default());

        let ast = Snippet::parse(&source.content);
        let mut errors = ast.errors;

        if let Some(ast) = ast.parsed {
            let preprocessor = Preprocessor::preprocess(&ast);
            errors.extend(preprocessor.errors);

            for (path, identifier, value) in &preprocessor.output.config_sets {
                // TODO do something if set already?
                self.global
                    .config
                    .entry(path.clone())
                    .or_default()
                    .insert(identifier.clone(), value.clone());
            }

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

        self.errors.push((source, errors));

        Ok(())
    }

    pub fn finish(self) -> CompileResult {
        CompileResult {
            output: self.global.output,
            errors: self.errors,
        }
    }
}

pub struct CompileResult {
    pub output: IntermediateOutput,
    pub errors: Vec<(Source, Vec<Error>)>,
}

impl CompileResult {
    pub fn eprint_errors(self) -> Option<IntermediateOutput> {
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

        success.then_some(self.output)
    }
}
