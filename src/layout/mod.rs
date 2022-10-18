pub mod scatter;

#[derive(Debug, Clone, Copy)]
pub struct Point(pub f32, pub f32);

impl Point {
    pub fn x(&self) -> f32 {
        self.0
    }
    pub fn y(&self) -> f32 {
        self.1
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox(pub Point, pub Point);

impl BoundingBox {
    pub fn lower_left(&self) -> Point {
        self.0
    }

    pub fn upper_right(&self) -> Point {
        self.1
    }

    pub fn width(&self) -> f32 {
        self.upper_right().x() - self.lower_left().x()
    }

    pub fn height(&self) -> f32 {
        self.upper_right().y() - self.lower_left().y()
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }
}
