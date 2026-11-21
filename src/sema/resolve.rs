use std::collections::BTreeMap;

use crate::source::SourceSpan;

use super::{BindingId, FunctionId, Type};

#[derive(Debug)]
pub struct Scopes {
    values: Vec<BTreeMap<String, BindingId>>,
    functions: BTreeMap<String, FunctionId>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            values: vec![BTreeMap::new()],
            functions: BTreeMap::new(),
        }
    }

    pub fn lookup_value(&self, name: &str) -> Option<BindingId> {
        self.values
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    pub fn current_value(&self, name: &str) -> Option<BindingId> {
        self.values.last()?.get(name).copied()
    }

    pub fn define_value(&mut self, name: impl Into<String>, id: BindingId) {
        let previous = self
            .values
            .last_mut()
            .expect("Scopes always contain the root scope")
            .insert(name.into(), id);

        debug_assert!(
            previous.is_none(),
            "duplicate binding must be diagnosed before definition"
        );
    }

    pub fn lookup_function(&self, name: &str) -> Option<FunctionId> {
        self.functions.get(name).copied()
    }

    pub fn define_function(&mut self, name: impl Into<String>, id: FunctionId) {
        let previous = self.functions.insert(name.into(), id);

        debug_assert!(
            previous.is_none(),
            "duplicate function must be diagnosed before definition"
        );
    }

    pub fn push_scope(&mut self) {
        self.values.push(BTreeMap::new());
    }

    pub fn pop_scope(&mut self) {
        assert!(self.values.len() > 1, "the root scope cannot be popped");
        self.values.pop();
    }
}

impl Default for Scopes {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BindingSymbol {
    pub id: BindingId,
    pub name: String,
    pub ty: Type,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionSymbol {
    pub id: FunctionId,
    pub name: String,
    pub parameter_types: Vec<Type>,
    pub parameter_spans: Vec<SourceSpan>,
    pub result: Type,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SymbolStorage {
    bindings: Vec<BindingSymbol>,
    functions: Vec<FunctionSymbol>,
}

impl SymbolStorage {
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn get(&self, id: BindingId) -> Option<&BindingSymbol> {
        self.bindings.get(id.index())
    }

    pub fn get_function(&self, id: FunctionId) -> Option<&FunctionSymbol> {
        self.functions.get(id.index())
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn function_len(&self) -> usize {
        self.functions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.functions.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.bindings.clear();
        self.functions.clear();
    }

    pub fn define(&mut self, name: impl Into<String>, ty: Type, span: SourceSpan) -> BindingId {
        let id = BindingId::new(self.bindings.len());
        self.bindings.push(BindingSymbol {
            id,
            name: name.into(),
            ty,
            span,
        });
        id
    }

    pub fn define_function(
        &mut self,
        name: impl Into<String>,
        parameter_types: Vec<Type>,
        parameter_spans: Vec<SourceSpan>,
        result: Type,
        span: SourceSpan,
    ) -> FunctionId {
        assert_eq!(
            parameter_types.len(),
            parameter_spans.len(),
            "function signature metadata must have equal lengths"
        );

        let id = FunctionId::new(self.functions.len());
        self.functions.push(FunctionSymbol {
            id,
            name: name.into(),
            parameter_types,
            parameter_spans,
            result,
            span,
        });
        id
    }
}
