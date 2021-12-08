
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Location {
    Office,
    Home,
    Other(OtherLocation)
}

impl Default for Location {
    fn default() -> Self {
        Location::Office
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct OtherLocation(pub Box<String>);