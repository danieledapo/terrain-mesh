use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Bbox {
    min: Vec2,
    max: Vec2,
}

#[derive(Debug, Copy, Clone)]
pub struct Circle {
    center: Vec2,
    radius: f64,
}

impl Vec2 {
    pub fn zero() -> Self {
        Vec2::new(0.0, 0.0)
    }

    pub fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    pub fn dist(&self, p: Vec2) -> f64 {
        self.dist2(p).sqrt()
    }

    pub fn dist2(&self, p: Vec2) -> f64 {
        (*self - p).norm2()
    }

    pub fn norm(&self) -> f64 {
        self.norm2().sqrt()
    }

    pub fn norm2(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }
}

impl Bbox {
    pub fn center(&self) -> Vec2 {
        unimplemented!()
    }

    pub fn split(&self, p: Vec2) -> [Bbox; 4] {
        unimplemented!()
    }

    pub fn expand(&mut self, p: Vec2) {
        unimplemented!()
    }

    pub fn contains(&self, p: Vec2) -> bool {
        unimplemented!()
    }
}

impl Circle {
    pub fn new(center: Vec2, radius: f64) -> Self {
        Circle { center, radius }
    }

    pub fn contains(&self, p: Vec2) -> bool {
        self.center.dist(p) <= self.radius
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(mut self, rhs: Vec2) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(mut self, rhs: Vec2) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}

impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(mut self, rhs: Vec2) -> Self::Output {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self
    }
}

impl Div for Vec2 {
    type Output = Vec2;

    fn div(mut self, rhs: Vec2) -> Self::Output {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self
    }
}
