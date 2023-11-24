use crate::prelude::*;

#[derive(Clone)]
pub enum TheAtom {
    Value(TheValue),
    Add(TheValue, TheValue),
}

