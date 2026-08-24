#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)
)]
pub enum FilmSimulation {
    Provia,
    Velvia,
    Astia,
    ProNegHigh,
    ProNegStandard,
    Monochrome,
    MonochromeYellow,
    MonochromeRed,
    MonochromeGreen,
    Sepia,
    ClassicChrome,
    Acros,
    AcrosYellow,
    AcrosRed,
    AcrosGreen,
    Eterna,
    ClassicNegative,
    EternaBleachBypass,
    NostalgicNegative,
    RealaAce,
}
