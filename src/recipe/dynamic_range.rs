#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynamicRange {
    Dr100,
    Dr200,
    Dr400,
}
