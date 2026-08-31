use crate::lexer::Span;
use super::label::contract::Contract;
use super::label::generics::Generic;
use super::label::implementation::Implementation;
use super::label::is::Is;
use super::label::means::Means;
use super::label::mutable::Mutability;
use super::label::parameters::Parameter;
use super::label::scope::Scope;
use super::label::r#type::TypeDescription;

#[derive(Debug, Clone)]
pub struct SpannedLabel {
    pub label: Label,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Label {
    Is(Is),
    Type(Option<TypeDescription>),
    Mutability(Option<Mutability>),
    Return(Option<TypeDescription>),
    Scope(Option<Scope>),
    Implementation(Option<Implementation>),
    Generic(Option<Generic>),
    Contract(Option<Contract>),
    Parameters(Option<Vec<Parameter>>),
    Means(Option<Means>),
}
