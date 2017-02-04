extern crate image;
extern crate num_traits;

pub use image::{Pixel, ImageBuffer};

pub mod gray_scale;

pub trait Colorizer<I, O: Pixel> {
    fn colorize(&self, input: &[I], size: (u32, u32)) -> ImageBuffer<O, Vec<O::Subpixel>>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
