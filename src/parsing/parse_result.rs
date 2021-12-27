use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum ParseResult<T, E> {
    None,
    Valid(T),
    Invalid(E),
    Incomplete,
}


impl<T, E> Default for ParseResult<T, E> {
    fn default() -> Self {
        ParseResult::None
    }
}

impl<T, E> ParseResult<T, E> {
    pub fn and_then<RT>(self, f: impl FnOnce(T) -> ParseResult<RT, E>) -> ParseResult<RT, E> {
        match self {
            ParseResult::Valid(v) => f(v),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
            ParseResult::None => ParseResult::None,
        }
    }

    pub fn map_invalid<RE, F: FnOnce(E) -> RE>(self, f: F) -> ParseResult<T, RE> {
        match self {
            ParseResult::Valid(v) => ParseResult::Valid(v),
            ParseResult::Invalid(e) => ParseResult::Invalid(f(e)),
            ParseResult::Incomplete => ParseResult::Incomplete,
            ParseResult::None => ParseResult::None,
        }
    }

    pub fn map<RT>(self, f: impl FnOnce(T) -> RT) -> ParseResult<RT, E> {
        match self {
            ParseResult::Valid(v) => ParseResult::Valid(f(v)),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
            ParseResult::None => ParseResult::None,
        }
    }

    pub(crate) fn as_ref(&self) -> ParseResult<&T, &E> {
        match self {
            ParseResult::Valid(r) => ParseResult::Valid(r),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
            ParseResult::None => ParseResult::None,
        }
    }


    pub fn or(self, default: T) -> ParseResult<T, E> {
        match self {
            ParseResult::Valid(v) => ParseResult::Valid(v),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
            ParseResult::None => ParseResult::Valid(default),
        }
    }

    pub fn get(self) -> Option<T> {
        match self {
            ParseResult::Valid(v) => Some(v),
            _ => None
        }
    }

    pub fn get_ref(&self) -> Option<&T> {
        self.as_ref().get()
    }
}

impl <T: Clone + Default, E> ParseResult<T, E> {
    pub fn get_with_default(&self, default: T) -> Option<T> {
        match self {
            ParseResult::None => Some(default),
            ParseResult::Valid(t) => Some(t.clone()),
            _ => None
        }
    }
}

impl <T: Default, E> ParseResult<T, E> {
    pub fn or_default(self) -> ParseResult<T, E> {
        self.or(T::default())
    }
}
