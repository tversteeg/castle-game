//! Define the colors using the DB32 color pallete.
//! [https://lospec.com/palette-list/dawnbringer-32]
//!
//! Converting colors in NeoVim:
//!
//! ```vim
//! :%s#.*#\=printf("\t/// `%s`.\n\tC%d,", submatch(0), line('.') / 2 + 1)
//! ```
//!
//! ```lua
//! :luado hex = line:gsub("#",""); return string.format("Palette::C%d => Color::Rgba {red: %f, green: %f, blue: %f, alpha:1.0},", linenr, tonumber("0x"..hex:sub(1,2))/255, tonumber("0x"..hex:sub(3,4))/255, tonumber("0x"..hex:sub(5,6))/255)
//! ```

use bevy::prelude::Color;
use bevy_inspector_egui::egui::Color32;

pub enum Palette {
    /// `#000000`.
    C1,
    /// `#222034`.
    C2,
    /// `#45283c`.
    C3,
    /// `#663931`.
    C4,
    /// `#8f563b`.
    C5,
    /// `#df7126`.
    C6,
    /// `#d9a066`.
    C7,
    /// `#eec39a`.
    C8,
    /// `#fbf236`.
    C9,
    /// `#99e550`.
    C10,
    /// `#6abe30`.
    C11,
    /// `#37946e`.
    C12,
    /// `#4b692f`.
    C13,
    /// `#524b24`.
    C14,
    /// `#323c39`.
    C15,
    /// `#3f3f74`.
    C16,
    /// `#306082`.
    C17,
    /// `#5b6ee1`.
    C18,
    /// `#639bff`.
    C19,
    /// `#5fcde4`.
    C20,
    /// `#cbdbfc`.
    C21,
    /// `#ffffff`.
    C22,
    /// `#9badb7`.
    C23,
    /// `#847e87`.
    C24,
    /// `#696a6a`.
    C25,
    /// `#595652`.
    C26,
    /// `#76428a`.
    C27,
    /// `#ac3232`.
    C28,
    /// `#d95763`.
    C29,
    /// `#d77bba`.
    C30,
    /// `#8f974a`.
    C31,
    /// `#8a6f30`.
    C32,
}

impl From<Palette> for Color {
    /// Create a bevy color.
    fn from(color: Palette) -> Color {
        match color {
            Palette::C1 => Color::Rgba {
                red: 0.000000,
                green: 0.000000,
                blue: 0.000000,
                alpha: 1.0,
            },
            Palette::C2 => Color::Rgba {
                red: 0.133333,
                green: 0.125490,
                blue: 0.203922,
                alpha: 1.0,
            },
            Palette::C3 => Color::Rgba {
                red: 0.270588,
                green: 0.156863,
                blue: 0.235294,
                alpha: 1.0,
            },
            Palette::C4 => Color::Rgba {
                red: 0.400000,
                green: 0.223529,
                blue: 0.192157,
                alpha: 1.0,
            },
            Palette::C5 => Color::Rgba {
                red: 0.560784,
                green: 0.337255,
                blue: 0.231373,
                alpha: 1.0,
            },
            Palette::C6 => Color::Rgba {
                red: 0.874510,
                green: 0.443137,
                blue: 0.149020,
                alpha: 1.0,
            },
            Palette::C7 => Color::Rgba {
                red: 0.850980,
                green: 0.627451,
                blue: 0.400000,
                alpha: 1.0,
            },
            Palette::C8 => Color::Rgba {
                red: 0.933333,
                green: 0.764706,
                blue: 0.603922,
                alpha: 1.0,
            },
            Palette::C9 => Color::Rgba {
                red: 0.984314,
                green: 0.949020,
                blue: 0.211765,
                alpha: 1.0,
            },
            Palette::C10 => Color::Rgba {
                red: 0.600000,
                green: 0.898039,
                blue: 0.313725,
                alpha: 1.0,
            },
            Palette::C11 => Color::Rgba {
                red: 0.415686,
                green: 0.745098,
                blue: 0.188235,
                alpha: 1.0,
            },
            Palette::C12 => Color::Rgba {
                red: 0.215686,
                green: 0.580392,
                blue: 0.431373,
                alpha: 1.0,
            },
            Palette::C13 => Color::Rgba {
                red: 0.294118,
                green: 0.411765,
                blue: 0.184314,
                alpha: 1.0,
            },
            Palette::C14 => Color::Rgba {
                red: 0.321569,
                green: 0.294118,
                blue: 0.141176,
                alpha: 1.0,
            },
            Palette::C15 => Color::Rgba {
                red: 0.196078,
                green: 0.235294,
                blue: 0.223529,
                alpha: 1.0,
            },
            Palette::C16 => Color::Rgba {
                red: 0.247059,
                green: 0.247059,
                blue: 0.454902,
                alpha: 1.0,
            },
            Palette::C17 => Color::Rgba {
                red: 0.188235,
                green: 0.376471,
                blue: 0.509804,
                alpha: 1.0,
            },
            Palette::C18 => Color::Rgba {
                red: 0.356863,
                green: 0.431373,
                blue: 0.882353,
                alpha: 1.0,
            },
            Palette::C19 => Color::Rgba {
                red: 0.388235,
                green: 0.607843,
                blue: 1.000000,
                alpha: 1.0,
            },
            Palette::C20 => Color::Rgba {
                red: 0.372549,
                green: 0.803922,
                blue: 0.894118,
                alpha: 1.0,
            },
            Palette::C21 => Color::Rgba {
                red: 0.796078,
                green: 0.858824,
                blue: 0.988235,
                alpha: 1.0,
            },
            Palette::C22 => Color::Rgba {
                red: 1.000000,
                green: 1.000000,
                blue: 1.000000,
                alpha: 1.0,
            },
            Palette::C23 => Color::Rgba {
                red: 0.607843,
                green: 0.678431,
                blue: 0.717647,
                alpha: 1.0,
            },
            Palette::C24 => Color::Rgba {
                red: 0.517647,
                green: 0.494118,
                blue: 0.529412,
                alpha: 1.0,
            },
            Palette::C25 => Color::Rgba {
                red: 0.411765,
                green: 0.415686,
                blue: 0.415686,
                alpha: 1.0,
            },
            Palette::C26 => Color::Rgba {
                red: 0.349020,
                green: 0.337255,
                blue: 0.321569,
                alpha: 1.0,
            },
            Palette::C27 => Color::Rgba {
                red: 0.462745,
                green: 0.258824,
                blue: 0.541176,
                alpha: 1.0,
            },
            Palette::C28 => Color::Rgba {
                red: 0.674510,
                green: 0.196078,
                blue: 0.196078,
                alpha: 1.0,
            },
            Palette::C29 => Color::Rgba {
                red: 0.850980,
                green: 0.341176,
                blue: 0.388235,
                alpha: 1.0,
            },
            Palette::C30 => Color::Rgba {
                red: 0.843137,
                green: 0.482353,
                blue: 0.729412,
                alpha: 1.0,
            },
            Palette::C31 => Color::Rgba {
                red: 0.560784,
                green: 0.592157,
                blue: 0.290196,
                alpha: 1.0,
            },
            Palette::C32 => Color::Rgba {
                red: 0.541176,
                green: 0.435294,
                blue: 0.188235,
                alpha: 1.0,
            },
        }
    }
}

impl From<Palette> for Color32 {
    fn from(color: Palette) -> Color32 {
        let bevy_color: Color = color.into();
        let rgba = bevy_color.as_rgba_f32();

        Color32::from_rgba_unmultiplied(
            (rgba[0] * 255.0) as u8,
            (rgba[1] * 255.0) as u8,
            (rgba[2] * 255.0) as u8,
            (rgba[3] * 255.0) as u8,
        )
    }
}
