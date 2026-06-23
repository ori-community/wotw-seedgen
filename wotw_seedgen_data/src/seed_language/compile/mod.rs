mod command;
mod content;
mod error;
mod evaluate;
mod expression;
mod function;
mod helpers;
mod ids;
mod lint;
mod literal;
mod preprocess;
mod scope;

pub use function::{
    clean_water, empty, energy_fragment, gorlek_ore, health_fragment, keystone, shard, shard_slot,
    skill, spirit_light, teleporter, weapon_upgrade, FunctionArg, FunctionIdentifier,
    FunctionSignature,
};
pub use helpers::{add_float, add_integer, store_boolean, store_float, store_integer};

use self::preprocess::{Preprocessor, PreprocessorOutput};
use crate::{
    assets::{SnippetAccess, UberStateData},
    seed_language::{
        ast::{self, Expression, Snippet, UberStateType},
        compile::{
            self, error::operand_error, ids::IdResolver, lint::LintData,
            preprocess::PreprocessedFunction, scope::Scopes,
        },
        output::{IntermediateOutput, Literal, SnippetDebugOutput, VariableValue},
        types::{uber_state_type, InferType, Type},
    },
    UberIdentifier,
};
use derivative::Derivative;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use std::{
    collections::hash_map::Entry,
    fmt::Debug,
    io::{self, Write},
    iter,
    path::PathBuf,
};
use wotw_seedgen_parse::{
    Delimited, Error, Identifier, Once, Punctuated, Recoverable, Result, SeparatedNonEmpty,
    Severity, Source, Span, Spanned, SpannedOption,
};

#[derive(Debug)]
pub struct Compiler<'snippets, 'uberstates> {
    rng: Pcg64Mcg,
    global: GlobalCompilerData<'snippets, 'uberstates>,
    compiled_snippets: FxHashMap<String, CompileState>,
    errors: FxHashMap<String, (Source, Vec<Error>)>,
}

#[derive(Debug)]
enum CompileState {
    Started,
    Finished,
}

/// How many memory slots to reserve for generated calculations
// TODO how much is needed
pub const RESERVED_MEMORY: usize = 20;
/// Memory slot for hardcoded calculations
pub const PRIVATE_MEMORY: usize = RESERVED_MEMORY;
/// Start of freely assignable memory
pub const FREE_MEMORY_START: usize = PRIVATE_MEMORY + 1;

pub(crate) trait Compile<'source> {
    type Output;

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output;
}

impl<'source, T: Compile<'source>> Compile<'source> for Spanned<T> {
    type Output = T::Output;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.data.compile(compiler)
    }
}

impl<'source, T: Compile<'source>> Compile<'source> for Option<T> {
    type Output = Option<T::Output>;

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.map(|t| t.compile(compiler))
    }
}

impl<'source, T: Compile<'source>> Compile<'source> for SpannedOption<T> {
    type Output = Option<T::Output>;

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.into_option().map(|t| t.compile(compiler))
    }
}

impl<'source, T: Compile<'source>, R> Compile<'source> for Recoverable<T, R> {
    type Output = Option<T::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.value.compile(compiler)
    }
}

impl<'source, T: Compile<'source>> Compile<'source> for Vec<T> {
    type Output = Vec<T::Output>; // TODO experiment with returning iterators instead of vectors from collection compile implementations

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}

impl<'source, Open, Content: Compile<'source>, Close> Compile<'source>
    for Delimited<Open, Content, Close>
{
    type Output = Option<Content::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.content.compile(compiler)
    }
}

impl<'source, T: Compile<'source>> Compile<'source> for Once<T> {
    type Output = T::Output;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.0.compile(compiler)
    }
}

impl<'source, Item: Compile<'source>, Punctuation> Compile<'source>
    for Punctuated<Item, Punctuation>
{
    type Output = Vec<Item::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}

impl<'source, Item: Compile<'source>, Separator> Compile<'source>
    for SeparatedNonEmpty<Item, Separator>
{
    type Output = Vec<Item::Output>;

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
        self.into_iter().map(|t| t.compile(compiler)).collect()
    }
}

impl<'source> Compile<'source> for ast::Snippet<'source> {
    type Output = ();

    #[inline]
    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_>) -> Self::Output {
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
    pub id_resolver: IdResolver,
    // TODO could be a reference
    pub config: FxHashMap<String, FxHashMap<String, String>>,
    pub lint_data: Option<LintData>,
}

#[derive(Debug)]
pub(crate) enum ExportedValue {
    Function(PreprocessedFunction),
    Literal(Literal),
}

impl<'snippets, 'uberstates> GlobalCompilerData<'snippets, 'uberstates> {
    pub(crate) fn new(
        uber_state_data: &'uberstates UberStateData,
        snippet_access: &'snippets dyn SnippetAccess,
        config: FxHashMap<String, FxHashMap<String, String>>,
        lockfile: Option<PathBuf>,
        lint: bool,
        debug: bool,
    ) -> Self {
        Self {
            output: IntermediateOutput::new(debug),
            uber_state_data,
            snippet_access,
            exported_values: Default::default(),
            id_resolver: IdResolver::new(lockfile),
            config,
            lint_data: lint.then(|| LintData::default()),
        }
    }

    pub(crate) fn finish(
        self,
        errors: &mut FxHashMap<String, (Source, Vec<Error>)>,
    ) -> IntermediateOutput {
        if let Some(lint_data) = self.lint_data {
            lint_data.finish(errors);
        }

        self.output
    }
}

// TODO not sure if all these fields are used anymore since pulling some stuff out into global
pub(crate) struct SnippetCompiler<'source, 'compiler, 'snippets, 'uberstates> {
    pub rng: Pcg64Mcg,
    pub identifier: String, // TODO could be a reference
    pub global: &'compiler mut GlobalCompilerData<'snippets, 'uberstates>,
    pub preprocessed: PreprocessorOutput,
    pub scopes: Scopes<'source>,
    pub errors: Vec<Error>,
}

const SEED_FAILED_MESSAGE: &str = "Failed to seed child RNG";
impl<'source, 'compiler, 'snippets, 'uberstates>
    SnippetCompiler<'source, 'compiler, 'snippets, 'uberstates>
{
    // TODO weird api
    pub(crate) fn compile<R: Rng>(
        ast: ast::Snippet<'source>,
        rng: &mut R,
        identifier: String,
        global: &'compiler mut GlobalCompilerData<'snippets, 'uberstates>,
        preprocessed: PreprocessorOutput,
    ) -> Self {
        let debug = global.output.assets.debug.is_some();

        let mut compiler = Self {
            rng: Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE),
            identifier,
            global,
            preprocessed,
            scopes: Scopes::new(debug),
            errors: vec![],
        };

        ast.compile(&mut compiler);

        if let Some(lint_data) = &mut compiler.global.lint_data {
            lint_data.finish_snippet(&compiler.identifier);
        }

        if let Some(debug) = &mut compiler.global.output.assets.debug {
            // TODO now it's inefficient that we're returning the whole compiler, could save some clones here
            // ... on the other hand, the things we're cloning are probably supposed to be references anyway
            debug.snippets.insert(
                compiler.identifier.clone(),
                SnippetDebugOutput {
                    // TODO debug symbols for scoped variables
                    variables: FxHashMap::default(),
                    function_indices: compiler
                        .preprocessed
                        .functions
                        .iter()
                        .map(|(identifier, function)| (identifier.clone(), function.index))
                        .collect(),
                },
            );
        }

        compiler
    }

    pub(crate) fn define_variable<T>(&mut self, identifier: Identifier<'source>, value: T)
    where
        T: Into<VariableValue>,
    {
        self.scopes.define_variable(identifier.0, value);
    }

    pub(crate) fn resolve_variable(
        &mut self,
        identifier: &Spanned<Identifier<'source>>,
    ) -> Option<&VariableValue> {
        let value = self.scopes.resolve_variable(identifier.data.0);

        if value.is_none() {
            self.errors.push(Error::error(
                "unknown identifier".to_string(),
                identifier.span(),
            ))
        }

        value
    }

    pub(crate) fn resolve_function(
        &mut self,
        identifier: &Spanned<Identifier>,
    ) -> Option<&PreprocessedFunction> {
        let function = self.preprocessed.functions.get(identifier.data.0);

        if function.is_none() {
            self.errors.push(Error::error(
                "unknown function".to_string(),
                identifier.span(),
            ))
        }

        function
    }

    pub(crate) fn check_snippet_included(&mut self, snippet_name: &Spanned<&str>) -> bool {
        let included = self.preprocessed.snippet_included(snippet_name.data);

        if !included {
            self.errors.push(
                Error::error("unknown snippet".to_string(), snippet_name.span.clone())
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
                .push(Error::error("Unknown UberState".to_string(), span.span()))
        }

        ty
    }

    pub(crate) fn infer_type<T: InferType<'source> + Span>(&mut self, t: &T) -> Option<Type> {
        let ty = t.infer_type(self);

        if ty.is_none() {
            self.errors
                .push(Error::error("Cannot infer type".to_string(), t.span()));
        }

        ty
    }

    pub(crate) fn common_type(
        &mut self,
        left: &Expression<'source>,
        right: &Expression<'source>,
    ) -> Option<Type> {
        let left_ty = self.infer_type(left);
        let right_ty = self.infer_type(right);

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
            (Type::WheelItemPosition, Type::Corner) | (Type::Corner, Type::WheelItemPosition) => {
                Some(Type::Corner)
            }
            (_, Type::String) | (Type::String, _) => Some(Type::String),
            _ => {
                self.errors
                    .push(operand_error(left_ty, right_ty, left, right));

                None
            }
        }
    }
}

impl<'snippets, 'uberstates> Compiler<'snippets, 'uberstates> {
    pub fn new<R: Rng, F: SnippetAccess>(
        rng: &mut R,
        snippet_access: &'snippets F,
        // TODO use asset access instead?
        uber_state_data: &'uberstates UberStateData,
        config: FxHashMap<String, FxHashMap<String, String>>,
        lockfile: Option<PathBuf>,
        lint: bool,
        debug: bool,
    ) -> Self {
        Self {
            rng: Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE),
            global: GlobalCompilerData::new(
                uber_state_data,
                snippet_access,
                config,
                lockfile,
                lint,
                debug,
            ),
            compiled_snippets: Default::default(),
            errors: Default::default(),
        }
    }

    pub fn compile_snippet(&mut self, identifier: &str) -> std::result::Result<(), String> {
        match self.compiled_snippets.entry(identifier.to_string()) {
            Entry::Occupied(entry) => match entry.get() {
                CompileState::Started => {
                    return Err(format!("\"{identifier}\" includes itself in a cycle").to_string())
                }
                CompileState::Finished => return Ok(()),
            },
            Entry::Vacant(entry) => {
                entry.insert(CompileState::Started);
            }
        }

        let source = self.global.snippet_access.read_snippet(identifier)?;

        let ast = Snippet::parse(&source.content);
        let mut errors = ast.errors;

        if let Some(ast) = ast.parsed {
            if self.global.lint_data.is_some() {
                lint::lint_ast(&ast, &mut errors);
            }

            let preprocessor = Preprocessor::preprocess(&ast, &self.global.output);
            errors.extend(preprocessor.errors);

            self.global.output.commands.lookup.extend(
                // Fill with placeholders for all the functions, this also ensures a sane result if some of the functions fail to compile
                iter::repeat_with(compile::empty).take(preprocessor.output.functions.len()),
            );

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
                    errors.push(Error::error(
                        format!("Failed to read snippet: {err}"),
                        include.span.clone(),
                    ));
                }
            }

            self.global
                .exported_values
                .insert(identifier.to_string(), Default::default());

            let compiler = SnippetCompiler::compile(
                ast,
                &mut self.rng,
                identifier.to_string(),
                &mut self.global,
                preprocessor.output,
            );

            errors.extend(compiler.errors);
        }

        self.errors.insert(identifier.to_string(), (source, errors));

        *self.compiled_snippets.get_mut(identifier).unwrap() = CompileState::Finished;

        Ok(())
    }

    pub fn finish(mut self) -> CompileResult {
        let output = self.global.finish(&mut self.errors);

        CompileResult {
            output,
            errors: self.errors,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileResult {
    pub output: IntermediateOutput,
    pub errors: FxHashMap<String, (Source, Vec<Error>)>,
}

impl CompileResult {
    pub fn eprint_errors(self) -> Option<IntermediateOutput> {
        let mut stderr = io::stderr().lock();

        let mut error_count = 0;

        for (source, errors) in self.errors.into_values() {
            for error in errors {
                if error.kind.severity() == Severity::Error {
                    error_count += 1;
                }

                error.write_pretty(&source, &mut stderr).unwrap();
            }
        }

        let success = error_count == 0;
        if !success {
            writeln!(
                &mut stderr,
                "Failed to compile Snippets with {error_count} error{}.",
                if error_count == 1 { "" } else { "s" }
            )
            .unwrap();
        }

        success.then_some(self.output)
    }
}
