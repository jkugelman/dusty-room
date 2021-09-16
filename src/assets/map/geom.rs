pub struct MapSpace {}
pub struct MapLength {}

pub type Box = euclid::Box2D<i16, MapSpace>;
pub type Length = euclid::Length<i16, MapLength>;
pub type Point = euclid::Point2D<i16, MapSpace>;
pub type Rect = euclid::Rect<i16, MapSpace>;
pub type Size = euclid::Size2D<i16, MapSpace>;
pub type Vector = euclid::Vector2D<i16, MapSpace>;
