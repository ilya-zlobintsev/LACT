#![allow(clippy::module_name_repetitions)]
use lact_schema::{ProcessProfileRule, ProfileRule};
use string_interner::{backend::StringBackend, symbol::SymbolU32, StringInterner};

#[derive(Debug)]
pub enum CompiledRule {
    Process(CompiledProcessRule),
    Gamemode(Option<CompiledProcessRule>),
}

#[derive(Debug)]
pub struct CompiledProcessRule {
    pub name: SymbolU32,
    pub args: Option<SymbolU32>,
}

impl CompiledRule {
    pub fn new(rule: &ProfileRule, interner: &mut StringInterner<StringBackend>) -> Self {
        match rule {
            ProfileRule::Process(rule) => Self::Process(CompiledProcessRule::new(rule, interner)),
            ProfileRule::Gamemode(rule) => Self::Gamemode(
                rule.as_ref()
                    .map(|rule| CompiledProcessRule::new(rule, interner)),
            ),
        }
    }
}

impl CompiledProcessRule {
    pub fn new(rule: &ProcessProfileRule, interner: &mut StringInterner<StringBackend>) -> Self {
        Self {
            name: interner.get_or_intern(&rule.name),
            args: rule.args.as_ref().map(|args| interner.get_or_intern(args)),
        }
    }
}
