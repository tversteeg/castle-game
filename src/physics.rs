use std::time::Duration;
use std::ops::Add;
use aabb2::{self, AABB2};

#[derive(Component, Debug, Copy, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    pub fn as_i32(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;

        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Velocity {
    pub x: f64,
    pub y: f64
}

impl Velocity {
    pub fn new(x: f64, y: f64) -> Self {
        Velocity { x, y }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rect { x, y, width, height }
    }

    pub fn hit_point(&self, point: (f64, f64)) -> bool {
        point.0 >= self.x && point.0 < self.x + self.width
            && point.1 >= self.y && point.1 < self.y + self.height
    }

    pub fn to_i32(&self) -> (i32, i32, i32, i32) {
        (self.x as i32, self.y as i32, self.width as i32, self.height as i32)
    }
}

impl Add<Position> for Rect {
    type Output = Rect;

    fn add(self, pos: Position) -> Rect {
        Rect::new(self.x + pos.x, self.y + pos.y, self.width, self.height)
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct BoundingBox(pub Rect);

impl BoundingBox {
    pub fn to_aabb(&self, pos: &Position) -> AABB2<f64> {
        let new_x = self.0.x + pos.x;
        let new_y = self.0.y + pos.y;
        aabb2::new([new_x, new_y],
                   [new_x + self.0.width, new_y + self.0.height])
    }
}

pub struct DeltaTime(pub Duration);

impl DeltaTime {
    pub fn new(time: f64) -> Self {
        DeltaTime(Duration::from_millis((time * 1000.0) as u64))
    }

    pub fn to_seconds(&self) -> f64 {
        self.0.as_secs() as f64 + self.0.subsec_nanos() as f64 * 1e-9
    }
}

pub struct Gravity(pub f64);
