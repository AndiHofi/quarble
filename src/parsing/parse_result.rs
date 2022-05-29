use std::fmt::Debug;

#[derive(Clone, Debug, Eq, PartialEq)]
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
            ParseResult::None => ParseResult::None,
            ParseResult::Valid(v) => f(v),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
        }
    }

    pub fn map_invalid<RE, F: FnOnce(E) -> RE>(self, f: F) -> ParseResult<T, RE> {
        match self {
            ParseResult::None => ParseResult::None,
            ParseResult::Valid(v) => ParseResult::Valid(v),
            ParseResult::Invalid(e) => ParseResult::Invalid(f(e)),
            ParseResult::Incomplete => ParseResult::Incomplete,
        }
    }

    pub fn map<RT>(self, f: impl FnOnce(T) -> RT) -> ParseResult<RT, E> {
        match self {
            ParseResult::None => ParseResult::None,
            ParseResult::Valid(v) => ParseResult::Valid(f(v)),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
        }
    }

    pub fn map_err<RE>(self, f: impl FnOnce(E) -> RE) -> ParseResult<T, RE> {
        match self {
            ParseResult::None => ParseResult::None,
            ParseResult::Valid(v) => ParseResult::Valid(v),
            ParseResult::Invalid(e) => ParseResult::Invalid(f(e)),
            ParseResult::Incomplete => ParseResult::Incomplete,
        }
    }

    pub(crate) fn as_ref(&self) -> ParseResult<&T, &E> {
        match self {
            ParseResult::None => ParseResult::None,
            ParseResult::Valid(r) => ParseResult::Valid(r),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
        }
    }

    pub fn or(self, default: T) -> ParseResult<T, E> {
        match self {
            ParseResult::None => ParseResult::Valid(default),
            ParseResult::Valid(v) => ParseResult::Valid(v),
            ParseResult::Invalid(e) => ParseResult::Invalid(e),
            ParseResult::Incomplete => ParseResult::Incomplete,
        }
    }

    pub fn or_else(self, f: impl FnOnce() -> ParseResult<T, E>) -> ParseResult<T, E> {
        match self {
            ParseResult::Incomplete | ParseResult::None => f(),
            i => i,
        }
    }

    pub fn get(self) -> Option<T> {
        match self {
            ParseResult::Valid(v) => Some(v),
            _ => None,
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            ParseResult::None | ParseResult::Invalid(_) | ParseResult::Incomplete => default,
            ParseResult::Valid(t) => t,
        }
    }

    pub fn get_ref(&self) -> Option<&T> {
        self.as_ref().get()
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, ParseResult::None | ParseResult::Incomplete)
    }
}

impl<T, E: Default> ParseResult<T, E> {
    pub fn expect_empty(i: (ParseResult<T, E>, &str)) -> ParseResult<T, E> {
        if i.1.is_empty() {
            i.0
        } else {
            ParseResult::Invalid(E::default())
        }
    }
}

impl<T: Clone, E> ParseResult<T, E> {
    pub fn get_with_default(&self, default: T) -> Option<T> {
        match self {
            ParseResult::None => Some(default),
            ParseResult::Valid(t) => Some(t.clone()),
            _ => None,
        }
    }
}

impl<T: Default, E> ParseResult<T, E> {
    pub fn or_default(self) -> ParseResult<T, E> {
        self.or(T::default())
    }
}

impl<T, E: Default> From<Option<T>> for ParseResult<T, E> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => ParseResult::Valid(v),
            None => ParseResult::Invalid(E::default()),
        }
    }
}

impl<T, E> From<Result<T, E>> for ParseResult<T, E> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(v) => ParseResult::Valid(v),
            Err(e) => ParseResult::Invalid(e),
        }
    }
}
