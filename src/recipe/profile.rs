use super::Recipe;

pub struct Profile {
    pub name: String,
    pub recipes: [Recipe; 7],
}

impl Profile {
    pub fn new(name: String, recipes: [Recipe; 7]) -> Self {
        Self { name, recipes }
    }
}
