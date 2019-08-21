use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Bbox {
    min: Vec2,
    max: Vec2,
}

#[derive(Debug, Copy, Clone)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f64,
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
    pub fn min(&self) -> Vec2 {
        self.min
    }

    pub fn max(&self) -> Vec2 {
        self.max
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn split(&self, p: Vec2) -> [Bbox; 4] {
        debug_assert!(self.contains(p));

        [
            Bbox {
                min: self.min,
                max: p,
            },
            Bbox {
                min: Vec2::new(p.x, self.min.y),
                max: Vec2::new(self.max.x, p.y),
            },
            Bbox {
                min: Vec2::new(self.min.x, p.y),
                max: Vec2::new(p.x, self.max.y),
            },
            Bbox {
                min: p,
                max: self.max,
            },
        ]
    }

    pub fn expand(&mut self, p: Vec2) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);

        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn contains(&self, p: Vec2) -> bool {
        self.min.x <= p.x && self.min.y <= p.y && self.max.x >= p.x && self.max.y >= p.y
    }
}

impl Circle {
    pub fn new(center: Vec2, radius: f64) -> Self {
        Circle { center, radius }
    }

    pub fn circumcircle(a: Vec2, b: Vec2, c: Vec2) -> Self {
        //
        // TODO
        //
        unimplemented!()
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

impl Add<f64> for Vec2 {
    type Output = Vec2;

    fn add(mut self, rhs: f64) -> Self::Output {
        self.x += rhs;
        self.y += rhs;
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

impl Sub<f64> for Vec2 {
    type Output = Vec2;

    fn sub(mut self, rhs: f64) -> Self::Output {
        self.x -= rhs;
        self.y -= rhs;
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

impl Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;
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

impl Div<f64> for Vec2 {
    type Output = Vec2;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.x /= rhs;
        self.y /= rhs;
        self
    }
}
