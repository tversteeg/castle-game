use cgmath::{Point2, EuclideanSpace};
use collision::Aabb2;
use std::ops::{Add, Deref, DerefMut};

pub fn p(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

pub fn bb(min: Point, max: Point) -> BoundingBox {
    BoundingBox::new(min, max)
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Point(Point2<f64>);

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point(Point2::new(x, y))
    }

    pub fn as_i32(self) -> (i32, i32) {
        (self.0.x as i32, self.0.y as i32)
    }

    pub fn as_usize(self) -> Point2<usize> {
        Point2::new(self.0.x as usize, self.0.y as usize)
    }
}

impl Deref for Point {
    type Target = Point2<f64>;

    fn deref(&self) -> &Point2<f64> {
        &self.0
    }
}

impl DerefMut for Point {
    fn deref_mut(&mut self) -> &mut Point2<f64> {
        &mut self.0
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct BoundingBox(Aabb2<f64>);

impl BoundingBox {
    pub fn new(p1: Point, p2: Point) -> Self {
        BoundingBox(Aabb2::new(*p1, *p2))
    }

    pub fn to_i32(self) -> (i32, i32, i32, i32) {
        (self.min.x as i32, self.min.y as i32,
         (self.max.x - self.min.x) as i32,
         (self.max.y - self.min.y) as i32)
    }
}

impl Deref for BoundingBox {
    type Target = Aabb2<f64>;

    fn deref(&self) -> &Aabb2<f64> {
        &self.0
    }
}

impl DerefMut for BoundingBox {
    fn deref_mut(&mut self) -> &mut Aabb2<f64> {
        &mut self.0
    }
}

impl Add<Point2<f64>> for BoundingBox {
    type Output = Self;

    fn add(self, pos: Point2<f64>) -> Self {
        BoundingBox::new(Point(self.min + pos.to_vec()), Point(self.max + pos.to_vec()))
    }
}

impl Add<Point> for BoundingBox {
    type Output = Self;

    fn add(self, pos: Point) -> Self {
        BoundingBox::new(Point(self.min + pos.to_vec()), Point(self.max + pos.to_vec()))
    }
}
