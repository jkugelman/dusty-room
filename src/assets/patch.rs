use std::marker::PhantomData;

pub struct Patch<'wad> {
    _height: usize,
    _columns: Vec<Column<'wad>>,
    _unused: PhantomData<&'wad ()>,
}

struct Column<'wad> {
    _unused: PhantomData<&'wad ()>,
}
