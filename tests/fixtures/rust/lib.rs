pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub enum Color { Red, Green, Blue }

pub trait Shape { fn area(&self) -> f64; }

impl Shape for Point {
    fn area(&self) -> f64 { self.x * self.y }
}
