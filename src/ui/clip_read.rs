#[derive(Debug, Eq, PartialEq)]
pub enum ClipRead {
    None,
    Reading,
    DoRead,
    Invalid,
    NoClip,
    Unexpected,
}

impl Default for ClipRead {
    fn default() -> Self {
        ClipRead::None
    }
}

impl ClipRead {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClipRead::None => "",
            ClipRead::Reading => "reading...",
            ClipRead::DoRead => "read clipboard",
            ClipRead::Invalid => "invalid clipboard",
            ClipRead::NoClip => "no clipboard",
            ClipRead::Unexpected => "unexpected",
        }
    }
}
