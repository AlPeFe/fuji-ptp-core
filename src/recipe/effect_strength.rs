#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)
)]
pub enum EffectStrength {
    Off,
    Weak,
    Strong,
}
