use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Location {
    Office,
    Home,
    Other(OtherLocation),
}

impl Default for Location {
    fn default() -> Self {
        Location::Office
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct OtherLocation(pub Box<String>);

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Office => f.write_str("Office"),
            Location::Home => f.write_str("Home Office"),
            Location::Other(l) => {
                f.write_str("Other: ")?;
                f.write_str(&l.0)
            }
        }
    }
}
