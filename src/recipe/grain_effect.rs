#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GrainEffect {
    Off,
    WeakSmall,
    StrongSmall,
    WeakLarge,
    StrongLarge,
}
