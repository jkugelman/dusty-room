use image::GrayImage;

/// An 8-bit image. Images can be [converted to RGBA] using the [`PaletteBank`]'s active
/// [`Palette`].
///
/// [converted to RGBA]: super::palette::ToRgba::to_rgba
/// [`PaletteBank`]: super::palette::PaletteBank
/// [`Palette`]: super::palette::Palette
pub type Image = GrayImage;
