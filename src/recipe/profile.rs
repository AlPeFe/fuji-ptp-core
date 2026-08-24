use super::Recipe;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct Profile {
    pub name: String,
    pub recipes: [Recipe; 7],
}

impl Profile {
    pub fn new(name: String, recipes: [Recipe; 7]) -> Self {
        Self { name, recipes }
    }
}
