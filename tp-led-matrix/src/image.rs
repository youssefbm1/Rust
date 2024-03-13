#[repr(C)]
#[derive(Clone, Copy, Default)]
#[derive(PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

use crate::gamma::gamma_correct;

impl Color {
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub fn gamma_correct(&self) -> Self {
        Color {
            r: gamma_correct(self.r),
            g: gamma_correct(self.g),
            b: gamma_correct(self.b),
        }
    }
}

use micromath::F32Ext as _;

use core::ops::Mul;

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            r: (self.r as f32 * rhs).clamp(0.0, 255.0).round() as u8,
            g: (self.g as f32 * rhs).clamp(0.0, 255.0).round() as u8,
            b: (self.b as f32 * rhs).clamp(0.0, 255.0).round() as u8,
        }
    }
}

use core::ops::Div;

impl Div<f32> for Color {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        self * (1.0 / rhs)
    }
}

pub struct Image([Color; 64]);

impl Image {
    pub fn new_solid(color: Color) -> Self {
        Image([color; 64])
    }
}

impl Default for Image {
    fn default() -> Self {
        Image([Color::WHITE; 64])
    }
}

use core::ops::Index;

impl Index<(usize, usize)> for Image {
    type Output = Color;

    fn index(&self, (row, column): (usize, usize)) -> &Self::Output {
        &self.0[row * 8 + column]
    }
}

use core::ops::IndexMut;

impl IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, (row, column): (usize, usize)) -> &mut Self::Output {
        &mut self.0[row * 8 + column]
    }
}

impl Image {
    pub fn row(&self, row: usize) -> &[Color] {
        &self.0[row * 8..(row + 1) * 8]
    }
}

impl Image {
    pub fn gradient(color: Color) -> Self {
        let mut gradient_image = Image::default();

        for row in 0..8 {
            for col in 0..8 {
                let divisor = 1 + row * row + col;
                gradient_image[(row, col)] = color / divisor as f32;
            }
        }

        gradient_image
    }
}

use core::mem;

impl AsRef<[u8; 192]> for Image {
    fn as_ref(&self) -> &[u8; 192] {
        unsafe { mem::transmute(self) }
    }
}

impl AsMut<[u8; 192]> for Image {
    fn as_mut(&mut self) -> &mut [u8; 192] {
        unsafe { mem::transmute(self) }
    }
}



// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_gradient() {
//         let gradient_image = Image::gradient(Color::RED);
//         assert_eq!(gradient_image[(0, 0)], Color::RED);
//     }
// }






