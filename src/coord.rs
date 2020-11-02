use crate::{chunk::TileChunk, dimensions::Dimensions3, lib::*, map::TileMap, tile::Tile};

pub trait ToWorldCoordinates<T: Tile, C: TileChunk<T>, M: TileMap<T, C>>:
    Dimensions3 + TileChunk<T>
{
    /// Takes a `Tile` coordinate and returns its location in the `Map`.
    fn to_world_coord(&self, coord: &Vec3, map: &M, translation: Vec2) -> Option<Vec3> {
        // takes in translation of tile
        let chunk_x = (translation.x() / T::WIDTH / Self::WIDTH) + map.max_x() - 1.;
        let chunk_y = 2. - (translation.y() / T::HEIGHT / Self::HEIGHT + map.max_y() - 1.);
        let x = Self::WIDTH * chunk_x + coord.x();
        let y = (Self::HEIGHT * map.max_x()) - (Self::HEIGHT * chunk_y) + coord.y();
        Some(Vec3::new(x, y, coord.z()))
    }
}

pub trait ToCoord2 {
    fn to_coord2(&self, width: f32, height: f32) -> Vec2;
}

impl ToCoord2 for Vec2 {
    fn to_coord2(&self, _width: f32, _height: f32) -> Vec2 {
        *self
    }
}

impl ToCoord2 for usize {
    fn to_coord2(&self, width: f32, height: f32) -> Vec2 {
        let y = *self as f32 / height;
        let x = *self as f32 % width;
        Vec2::new(x, y)
    }
}

pub trait ToCoord3 {
    fn to_coord3(&self, width: f32, height: f32) -> Vec3;
}

impl ToCoord3 for Vec3 {
    fn to_coord3(&self, _width: f32, _height: f32) -> Vec3 {
        *self
    }
}

impl ToCoord3 for usize {
    fn to_coord3(&self, width: f32, height: f32) -> Vec3 {
        let z = *self as f32 / (width * height);
        let idx = *self as f32 - (z * width * height);
        let y = height - 1. - (idx / width);
        let x = idx % width;
        Vec3::new(x, y, z)
    }
}

pub trait ToIndex {
    fn to_index(&self, width: f32, height: f32) -> usize;
}

impl ToIndex for usize {
    fn to_index(&self, _width: f32, _height: f32) -> usize {
        *self
    }
}

impl ToIndex for Vec2 {
    fn to_index(&self, width: f32, _height: f32) -> usize {
        ((self.y() * width) + self.x()) as usize
    }
}

impl ToIndex for Vec3 {
    fn to_index(&self, width: f32, height: f32) -> usize {
        ((self.z() * width * height) + (self.y() * width) + self.x()) as usize
    }
}
