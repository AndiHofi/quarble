
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Location {
    Office,
    Home,
    Other(OtherLocation)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct OtherLocation(pub Box<String>);