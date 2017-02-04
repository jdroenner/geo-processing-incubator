use super::{Colorizer, ImageBuffer};
use image::{Luma};
use num_traits::bounds::Bounded;
use num_traits::cast::ToPrimitive;
use num_traits::Num;
use std::fmt::Debug;

pub struct SimpleScale<T: PartialOrd> {
    offset: T,
    scale: T,
}

impl <T: PartialOrd> SimpleScale<T> {
    pub fn new(offset: T, scale: T) -> Self {
        SimpleScale{offset: offset, scale: scale}
    }
}

impl <T> Colorizer<T, Luma<u8>> for SimpleScale<T> where T: Num + PartialOrd + ToPrimitive + Copy {
    fn colorize(&self, input: &[T], size: (u32, u32)) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let vec = input.iter().map(|&x| ((x - self.offset) * self.scale).to_u8().unwrap() ).collect::<Vec<_>>();
        ImageBuffer::from_raw(size.0, size.1, vec).unwrap()
    }
}

pub struct MinMaxScale;

impl MinMaxScale {
    pub fn new() -> Self {
        MinMaxScale{}
    }
}

impl <T> Colorizer<T, Luma<u8>> for MinMaxScale where T: Num + ToPrimitive + Bounded + PartialOrd + Copy + Debug {
    fn colorize(&self, input: &[T], size: (u32, u32)) -> ImageBuffer<Luma<u8>, Vec<u8>> {

        let (min, max) = input.iter().fold(
            (T::max_value(), T::min_value()), |a, b| {
                let l = if *b < a.0 { *b } else { a.0 };
                let h = if *b > a.1 { *b } else { a.1 };
                (l, h)
            }
        );
        
        println!("min,max: {:?}, {:?}", min, max);

        let vec = input.iter().map( |&x| (((x - min) / (max - min)).to_f32().unwrap() * 255.0).to_u8().unwrap() ).collect::<Vec<_>>();
        ImageBuffer::from_raw(size.0, size.1, vec).unwrap()
    }
}