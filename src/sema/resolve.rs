use crate::source::SourceSpan;

use super::{BindingId, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct BindingSymbol {
    pub id: BindingId,
    pub name: String,
    pub ty: Type,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SymbolStorage {
    bindings: Vec<BindingSymbol>,
}

impl SymbolStorage {
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn get(&self, id: BindingId) -> Option<&BindingSymbol> {
        self.bindings.get(id.index())
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.bindings.clear();
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
}
