use shark::point::{Point, primitives::line};

pub fn test_strip() -> impl Iterator<Item = Point> + Clone {
    line(Point::new(-18.0, 0.0, 0.0), Point::new(18.0, 0.0, 0.0), 72)
}
