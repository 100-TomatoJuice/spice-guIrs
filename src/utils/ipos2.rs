use std::ops::{Add, Rem};

use egui::Pos2;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IPos2 {
    pub x: i32,
    pub y: i32,
}

impl IPos2 {
    pub const ZERO: IPos2 = IPos2 { x: 0, y: 0 };
    pub const X: IPos2 = IPos2 { x: 1, y: 0 };
    pub const Y: IPos2 = IPos2 { x: 0, y: 1 };
    pub const NEG_X: IPos2 = IPos2 { x: -1, y: 0 };
    pub const NEG_Y: IPos2 = IPos2 { x: 0, y: -1 };
    pub const DIRECTIONS: [IPos2; 4] = [IPos2::X, IPos2::Y, IPos2::NEG_X, IPos2::NEG_Y];

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_pos2(&self) -> Pos2 {
        Pos2::new(self.x as f32, self.y as f32)
    }
}

impl Add for IPos2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Rem<i32> for IPos2 {
    type Output = Self;

    fn rem(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x % rhs,
            y: self.y % rhs,
        }
    }
}

pub trait Pos2Ext {
    fn to_ipos2(&self, grid_size: i32) -> IPos2;
}

impl Pos2Ext for Pos2 {
    fn to_ipos2(&self, grid_size: i32) -> IPos2 {
        IPos2 {
            x: (self.x / grid_size as f32).round() as i32 * grid_size,
            y: (self.y / grid_size as f32).round() as i32 * grid_size,
        }
    }
}
