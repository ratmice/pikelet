use std::fmt;
use std::rc::Rc;

use parse::Declaration as ParseDeclaration;
use parse::Module as ParseModule;
use parse::Term as ParseTerm;
use pretty::{self, ToDoc};
use var::{Debruijn, Name, Named, Var};

/// A module definition
pub struct Module {
    /// The name of the module
    pub name: String,
    /// The definitions contained in the module
    pub definitions: Vec<Definition>,
}

/// Top level definitions
pub struct Definition {
    /// The name of the declaration
    pub name: String,
    /// The body of the definition
    pub term: RcTerm,
    /// An optional type annotation to aid in type inference
    pub ann: Option<RcTerm>,
}

/// Terms
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// A term annotated with a type
    ///
    /// ```text
    /// e : t
    /// ```
    Ann(RcTerm, RcTerm),
    /// Type of types
    Type,
    /// A variable
    Var(Var),
    /// Lambda abstractions
    ///
    /// ```text
    /// \x => t
    /// \x : t => t
    /// ```
    Lam(Named<Option<RcTerm>>, RcTerm),
    /// Dependent function type
    ///
    /// ```text
    /// (x : t) -> t
    /// ```
    Pi(Named<RcTerm>, RcTerm),
    /// Term application
    ///
    /// ```text
    /// f x
    /// ```
    App(RcTerm, RcTerm),
}

impl From<Var> for Term {
    fn from(src: Var) -> Term {
        Term::Var(src)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc(pretty::Context::default())
            .group()
            .render_fmt(f.width().unwrap_or(80), f)
    }
}

/// Normal forms
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The type of types
    Type,
    /// Variables
    Var(Var),
    /// A partially evaluated lambda
    Lam(Named<Option<RcValue>>, RcValue),
    /// A pi type
    Pi(Named<RcValue>, RcValue),
    /// Term application
    App(RcValue, RcValue),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc(pretty::Context::default())
            .group()
            .render_fmt(f.width().unwrap_or(80), f)
    }
}

// Wrapper types

#[macro_export]
macro_rules! make_wrapper {
    ($name:ident, $wrapper:ident, $inner:ty) => {
        #[derive(Clone, PartialEq)]
        pub struct $name {
            pub inner: $wrapper<$inner>,
        }

        impl From<$inner> for $name {
            fn from(src: $inner) -> $name {
                $name {
                    inner: $wrapper::new(src),
                }
            }
        }

        impl $crate::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter) -> $crate::std::fmt::Result {
                $crate::std::fmt::Debug::fmt(&self.inner, f)
            }
        }
    };
}

make_wrapper!(RcTerm, Rc, Term);
make_wrapper!(RcValue, Rc, Value);

/// Types are at the term level, so this is just an alias
pub type Type = Value;

/// Types are at the term level, so this is just an alias
pub type RcType = RcValue;

// Abstraction and instantiation

impl RcTerm {
    pub fn abstract0(&mut self, name: &Name) {
        self.abstract_at(Debruijn::ZERO, name);
    }

    pub fn abstract_at(&mut self, level: Debruijn, name: &Name) {
        match *Rc::make_mut(&mut self.inner) {
            Term::Ann(ref mut expr, ref mut ty) => {
                expr.abstract_at(level, name);
                ty.abstract_at(level, name);
            }
            Term::Lam(Named(_, None), ref mut body) => body.abstract_at(level.succ(), name),
            Term::Lam(Named(_, Some(ref mut ty)), ref mut body) => {
                ty.abstract_at(level, name);
                body.abstract_at(level.succ(), name);
            }
            Term::Pi(Named(_, ref mut ty), ref mut body) => {
                ty.abstract_at(level, name);
                body.abstract_at(level.succ(), name);
            }
            Term::Var(ref mut var) => var.abstract_at(level, name),
            Term::Type => {}
            Term::App(ref mut f, ref mut x) => {
                f.abstract_at(level, name);
                x.abstract_at(level, name);
            }
        }
    }
}

impl RcValue {
    pub fn instantiate0(&self, x: &RcValue) -> RcValue {
        self.instantiate_at(Debruijn::ZERO, &x)
    }

    pub fn instantiate_at(&self, level: Debruijn, x: &RcValue) -> RcValue {
        match *self.inner {
            Value::Type => self.clone(),
            Value::Var(ref var) => match var.instantiate_at(level) {
                true => x.clone(),
                false => self.clone(),
            },
            Value::Lam(Named(ref name, ref param_ty), ref body) => {
                let param_ty = param_ty
                    .as_ref()
                    .map(|param_ty| param_ty.instantiate_at(level, x));
                let body = body.instantiate_at(level.succ(), x);

                Value::Lam(Named(name.clone(), param_ty), body).into()
            }
            Value::Pi(Named(ref name, ref param_ty), ref body) => {
                let param_ty = param_ty.instantiate_at(level, x);
                let body = body.instantiate_at(level.succ(), x);

                Value::Pi(Named(name.clone(), param_ty), body).into()
            }
            Value::App(ref fn_expr, ref arg_expr) => {
                let fn_expr = fn_expr.instantiate_at(level, x);
                let arg = arg_expr.instantiate_at(level, x);

                Value::App(fn_expr, arg).into()
            }
        }
    }
}

// Conversions from the parse tree

fn lam_from_parse(params: &[(String, Option<Box<ParseTerm>>)], body: &ParseTerm) -> RcTerm {
    let mut term = RcTerm::from_parse(body);

    for &(ref name, ref ann) in params.iter().rev() {
        let name = Name::User(name.clone());
        term.abstract0(&name);
        term = match *ann {
            None => Term::Lam(Named(name, None), term).into(),
            Some(ref ann) => {
                let ann = RcTerm::from_parse(ann).into();
                Term::Lam(Named(name, Some(ann)), term).into()
            }
        };
    }

    term
}

impl Module {
    pub fn from_parse(module: &ParseModule) -> Module {
        use std::collections::BTreeMap;
        use std::collections::btree_map::Entry;

        let mut claims = BTreeMap::new();
        let mut definitions = Vec::<Definition>::new();

        for declaration in &module.declarations {
            match *declaration {
                ParseDeclaration::Claim(ref name, ref term) => {
                    match claims.entry(name) {
                        Entry::Occupied(_) => panic!(), // FIXME: Better error
                        Entry::Vacant(mut entry) => {
                            let mut term = RcTerm::from_parse(term);

                            for (level, definition) in definitions.iter().rev().enumerate() {
                                term.abstract_at(
                                    Debruijn(level as u32),
                                    &Name::user(definition.name.clone()),
                                );
                            }

                            entry.insert(term)
                        }
                    };
                }
                ParseDeclaration::Definition(ref name, ref params, ref term) => {
                    let name = name.clone();
                    let mut term = lam_from_parse(params, term);
                    let ann = claims.remove(&name);

                    for (level, definition) in definitions.iter().rev().enumerate() {
                        term.abstract_at(
                            Debruijn(level as u32),
                            &Name::user(definition.name.clone()),
                        );
                    }

                    definitions.push(Definition { name, term, ann });
                }
            }
        }

        // FIXME: Better error
        assert!(claims.is_empty());

        Module {
            name: module.name.clone(),
            definitions,
        }
    }
}

impl RcTerm {
    /// Convert a parsed term into a checkable term
    pub fn from_parse(term: &ParseTerm) -> RcTerm {
        match *term {
            ParseTerm::Var(ref x) => Term::Var(Var::Free(Name::User(x.clone()))).into(),
            ParseTerm::Type => Term::Type.into(),
            ParseTerm::Ann(ref e, ref t) => {
                Term::Ann(RcTerm::from_parse(e).into(), RcTerm::from_parse(t).into()).into()
            }
            ParseTerm::Lam(ref params, ref body) => lam_from_parse(params, body),
            ParseTerm::Pi(ref name, ref ann, ref body) => {
                let name = Name::User(name.clone());
                let mut body = RcTerm::from_parse(body);
                body.abstract0(&name);

                Term::Pi(Named(name, RcTerm::from_parse(ann).into()), body).into()
            }
            ParseTerm::Arrow(ref ann, ref body) => {
                let name = Name::Abstract;
                let mut body = RcTerm::from_parse(body);
                body.abstract0(&name);

                Term::Pi(Named(name, RcTerm::from_parse(ann).into()), body).into()
            }
            ParseTerm::App(ref f, ref arg) => {
                Term::App(RcTerm::from_parse(f), RcTerm::from_parse(arg)).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> RcTerm {
        RcTerm::from_parse(&src.parse().unwrap())
    }

    mod alpha_eq {
        use super::*;

        #[test]
        fn var() {
            assert_eq!(parse(r"x"), parse(r"x"));
        }

        #[test]
        #[should_panic]
        fn var_diff() {
            assert_eq!(parse(r"x"), parse(r"y"));
        }

        #[test]
        fn ty() {
            assert_eq!(parse(r"Type"), parse(r"Type"));
        }

        #[test]
        fn lam() {
            assert_eq!(parse(r"\x : Type => x"), parse(r"\a : Type => a"));
        }

        #[test]
        fn pi() {
            assert_eq!(parse(r"(x : Type) -> x"), parse(r"(a : Type) -> a"));
        }

        #[test]
        fn lam_app() {
            assert_eq!(
                parse(r"\x : Type -> Type => x Type"),
                parse(r"\a : Type -> Type => a Type")
            );
        }

        #[test]
        fn pi_app() {
            assert_eq!(
                parse(r"(x : Type -> Type) -> x Type"),
                parse(r"(a : Type -> Type) -> a Type")
            );
        }

        #[test]
        fn lam_lam_app() {
            assert_eq!(
                parse(r"\x : Type -> Type => \y : Type => x y"),
                parse(r"\a : Type -> Type => \b : Type => a b"),
            );
        }

        #[test]
        fn pi_pi_app() {
            assert_eq!(
                parse(r"(x : Type -> Type) -> (y : Type) -> x y"),
                parse(r"(a : Type -> Type) -> (b : Type) -> a b"),
            );
        }
    }
}
