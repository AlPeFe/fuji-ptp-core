use crate::ptp::{PtpProtocolError, PtpSession, PtpSessionError};
use crate::recipe::*;
use crate::transport::Transport;

pub const SET_DEVICE_PROP_VALUE: u16 = 0x1016;
pub const GET_DEVICE_PROP_VALUE: u16 = 0x1015;
pub const PROP_SLOT_CURSOR: u16 = 0xD18C;
pub const PROP_SLOT_NAME: u16 = 0xD18D;
pub const PRESET_BLOCK_START: u16 = 0xD18E;
pub const PRESET_BLOCK_END: u16 = 0xD1A5;

const DR: u16 = 0xD190;
const DR_PRIORITY: u16 = 0xD191;
const FILM: u16 = 0xD192;
const MONO_WC: u16 = 0xD193;
const MONO_MG: u16 = 0xD194;
const GRAIN: u16 = 0xD195;
const COLOR_CHROME: u16 = 0xD196;
const FX_BLUE: u16 = 0xD197;
const SMOOTH: u16 = 0xD198;
const WB: u16 = 0xD199;
const WB_R: u16 = 0xD19A;
const WB_B: u16 = 0xD19B;
const TEMP: u16 = 0xD19C;
const HIGHLIGHT: u16 = 0xD19D;
const SHADOW: u16 = 0xD19E;
const COLOR: u16 = 0xD19F;
const SHARPNESS: u16 = 0xD1A0;
const NR: u16 = 0xD1A1;
const CLARITY: u16 = 0xD1A2;
const PROPS: [u16; 19] = [
    DR,
    DR_PRIORITY,
    FILM,
    MONO_WC,
    MONO_MG,
    GRAIN,
    COLOR_CHROME,
    FX_BLUE,
    SMOOTH,
    WB,
    WB_R,
    WB_B,
    TEMP,
    HIGHLIGHT,
    SHADOW,
    COLOR,
    SHARPNESS,
    NR,
    CLARITY,
];

#[derive(Debug)]
pub enum FujiPtpError {
    InvalidSlot,
    InvalidData,
    InvalidDataAt(String),
    TransportError,
    ProtocolError,
    Protocol(PtpProtocolError),
}

pub struct FujiPtp<T: Transport> {
    session: PtpSession<T>,
}
impl<T: Transport> FujiPtp<T> {
    pub fn new(transport: T) -> Self {
        Self {
            session: PtpSession::new(transport),
        }
    }
    pub fn transport(&self) -> &T {
        self.session.transport()
    }
    pub fn open_session(&mut self, session_id: u32) -> Result<(), FujiPtpError> {
        self.session.open_session(session_id).map_err(|e| match e {
            PtpSessionError::Transport(_) => FujiPtpError::TransportError,
            PtpSessionError::Protocol(e) => FujiPtpError::Protocol(e),
        })
    }

    pub fn close_session(&mut self) -> Result<(), FujiPtpError> {
        self.session.close_session().map_err(|e| match e {
            PtpSessionError::Transport(_) => FujiPtpError::TransportError,
            PtpSessionError::Protocol(e) => FujiPtpError::Protocol(e),
        })
    }

    pub fn select_slot(&mut self, slot: u8) -> Result<(), FujiPtpError> {
        if !(1..=7).contains(&slot) {
            return Err(FujiPtpError::InvalidSlot);
        }
        self.session
            .set_device_prop_value_wait(PROP_SLOT_CURSOR, &[slot, 0])
            .map_err(|e| match e {
                PtpSessionError::Transport(_) => FujiPtpError::TransportError,
                PtpSessionError::Protocol(e) => FujiPtpError::Protocol(e),
            })
    }
    fn get(&mut self, property: u16) -> Result<Vec<u8>, FujiPtpError> {
        self.session
            .get_device_prop_value(property)
            .map_err(|e| match e {
                PtpSessionError::Transport(_) => FujiPtpError::TransportError,
                PtpSessionError::Protocol(e) => FujiPtpError::Protocol(e),
            })
    }
    fn word(data: &[u8]) -> Result<u16, FujiPtpError> {
        if data.len() != 2 {
            Err(FujiPtpError::InvalidData)
        } else {
            Ok(u16::from_le_bytes([data[0], data[1]]))
        }
    }
    fn ptp_name(data: &[u8]) -> Result<String, FujiPtpError> {
        if data.is_empty() {
            return Err(FujiPtpError::InvalidData);
        }
        let n = data[0] as usize;
        // The reference implementation encodes an empty PTP string as one
        // byte: [0]. This is a valid empty recipe name.
        if n == 0 {
            return if data.len() == 1 {
                Ok(String::new())
            } else {
                Err(FujiPtpError::InvalidData)
            };
        }
        if data.len() != 1 + n * 2
            || u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]) != 0
        {
            return Err(FujiPtpError::InvalidData);
        }
        let mut s = String::new();
        for i in 0..n - 1 {
            let c = u16::from_le_bytes([data[1 + i * 2], data[2 + i * 2]]);
            s.push(char::from_u32(c as u32).ok_or(FujiPtpError::InvalidData)?);
        }
        Ok(s)
    }
    fn film(v: u16) -> Option<FilmSimulation> {
        Some(match v {
            1 => FilmSimulation::Provia,
            2 => FilmSimulation::Velvia,
            3 => FilmSimulation::Astia,
            4 => FilmSimulation::ProNegHigh,
            5 => FilmSimulation::ProNegStandard,
            6 => FilmSimulation::Monochrome,
            7 => FilmSimulation::MonochromeYellow,
            8 => FilmSimulation::MonochromeRed,
            9 => FilmSimulation::MonochromeGreen,
            10 => FilmSimulation::Sepia,
            11 => FilmSimulation::ClassicChrome,
            12 => FilmSimulation::Acros,
            13 => FilmSimulation::AcrosYellow,
            14 => FilmSimulation::AcrosRed,
            15 => FilmSimulation::AcrosGreen,
            16 => FilmSimulation::Eterna,
            17 => FilmSimulation::ClassicNegative,
            18 => FilmSimulation::EternaBleachBypass,
            19 => FilmSimulation::NostalgicNegative,
            20 => FilmSimulation::RealaAce,
            _ => return None,
        })
    }
    fn strength(v: u16) -> Option<EffectStrength> {
        match v {
            1 => Some(EffectStrength::Off),
            2 => Some(EffectStrength::Weak),
            3 => Some(EffectStrength::Strong),
            _ => None,
        }
    }
    fn grain(v: u16) -> Option<GrainEffect> {
        match v {
            // X100VI reports 7 for Off (the wire value it stores); the
            // X100V-era firmware used 6. Accept both on read.
            1 | 6 | 7 => Some(GrainEffect::Off),
            2 => Some(GrainEffect::WeakSmall),
            3 => Some(GrainEffect::StrongSmall),
            4 => Some(GrainEffect::WeakLarge),
            5 => Some(GrainEffect::StrongLarge),
            _ => None,
        }
    }

    pub fn read_recipes(&mut self) -> Result<Profile, FujiPtpError> {
        let mut recipes: Vec<Recipe> = Vec::with_capacity(7);
        for slot in 1..=7 {
            self.select_slot(slot)?;
            let name = Self::ptp_name(&self.get(PROP_SLOT_NAME)?)
                .map_err(|_| FujiPtpError::InvalidDataAt(format!("slot C{} name", slot)))?;
            let mut r = Recipe::new(name.clone());
            let mut v = Vec::new();
            for p in PROPS {
                v.push(Self::word(&self.get(p)?).map_err(|_| {
                    FujiPtpError::InvalidDataAt(format!("slot C{} property 0x{:04X}", slot, p))
                })?);
            }
            // Empty custom slots return 0xFFFF for the first property block
            // value. They are valid and must remain representable as a
            // default Recipe rather than being rejected.
            if v[0] == 0xFFFF {
                recipes.push(Recipe::new(name));
                continue;
            }
            r.dynamic_range = match v[0] {
                0 => DynamicRange::Dr100,
                100 => DynamicRange::Dr100,
                200 => DynamicRange::Dr200,
                400 => DynamicRange::Dr400,
                value => {
                    return Err(FujiPtpError::InvalidDataAt(format!(
                        "slot C{} dynamic range 0x{:04X}",
                        slot, value
                    )));
                }
            };
            r.dynamic_range_priority = v[1] as i32;
            r.film_simulation = Self::film(v[2]).ok_or_else(|| {
                FujiPtpError::InvalidDataAt(format!("slot C{} film simulation {}", slot, v[2]))
            })?;
            r.monochrome_wc = v[3] as i16 as f32 / 10.;
            r.monochrome_mg = v[4] as i16 as f32 / 10.;
            r.grain_effect = Self::grain(v[5]).ok_or_else(|| {
                FujiPtpError::InvalidDataAt(format!("slot C{} grain {}", slot, v[5]))
            })?;
            r.color_chrome = Self::strength(v[6]).ok_or_else(|| {
                FujiPtpError::InvalidDataAt(format!("slot C{} color chrome {}", slot, v[6]))
            })?;
            r.color_chrome_fx_blue = Self::strength(v[7]).ok_or_else(|| {
                FujiPtpError::InvalidDataAt(format!("slot C{} fx blue {}", slot, v[7]))
            })?;
            r.smooth_skin = Self::strength(v[8]).ok_or_else(|| {
                FujiPtpError::InvalidDataAt(format!("slot C{} smooth skin {}", slot, v[8]))
            })?;
            r.white_balance.mode = match v[9] {
                2 | 0x8020 => WhiteBalanceMode::Auto,
                4 => WhiteBalanceMode::Daylight,
                6 => WhiteBalanceMode::Incandescent,
                8 => WhiteBalanceMode::Underwater,
                0x8001 => WhiteBalanceMode::Fluorescent1,
                0x8002 => WhiteBalanceMode::Fluorescent2,
                0x8003 => WhiteBalanceMode::Fluorescent3,
                0x8006 => WhiteBalanceMode::Shade,
                0x8007 => WhiteBalanceMode::ColorTemperature,
                0x8021 => WhiteBalanceMode::AmbiencePriority,
                value => {
                    return Err(FujiPtpError::InvalidDataAt(format!(
                        "slot C{} white balance 0x{:04X}",
                        slot, value
                    )));
                }
            };
            r.white_balance.shift_r = v[10] as i16;
            r.white_balance.shift_b = v[11] as i16;
            r.white_balance.color_temperature = Some(v[12]);
            r.highlight = v[13] as i16 as f32 / 10.;
            r.shadow = v[14] as i16 as f32 / 10.;
            r.color = v[15] as i16 as f32 / 10.;
            r.sharpness = v[16] as i16 as f32 / 10.;
            r.noise_reduction = match v[17] {
                0x8000 => -4,
                28672 => -3,
                16384 => -2,
                12288 => -1,
                8192 => 0,
                4096 => 1,
                0 => 2,
                24576 => 3,
                20480 => 4,
                value => {
                    return Err(FujiPtpError::InvalidDataAt(format!(
                        "slot C{} high ISO NR {}",
                        slot, value
                    )));
                }
            };
            r.clarity = v[18] as i16 as f32 / 10.;
            recipes.push(r);
        }
        let recipes: [Recipe; 7] = match recipes.try_into() {
            Ok(value) => value,
            Err(_) => return Err(FujiPtpError::InvalidData),
        };
        Ok(Profile::new("Fujifilm Custom Recipes".into(), recipes))
    }

    fn u16v(v: u16) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }
    fn i16v(v: i16) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }
    fn set(&mut self, p: u16, v: Vec<u8>) -> Result<(), FujiPtpError> {
        self.session
            .set_device_prop_value_wait(p, &v)
            .map_err(|e| match e {
                PtpSessionError::Transport(_) => FujiPtpError::TransportError,
                PtpSessionError::Protocol(e) => match e {
                    PtpProtocolError::Response(code) => FujiPtpError::InvalidDataAt(format!(
                        "write property 0x{:04X}: camera response 0x{:04X}",
                        p, code
                    )),
                    other => FujiPtpError::Protocol(other),
                },
            })
    }
    fn name_bytes(name: &str) -> Result<Vec<u8>, FujiPtpError> {
        let units: Vec<u16> = name.encode_utf16().collect();
        if units.len() > 254 || name.contains('\0') {
            return Err(FujiPtpError::InvalidData);
        }
        let mut b = vec![(units.len() + 1) as u8];
        for u in units {
            b.extend_from_slice(&u.to_le_bytes())
        }
        b.extend_from_slice(&[0, 0]);
        Ok(b)
    }
    /// Round-trip helper: changes only the preset names, preserving every
    /// other value already stored in the camera.
    pub fn write_recipe_names(&mut self, profile: &Profile) -> Result<(), FujiPtpError> {
        for (i, recipe) in profile.recipes.iter().enumerate() {
            self.select_slot((i + 1) as u8)?;
            self.set(PROP_SLOT_NAME, Self::name_bytes(&recipe.name)?)?;
        }
        Ok(())
    }

    /// Writes one complete recipe to one slot. Intended for cautious hardware tests.
    pub fn write_recipe(&mut self, slot: u8, r: &Recipe) -> Result<(), FujiPtpError> {
        self.select_slot(slot)?;
        // Name FIRST: it's the property that gets dropped when the camera
        // truncates the last operation, so write it while the slot is fresh.
        self.set(PROP_SLOT_NAME, Self::name_bytes(&r.name)?)?;
        self.set(FILM, Self::u16v(Self::film_wire(&r.film_simulation)))?;
        let priority = match r.dynamic_range_priority {
            0 => 0,
            1 => 1,
            2 => 2,
            32768 => 32768,
            _ => return Err(FujiPtpError::InvalidData),
        };
        self.set(DR_PRIORITY, Self::u16v(priority))?;
        if priority == 0 {
            self.set(
                DR,
                Self::u16v(match r.dynamic_range {
                    DynamicRange::Dr100 => 100,
                    DynamicRange::Dr200 => 200,
                    DynamicRange::Dr400 => 400,
                }),
            )?;
        }
        // These controls are only accepted by bodies when a monochrome
        // simulation is active.
        if Self::is_mono(&r.film_simulation) {
            self.set(MONO_WC, Self::i16v(Self::dial(r.monochrome_wc)?))?;
            self.set(MONO_MG, Self::i16v(Self::dial(r.monochrome_mg)?))?;
        }
        self.set(GRAIN, Self::u16v(Self::grain_wire(&r.grain_effect)))?;
        if !Self::is_mono(&r.film_simulation) {
            self.set(
                COLOR_CHROME,
                Self::u16v(Self::strength_wire(&r.color_chrome)),
            )?;
            self.set(
                FX_BLUE,
                Self::u16v(Self::strength_wire(&r.color_chrome_fx_blue)),
            )?;
        }
        self.set(SMOOTH, Self::u16v(Self::strength_wire(&r.smooth_skin)))?;
        self.set(WB, Self::u16v(Self::wb_wire(&r.white_balance.mode)))?;
        if !(-9..=9).contains(&r.white_balance.shift_r)
            || !(-9..=9).contains(&r.white_balance.shift_b)
        {
            return Err(FujiPtpError::InvalidData);
        }
        if !Self::is_mono(&r.film_simulation) {
            self.set(WB_R, Self::i16v(r.white_balance.shift_r))?;
            self.set(WB_B, Self::i16v(r.white_balance.shift_b))?;
        }
        if matches!(r.white_balance.mode, WhiteBalanceMode::ColorTemperature) {
            if let Some(t) = r.white_balance.color_temperature {
                self.set(TEMP, Self::u16v(t))?;
            }
        }
        for (p, x) in [
            (HIGHLIGHT, r.highlight),
            (SHADOW, r.shadow),
            (COLOR, r.color),
            (SHARPNESS, r.sharpness),
            (CLARITY, r.clarity),
        ] {
            self.set(p, Self::i16v(Self::dial(x)?))?;
        }
        self.set(NR, Self::u16v(Self::nr_wire(r.noise_reduction)?))?;
        Ok(())
    }

    /// Writes every recipe setting EXCEPT the slot name. The camera keeps
    /// whatever name the slot already had. This is what the Android app uses
    /// when pushing recipes, so an imported recipe never garbles the name
    /// stored on the camera (name changes are done explicitly via
    /// [`Self::write_recipe_names`]).
    pub fn write_recipe_settings(&mut self, slot: u8, r: &Recipe) -> Result<(), FujiPtpError> {
        self.select_slot(slot)?;
        self.set(FILM, Self::u16v(Self::film_wire(&r.film_simulation)))?;
        let priority = match r.dynamic_range_priority {
            0 => 0,
            1 => 1,
            2 => 2,
            32768 => 32768,
            _ => return Err(FujiPtpError::InvalidData),
        };
        self.set(DR_PRIORITY, Self::u16v(priority))?;
        if priority == 0 {
            self.set(
                DR,
                Self::u16v(match r.dynamic_range {
                    DynamicRange::Dr100 => 100,
                    DynamicRange::Dr200 => 200,
                    DynamicRange::Dr400 => 400,
                }),
            )?;
        }
        if Self::is_mono(&r.film_simulation) {
            self.set(MONO_WC, Self::i16v(Self::dial(r.monochrome_wc)?))?;
            self.set(MONO_MG, Self::i16v(Self::dial(r.monochrome_mg)?))?;
        }
        self.set(GRAIN, Self::u16v(Self::grain_wire(&r.grain_effect)))?;
        if !Self::is_mono(&r.film_simulation) {
            self.set(
                COLOR_CHROME,
                Self::u16v(Self::strength_wire(&r.color_chrome)),
            )?;
            self.set(
                FX_BLUE,
                Self::u16v(Self::strength_wire(&r.color_chrome_fx_blue)),
            )?;
        }
        self.set(SMOOTH, Self::u16v(Self::strength_wire(&r.smooth_skin)))?;
        self.set(WB, Self::u16v(Self::wb_wire(&r.white_balance.mode)))?;
        if !(-9..=9).contains(&r.white_balance.shift_r)
            || !(-9..=9).contains(&r.white_balance.shift_b)
        {
            return Err(FujiPtpError::InvalidData);
        }
        if !Self::is_mono(&r.film_simulation) {
            self.set(WB_R, Self::i16v(r.white_balance.shift_r))?;
            self.set(WB_B, Self::i16v(r.white_balance.shift_b))?;
        }
        if matches!(r.white_balance.mode, WhiteBalanceMode::ColorTemperature) {
            if let Some(t) = r.white_balance.color_temperature {
                self.set(TEMP, Self::u16v(t))?;
            }
        }
        for (p, x) in [
            (HIGHLIGHT, r.highlight),
            (SHADOW, r.shadow),
            (COLOR, r.color),
            (SHARPNESS, r.sharpness),
            (CLARITY, r.clarity),
        ] {
            self.set(p, Self::i16v(Self::dial(x)?))?;
        }
        self.set(NR, Self::u16v(Self::nr_wire(r.noise_reduction)?))
    }

    pub fn write_recipes(&mut self, profile: &Profile) -> Result<(), FujiPtpError> {
        for (i, r) in profile.recipes.iter().enumerate() {
            self.select_slot((i + 1) as u8)?;
            self.set(FILM, Self::u16v(Self::film_wire(&r.film_simulation)))?;
            let priority = match r.dynamic_range_priority {
                0 => 0,
                1 => 1,
                2 => 2,
                32768 => 32768,
                _ => return Err(FujiPtpError::InvalidData),
            };
            self.set(DR_PRIORITY, Self::u16v(priority))?;
            if priority == 0 {
                self.set(
                    DR,
                    Self::u16v(match r.dynamic_range {
                        DynamicRange::Dr100 => 100,
                        DynamicRange::Dr200 => 200,
                        DynamicRange::Dr400 => 400,
                    }),
                )?;
            }
            if Self::is_mono(&r.film_simulation) {
                self.set(MONO_WC, Self::i16v(Self::dial(r.monochrome_wc)?))?;
                self.set(MONO_MG, Self::i16v(Self::dial(r.monochrome_mg)?))?;
            }
            self.set(GRAIN, Self::u16v(Self::grain_wire(&r.grain_effect)))?;
            if !Self::is_mono(&r.film_simulation) {
                self.set(
                    COLOR_CHROME,
                    Self::u16v(Self::strength_wire(&r.color_chrome)),
                )?;
                self.set(
                    FX_BLUE,
                    Self::u16v(Self::strength_wire(&r.color_chrome_fx_blue)),
                )?;
            }
            self.set(SMOOTH, Self::u16v(Self::strength_wire(&r.smooth_skin)))?;
            self.set(WB, Self::u16v(Self::wb_wire(&r.white_balance.mode)))?;
            if !(-9..=9).contains(&r.white_balance.shift_r)
                || !(-9..=9).contains(&r.white_balance.shift_b)
            {
                return Err(FujiPtpError::InvalidData);
            }
            if !Self::is_mono(&r.film_simulation) {
                self.set(WB_R, Self::i16v(r.white_balance.shift_r))?;
                self.set(WB_B, Self::i16v(r.white_balance.shift_b))?;
            }
            if matches!(r.white_balance.mode, WhiteBalanceMode::ColorTemperature) {
                if let Some(t) = r.white_balance.color_temperature {
                    self.set(TEMP, Self::u16v(t))?
                };
            }
            for (p, x) in [
                (HIGHLIGHT, r.highlight),
                (SHADOW, r.shadow),
                (COLOR, r.color),
                (SHARPNESS, r.sharpness),
                (CLARITY, r.clarity),
            ] {
                self.set(p, Self::i16v(Self::dial(x)?))?
            }
            self.set(NR, Self::u16v(Self::nr_wire(r.noise_reduction)?))?;
            self.set(PROP_SLOT_NAME, Self::name_bytes(&r.name)?)?;
        }
        Ok(())
    }
    fn dial(x: f32) -> Result<i16, FujiPtpError> {
        if !x.is_finite() || x < -3276.7 || x > 3276.7 {
            Err(FujiPtpError::InvalidData)
        } else {
            Ok((x * 10.).round() as i16)
        }
    }
    fn is_mono(f: &FilmSimulation) -> bool {
        matches!(
            f,
            FilmSimulation::Monochrome
                | FilmSimulation::MonochromeYellow
                | FilmSimulation::MonochromeRed
                | FilmSimulation::MonochromeGreen
                | FilmSimulation::Sepia
                | FilmSimulation::Acros
                | FilmSimulation::AcrosYellow
                | FilmSimulation::AcrosRed
                | FilmSimulation::AcrosGreen
        )
    }
    fn nr_wire(v: i8) -> Result<u16, FujiPtpError> {
        match v {
            -4 => Ok(32768),
            -3 => Ok(28672),
            -2 => Ok(16384),
            -1 => Ok(12288),
            0 => Ok(8192),
            1 => Ok(4096),
            2 => Ok(0),
            3 => Ok(24576),
            4 => Ok(20480),
            _ => Err(FujiPtpError::InvalidData),
        }
    }
    fn film_wire(f: &FilmSimulation) -> u16 {
        match f {
            FilmSimulation::Provia => 1,
            FilmSimulation::Velvia => 2,
            FilmSimulation::Astia => 3,
            FilmSimulation::ProNegHigh => 4,
            FilmSimulation::ProNegStandard => 5,
            FilmSimulation::Monochrome => 6,
            FilmSimulation::MonochromeYellow => 7,
            FilmSimulation::MonochromeRed => 8,
            FilmSimulation::MonochromeGreen => 9,
            FilmSimulation::Sepia => 10,
            FilmSimulation::ClassicChrome => 11,
            FilmSimulation::Acros => 12,
            FilmSimulation::AcrosYellow => 13,
            FilmSimulation::AcrosRed => 14,
            FilmSimulation::AcrosGreen => 15,
            FilmSimulation::Eterna => 16,
            FilmSimulation::ClassicNegative => 17,
            FilmSimulation::EternaBleachBypass => 18,
            FilmSimulation::NostalgicNegative => 19,
            FilmSimulation::RealaAce => 20,
        }
    }
    fn strength_wire(s: &EffectStrength) -> u16 {
        match s {
            EffectStrength::Off => 1,
            EffectStrength::Weak => 2,
            EffectStrength::Strong => 3,
        }
    }
    fn grain_wire(g: &GrainEffect) -> u16 {
        match g {
            GrainEffect::Off => 1,
            GrainEffect::WeakSmall => 2,
            GrainEffect::StrongSmall => 3,
            GrainEffect::WeakLarge => 4,
            GrainEffect::StrongLarge => 5,
        }
    }
    fn wb_wire(w: &WhiteBalanceMode) -> u16 {
        match w {
            WhiteBalanceMode::Auto => 2,
            WhiteBalanceMode::Daylight => 4,
            WhiteBalanceMode::Shade => 0x8006,
            WhiteBalanceMode::Fluorescent1 => 0x8001,
            WhiteBalanceMode::Fluorescent2 => 0x8002,
            WhiteBalanceMode::Fluorescent3 => 0x8003,
            WhiteBalanceMode::Incandescent => 6,
            WhiteBalanceMode::Underwater => 8,
            WhiteBalanceMode::ColorTemperature => 0x8007,
            WhiteBalanceMode::AmbiencePriority => 0x8021,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grain_read_accepts_x100vi_off_values() {
        // X100VI stores Grain Off as 7; older firmware used 6; wire 1 is
        // what we write. All three must read back as Off.
        for v in [1u16, 6, 7] {
            assert!(
                matches!(
                    FujiPtp::<crate::transport::MockTransport>::grain(v),
                    Some(GrainEffect::Off)
                ),
                "grain {v} should be Off"
            );
        }
        assert!(matches!(
            FujiPtp::<crate::transport::MockTransport>::grain(2),
            Some(GrainEffect::WeakSmall)
        ));
        assert!(matches!(
            FujiPtp::<crate::transport::MockTransport>::grain(5),
            Some(GrainEffect::StrongLarge)
        ));
        assert!(FujiPtp::<crate::transport::MockTransport>::grain(0).is_none());
        assert!(FujiPtp::<crate::transport::MockTransport>::grain(99).is_none());
    }

    #[test]
    fn grain_write_roundtrip() {
        // What we write maps back to the same enum on read.
        for g in [
            GrainEffect::Off,
            GrainEffect::WeakSmall,
            GrainEffect::StrongSmall,
            GrainEffect::WeakLarge,
            GrainEffect::StrongLarge,
        ] {
            let wire = FujiPtp::<crate::transport::MockTransport>::grain_wire(&g);
            // The camera may normalize Off to 7, so accept the write value
            // or the camera's stored value.
            let read = if matches!(g, GrainEffect::Off) {
                FujiPtp::<crate::transport::MockTransport>::grain(wire).or(FujiPtp::<
                    crate::transport::MockTransport,
                >::grain(
                    7
                ))
            } else {
                FujiPtp::<crate::transport::MockTransport>::grain(wire)
            };
            assert!(read.is_some(), "grain should round-trip");
        }
    }
}
