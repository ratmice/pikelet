//! Delaborate the core language into the surface language.

use crate::core::{Constant, Locals, Term, UniverseLevel, UniverseOffset};
use crate::surface;

pub struct State<'me> {
    // TODO: global names
    // TODO: used names
    names: &'me mut Locals<Option<String>>,
}

impl<'me> State<'me> {
    pub fn new(names: &'me mut Locals<Option<String>>) -> State<'me> {
        State { names }
    }
}

pub fn delaborate_term(state: &mut State<'_>, term: &Term) -> surface::Term<String> {
    match term {
        Term::Universe(UniverseLevel(0)) => surface::Term::Name(0..0, "Type".to_owned()),
        Term::Universe(UniverseLevel(level)) => {
            let universe0 = Box::new(surface::Term::Name(0..0, "Type".to_owned()));
            surface::Term::Lift(0..0, universe0, *level)
        }
        Term::Global(name) => surface::Term::Name(0..0, name.to_owned()),
        Term::Local(index) => {
            // FIXME: unwraps
            surface::Term::Name(0..0, state.names.get(*index).cloned().unwrap().unwrap())
        }
        Term::Constant(constant) => delaborate_constant(constant),
        Term::Sequence(entry_terms) => {
            let core_entry_terms = entry_terms
                .iter()
                .map(|entry_term| delaborate_term(state, entry_term))
                .collect();

            surface::Term::Sequence(0..0, core_entry_terms)
        }
        Term::Ann(term, r#type) => surface::Term::Ann(
            Box::new(delaborate_term(state, term)),
            Box::new(delaborate_term(state, r#type)),
        ),
        Term::RecordType(type_entries) => {
            let core_type_entries = type_entries
                .iter()
                .map(|(entry_name, entry_type)| {
                    let entry_type = delaborate_term(state, entry_type);
                    state.names.push(Some(entry_name.clone()));
                    (0..0, entry_name.clone(), entry_type)
                })
                .collect();

            surface::Term::RecordType(0..0, core_type_entries)
        }
        Term::RecordTerm(term_entries) => {
            let core_term_entries = term_entries
                .iter()
                .map(|(entry_name, entry_term)| {
                    (0..0, entry_name.clone(), delaborate_term(state, entry_term))
                })
                .collect();
            state.names.pop_many(term_entries.len());

            surface::Term::RecordTerm(0..0, core_term_entries)
        }
        Term::RecordElim(head, name) => {
            surface::Term::RecordElim(Box::new(delaborate_term(state, head)), 0..0, name.clone())
        }
        Term::FunctionType(param_name_hint, param_type, body_type) => {
            // FIXME: properly group parameters and deal with name binding!
            let param_type_groups = vec![(
                vec![(
                    0..0,
                    param_name_hint.clone().unwrap_or_else(|| "TODO".to_owned()),
                )],
                delaborate_term(state, param_type),
            )];
            state.names.push(param_name_hint.clone());

            surface::Term::FunctionType(
                0..,
                param_type_groups,
                Box::new(delaborate_term(state, body_type)),
            )
        }
        Term::FunctionTerm(param_name_hint, body) => {
            let mut current_body = body;

            let mut param_names = vec![(0..0, param_name_hint.clone())]; // FIXME: Name avoidance
            state.names.push(Some(param_name_hint.clone()));

            while let Term::FunctionTerm(param_name_hint, body) = current_body.as_ref() {
                param_names.push((0..0, param_name_hint.clone())); // FIXME: Name avoidance
                state.names.push(Some(param_name_hint.clone()));
                current_body = body;
            }

            let body = delaborate_term(state, current_body);
            state.names.pop_many(param_names.len());

            surface::Term::FunctionTerm(0.., param_names, Box::new(body))
        }
        Term::FunctionElim(head, argument) => {
            let mut current_head = head;

            let mut arguments = vec![delaborate_term(state, argument)];
            while let Term::FunctionElim(head, argument) = current_head.as_ref() {
                arguments.push(delaborate_term(state, argument));
                current_head = head;
            }
            arguments.reverse();

            let head = delaborate_term(state, current_head);
            surface::Term::FunctionElim(Box::new(head), arguments)
        }
        Term::Lift(term, UniverseOffset(offset)) => {
            surface::Term::Lift(0..0, Box::new(delaborate_term(state, term)), *offset)
        }
        Term::Error => surface::Term::Error(0..0),
    }
}

pub fn delaborate_constant(constant: &Constant) -> surface::Term<String> {
    use crate::surface::Literal::{Char, Number, String};

    match constant {
        Constant::U8(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::U16(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::U32(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::U64(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::S8(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::S16(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::S32(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::S64(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::F32(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::F64(value) => surface::Term::Literal(0..0, Number(value.to_string())),
        Constant::Char(value) => surface::Term::Literal(0..0, Char(format!("{:?}", value))),
        Constant::String(value) => surface::Term::Literal(0..0, String(format!("{:?}", value))),
    }
}
