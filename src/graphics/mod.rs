use raqote::SolidSource;

pub mod healthbar;

/// Different colors.
///
/// Based on DB32 scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Color {
    Black,
    DarkestBlue,
    DarkPurple,
    DarkBrown,
    Brown,
    Orange,
    DarkSand,
    Sand,
    Yellow,
    LightGreen,
    Green,
    Turqoise,
    DarkGreen,
    DarkGreenBrown,
    DarkGreenBlue,
    DarkBlue,
    DarkTurqoise,
    Blue,
    LightBlue,
    LighterBlue,
    SkyBlue,
    White,
    Gray,
    DarkGray,
    DarkerGray,
    DarkestGray,
    Purple,
    Red,
    Salmon,
    Pink,
    ForestGreen,
    DarkForestGreen,
}

impl Color {
    /// Convert the color to it's binary representation.
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::Black => 0xff_00_00_00,
            Self::DarkestBlue => 0xff_22_20_34,
            Self::DarkPurple => 0xff_45_28_3c,
            Self::DarkBrown => 0xff_66_39_31,
            Self::Brown => 0xff_8f_56_3b,
            Self::Orange => 0xff_df_71_26,
            Self::DarkSand => 0xff_d9_a0_66,
            Self::Sand => 0xff_ee_c3_9a,
            Self::Yellow => 0xff_fb_f2_36,
            Self::LightGreen => 0xff_99_e5_50,
            Self::Green => 0xff_6a_be_30,
            Self::Turqoise => 0xff_37_94_6e,
            Self::DarkGreen => 0xff_4b_69_2f,
            Self::DarkGreenBrown => 0xff_52_4b_24,
            Self::DarkGreenBlue => 0xff_32_3c_39,
            Self::DarkBlue => 0xff_3f_3f_74,
            Self::DarkTurqoise => 0xff_30_60_82,
            Self::Blue => 0xff_5b_6e_e1,
            Self::LightBlue => 0xff_63_9b_ff,
            Self::LighterBlue => 0xff_5f_cd_e4,
            Self::SkyBlue => 0xff_cb_db_fc,
            Self::White => 0xff_ff_ff_ff,
            Self::Gray => 0xff_9b_ad_b7,
            Self::DarkGray => 0xff_84_7e_87,
            Self::DarkerGray => 0xff_69_6a_6a,
            Self::DarkestGray => 0xff_59_56_52,
            Self::Purple => 0xff_76_42_8a,
            Self::Red => 0xff_ac_32_32,
            Self::Salmon => 0xff_d9_57_63,
            Self::Pink => 0xff_d7_7b_ba,
            Self::ForestGreen => 0xff_8f_97_4a,
            Self::DarkForestGreen => 0xff_8a_6f_30,
        }
    }

    /// To raqote solid source.
    pub fn to_source(self) -> SolidSource {
        let [b, g, r, a] = self.as_u32().to_ne_bytes();

        SolidSource::from_unpremultiplied_argb(a, r, g, b)
    }
}
