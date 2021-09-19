use std::fmt;

use crate::wad::{self, Lump, Wad};

pub struct Map {
    name: String,
    _things: (),
    _vertices: (),
    _sides: (),
    _lines: (),
    _sectors: (),
}

impl Map {
    /// Loads a map, typically named `ExMy` for DOOM or `MAPnn` for DOOM II.
    ///
    /// Returns `Ok(None)` if the map is missing.
    pub fn load(wad: &Wad, name: &str) -> wad::Result<Option<Self>> {
        let lumps = wad.try_lumps_following(name, 11)?;
        if lumps.is_none() {
            return Ok(None);
        }
        let lumps = lumps.unwrap();

        let name = name.to_owned();
        let things = Self::read_things(lumps.get_with_name(1, "THINGS")?);
        let vertices = Self::read_vertices(lumps.get_with_name(4, "VERTEXES")?);
        let sectors = Self::read_sectors(lumps.get_with_name(8, "SECTORS")?);
        let sides = Self::read_sides(lumps.get_with_name(3, "SIDEDEFS")?);
        let lines = Self::read_lines(lumps.get_with_name(2, "LINEDEFS")?);

        Ok(Some(Map {
            name,
            _things: things,
            _vertices: vertices,
            _sides: sides,
            _lines: lines,
            _sectors: sectors,
        }))
    }

    fn read_things(_lump: &Lump) {}
    fn read_vertices(_lump: &Lump) {}
    fn read_sectors(_lump: &Lump) {}
    fn read_sides(_lump: &Lump) {}
    fn read_lines(_lump: &Lump) {}
}

impl fmt::Debug for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.name)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        assert_matches!(Map::load(&*DOOM_WAD, "E1M1"), Ok(Some(_)));
        assert_matches!(Map::load(&*DOOM_WAD, "E9M9"), Ok(None));
    }
}
