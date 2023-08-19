use raqote::{AntialiasMode, BlendMode, DrawOptions, DrawTarget, SolidSource, Source};
use vek::{Extent2, Vec2};

use crate::{
    camera::{self, Camera},
    graphics::Color,
    SIZE,
};

/// Don't draw with anti-aliasing.
const DRAW_OPTIONS: DrawOptions = DrawOptions {
    antialias: AntialiasMode::None,
    blend_mode: BlendMode::SrcOver,
    alpha: 1.0,
};

/// Draw a healthbar for a unit.
pub fn healthbar(
    health: f64,
    max_health: f64,
    pos: Vec2<f64>,
    size: Extent2<f32>,
    canvas: &mut [u32],
    camera: &Camera,
) {
    puffin::profile_scope!("Render healthbar");

    // Converted camera position
    let pos = camera.translate(pos).as_();

    // Convert the buffer to a raqote target
    let mut draw = DrawTarget::from_backing(SIZE.w as i32, SIZE.h as i32, canvas);

    // Draw background
    draw.fill_rect(
        pos.x,
        pos.y,
        size.w,
        size.h,
        &Source::Solid(Color::Red.to_source()),
        &DRAW_OPTIONS,
    );

    // Draw fill
    let fill_width = (health / max_health).clamp(0.0, 1.0) as f32 * size.w;
    draw.fill_rect(
        pos.x,
        pos.y,
        fill_width,
        size.h,
        &Source::Solid(Color::Green.to_source()),
        &DRAW_OPTIONS,
    );
}
