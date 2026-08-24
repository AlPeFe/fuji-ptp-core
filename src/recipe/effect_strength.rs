#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EffectStrength {
    Off,
    Weak,
    Strong,
}
