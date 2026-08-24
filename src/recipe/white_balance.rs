use super::WhiteBalanceMode;

pub struct WhiteBalance {
    pub mode: WhiteBalanceMode,
    pub shift_r: i16,
    pub shift_b: i16,
    pub color_temperature: Option<u16>,
}
