/// Indicates that a type has coordinates in map space, which is useful to prevent mixing values
/// from other coordinate systems such as world space or screen space.
pub struct Space;

pub type Angle = euclid::Angle<f32>;
pub type Box2D = euclid::Box2D<i16, Space>;
pub type Length = euclid::Length<i16, Space>;
pub type Point2D = euclid::Point2D<i16, Space>;
pub type Rect = euclid::Rect<i16, Space>;
pub type Size2D = euclid::Size2D<i16, Space>;
pub type Vector2D = euclid::Vector2D<i16, Space>;
