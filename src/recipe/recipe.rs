use super::{
    DynamicRange, EffectStrength, FilmSimulation, GrainEffect, WhiteBalance, WhiteBalanceMode,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_recipe() {
        let recipe = Recipe::new("My Recipe".to_string());

        assert_eq!(recipe.name, "My Recipe");
    }
}

impl Recipe {
    pub fn new(name: String) -> Self {
        Self {
            name,
            film_simulation: FilmSimulation::ClassicChrome,
            dynamic_range: DynamicRange::Dr100,
            grain_effect: GrainEffect::Off,
            smooth_skin: EffectStrength::Off,
            color_chrome: EffectStrength::Off,
            color_chrome_fx_blue: EffectStrength::Off,
            white_balance: WhiteBalance {
                mode: WhiteBalanceMode::Auto,
                shift_r: 0,
                shift_b: 0,
                color_temperature: None,
            },
            highlight: 0.0,
            shadow: 0.0,
            color: 0.0,
            sharpness: 0.0,
            noise_reduction: 0,
            clarity: 0.0,
            exposure: 0.0,
            dynamic_range_priority: 0,
            monochrome_wc: 0.0,
            monochrome_mg: 0.0,
        }
    }
}

/// `Recipe` is the Fujifilm custom recipe domain model. Serialization is
/// optional (feature `serde`); the Android bridge uses it as its DTO so the
/// bridge itself stays a thin transport layer.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct Recipe {
    pub name: String,

    pub film_simulation: FilmSimulation,
    pub dynamic_range: DynamicRange,
    pub grain_effect: GrainEffect,

    pub smooth_skin: EffectStrength,
    pub color_chrome: EffectStrength,
    pub color_chrome_fx_blue: EffectStrength,

    pub white_balance: WhiteBalance,

    pub highlight: f32,
    pub shadow: f32,
    pub color: f32,
    pub sharpness: f32,
    pub noise_reduction: i8,
    pub clarity: f32,

    pub exposure: f32,
    pub dynamic_range_priority: i32,

    pub monochrome_wc: f32,
    pub monochrome_mg: f32,
}
