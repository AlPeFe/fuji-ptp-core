use super::WhiteBalanceMode;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct WhiteBalance {
    pub mode: WhiteBalanceMode,
    pub shift_r: i16,
    pub shift_b: i16,
    pub color_temperature: Option<u16>,
}
