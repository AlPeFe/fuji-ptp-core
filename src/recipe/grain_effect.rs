#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)
)]
pub enum GrainEffect {
    Off,
    WeakSmall,
    StrongSmall,
    WeakLarge,
    StrongLarge,
}
