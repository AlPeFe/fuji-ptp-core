#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WhiteBalanceMode {
    Auto,
    Daylight,
    Shade,
    Fluorescent1,
    Fluorescent2,
    Fluorescent3,
    Incandescent,
    Underwater,
    ColorTemperature,
    AmbiencePriority,
}
