use std::{
    borrow::{Borrow, Cow},
    hash::Hash,
};

use rustc_hash::FxHashMap;
use wotw_seedgen_parse::Identifier;

use crate::seed_language::{
    compile::FunctionSignature,
    output::{Literal, Reference, VariableValue},
    types::Type,
};

pub struct Scopes<'source> {
    stack: Vec<Scope<'source>>,
    // debug: Option<ScopeDebugSymbols<'source>>,
}

impl<'source> Scopes<'source> {
    pub fn new(_debug: bool) -> Self {
        let mut scopes = Self {
            stack: Vec::with_capacity(2),
            // debug: debug.then_some(t),
        };

        scopes.push();

        scopes
    }

    pub fn push(&mut self) {
        self.stack.push(Scope::default());
    }

    pub fn push_function(&mut self, signature: &FunctionSignature) {
        let mut scope = Scope::default();

        let mut booleans = 0..;
        let mut integers = 0..;
        let mut floats = 0..;
        let mut strings = 0..;

        for arg in &signature.args {
            match arg.ty {
                Type::Boolean => scope.define_variable(
                    arg.identifier.clone(),
                    Reference::BooleanStack(booleans.next().unwrap()),
                ),
                Type::Integer => scope.define_variable(
                    arg.identifier.clone(),
                    Reference::IntegerStack(integers.next().unwrap()),
                ),
                Type::Float => scope.define_variable(
                    arg.identifier.clone(),
                    Reference::FloatStack(floats.next().unwrap()),
                ),
                Type::String => scope.define_variable(
                    arg.identifier.clone(),
                    Reference::StringStack(strings.next().unwrap()),
                ),
                _ => {}
            }
        }

        self.stack.push(scope);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn define_variable<I, V>(&mut self, identifier: I, value: V)
    where
        I: Into<Cow<'source, str>>,
        V: Into<VariableValue>,
    {
        self.current()
            .variables
            .insert(identifier.into(), value.into());
    }

    pub fn define_random_pool(&mut self, identifier: Identifier<'source>, value: Vec<Literal>) {
        self.current().random_pools.insert(identifier, value);
    }

    fn current(&mut self) -> &mut Scope<'source> {
        self.stack.last_mut().unwrap()
    }

    pub fn resolve_variable<Q>(&self, identifier: &Q) -> Option<&VariableValue>
    where
        Cow<'source, str>: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.resolve(|scope| scope.variables.get(identifier))
    }

    pub fn resolve_random_pool(
        &mut self,
        identifier: Identifier<'source>,
    ) -> Option<&mut Vec<Literal>> {
        self.resolve_mut(|scope| scope.random_pools.get_mut(&identifier))
    }

    fn resolve<V, F>(&self, f: F) -> Option<&V>
    where
        F: for<'a> FnMut(&'a Scope<'source>) -> Option<&'a V>,
    {
        self.stack.iter().rev().find_map(f)
    }

    fn resolve_mut<V, F>(&mut self, f: F) -> Option<&mut V>
    where
        F: for<'a> FnMut(&'a mut Scope<'source>) -> Option<&'a mut V>,
    {
        self.stack.iter_mut().rev().find_map(f)
    }
}

#[derive(Default)]
struct Scope<'source> {
    variables: FxHashMap<Cow<'source, str>, VariableValue>,
    random_pools: FxHashMap<Identifier<'source>, Vec<Literal>>,
}

impl<'source> Scope<'source> {
    fn define_variable<T>(&mut self, identifier: Cow<'source, str>, value: T)
    where
        T: Into<VariableValue>,
    {
        self.variables.insert(identifier, value.into());
    }
}

// pub struct ScopeDebugSymbols<'source> {
//     scope: Scope<'source>,
//     nested: FxHashMap<Identifier<'source>, ScopeDebugSymbols<'source>>,
// }
