use crate::syntax::map::UnorderedMap;
use crate::syntax::resolve::Resolution;
use crate::syntax::types::Types;
use crate::syntax::{mangle, Symbol, Ty1, Type};
use proc_macro2::{Ident, Span};
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum ImplKey<'a> {
    RustBox(NamedImplKey<'a>),
    RustVec(NamedImplKey<'a>),
    RustOption(OptionInner<'a>),
    UniquePtr(NamedImplKey<'a>),
    SharedPtr(NamedImplKey<'a>),
    WeakPtr(NamedImplKey<'a>),
    CxxVector(NamedImplKey<'a>),
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum OptionInner<'a> {
    RustBox(NamedImplKey<'a>),
    Ref(NamedImplKey<'a>),
    MutRef(NamedImplKey<'a>),
    RefVec(NamedImplKey<'a>),
    MutRefVec(NamedImplKey<'a>),
}

impl<'a> ImplKey<'a> {
    /// Whether to produce FFI symbols instantiating the given generic type even
    /// when an explicit `impl Foo<T> {}` is not present in the current bridge.
    ///
    /// The main consideration is that the same instantiation must not be
    /// present in two places, which is accomplished using trait impls and the
    /// orphan rule. Every instantiation of a C++ template like `CxxVector<T>`
    /// and Rust generic type like `Vec<T>` requires the implementation of
    /// traits defined by the `cxx` crate for some local type or for a
    /// fundamental type like `Box<LocalType>`.
    pub(crate) fn is_implicit_impl_ok(&self, types: &Types) -> bool {
        match self {
            ImplKey::RustOption(_) => true,
            _ => types.is_local(self.inner()),
        }
    }

    /// Returns the type argument in the generic instantiation described by
    /// `self`. For example, if `self` represents `UniquePtr<u32>` then this
    /// will return `u32`.
    fn inner(&self) -> &'a Type {
        let named_impl_key = match self {
            ImplKey::RustBox(key)
            | ImplKey::RustVec(key)
            | ImplKey::UniquePtr(key)
            | ImplKey::SharedPtr(key)
            | ImplKey::WeakPtr(key)
            | ImplKey::CxxVector(key) => key,
            ImplKey::RustOption(option_inner) => match option_inner {
                OptionInner::RustBox(key)
                | OptionInner::Ref(key)
                | OptionInner::MutRef(key)
                | OptionInner::RefVec(key)
                | OptionInner::MutRefVec(key) => key,
            },
        };
        named_impl_key.inner
    }
}

pub(crate) struct NamedImplKey<'a> {
    #[cfg_attr(not(proc_macro), expect(dead_code))]
    pub begin_span: Span,
    /// Mangled form of the `inner` type.
    pub symbol: Symbol,
    /// Generic type - e.g. `UniquePtr<u8>`.
    #[cfg_attr(proc_macro, expect(dead_code))]
    pub outer: &'a Type,
    /// Generic type argument - e.g. `u8` from `UniquePtr<u8>`.
    pub inner: &'a Type,
    #[cfg_attr(not(proc_macro), expect(dead_code))]
    pub end_span: Span,
}

impl Type {
    pub(crate) fn impl_key(&self, res: &UnorderedMap<&Ident, Resolution>) -> Option<ImplKey> {
        match self {
            Type::RustBox(ty) => Some(ImplKey::RustBox(NamedImplKey::new(self, ty, res)?)),
            Type::RustVec(ty) => Some(ImplKey::RustVec(NamedImplKey::new(self, ty, res)?)),
            Type::RustOption(ty) => match &ty.inner {
                Type::RustBox(_) => {
                    let impl_key = ty.inner.impl_key(res)?;
                    match impl_key {
                        ImplKey::RustBox(named) => {
                            Some(ImplKey::RustOption(OptionInner::RustBox(named)))
                        }
                        _ => unreachable!(),
                    }
                }
                Type::Ref(r) => match &r.inner {
                    Type::RustVec(_) => {
                        if let Some(ImplKey::RustVec(impl_key)) = r.inner.impl_key(res) {
                            if r.mutable {
                                Some(ImplKey::RustOption(OptionInner::MutRefVec(impl_key)))
                            } else {
                                Some(ImplKey::RustOption(OptionInner::RefVec(impl_key)))
                            }
                        } else {
                            None
                        }
                    }
                    Type::Ident(_) => {
                        if r.mutable {
                            Some(ImplKey::RustOption(OptionInner::MutRef(
                                NamedImplKey::new(self, ty, res)?,
                            )))
                        } else {
                            Some(ImplKey::RustOption(OptionInner::Ref(
                                NamedImplKey::new(self, ty, res)?,
                            )))
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            Type::UniquePtr(ty) => Some(ImplKey::UniquePtr(NamedImplKey::new(self, ty, res)?)),
            Type::SharedPtr(ty) => Some(ImplKey::SharedPtr(NamedImplKey::new(self, ty, res)?)),
            Type::WeakPtr(ty) => Some(ImplKey::WeakPtr(NamedImplKey::new(self, ty, res)?)),
            Type::CxxVector(ty) => Some(ImplKey::CxxVector(NamedImplKey::new(self, ty, res)?)),
            _ => None,
        }
    }
}

impl<'a> PartialEq for NamedImplKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.symbol, &other.symbol)
    }
}

impl<'a> Eq for NamedImplKey<'a> {}

impl<'a> Hash for NamedImplKey<'a> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.symbol.hash(hasher);
    }
}

impl<'a> NamedImplKey<'a> {
    fn new(outer: &'a Type, ty1: &'a Ty1, res: &UnorderedMap<&Ident, Resolution>) -> Option<Self> {
        let inner = &ty1.inner;
        Some(NamedImplKey {
            symbol: mangle::typename(inner, res)?,
            begin_span: ty1.name.span(),
            outer,
            inner,
            end_span: ty1.rangle.span,
        })
    }
}
