extern crate image;
extern crate gdal;
extern crate colorizers;

use std::path::Path;
use std::fs::File;
use gdal::raster::{Dataset, RasterBand};
use gdal::metadata::Metadata;

use colorizers::simple::SimpleScale;
use colorizers::Colorizer;

use image::{ImageLuma8, PNG};

static RASTER_IN: &'static str = "../data/test2_f32.tif";
static IMAGE_OUT: &'static str = "../data/meh.png";

fn main() {
    let path = Path::new(RASTER_IN);
    let dataset = Dataset::open(path).unwrap();
    println!("dataset description: {:?}", dataset.description());

    let size = (3712, 3712);

    let rasterband: RasterBand = dataset.rasterband(1).unwrap();
    println!("rasterband description: {:?}", rasterband.description());
    println!("rasterband no_data_value: {:?}", rasterband.no_data_value());
    println!("rasterband type: {:?}", rasterband.band_type());
    println!("rasterband scale: {:?}", rasterband.scale());
    println!("rasterband offset: {:?}", rasterband.offset());
    let rv = rasterband.read_as::<f32>(
        (0, 0),
        (size.0 as usize, size.1 as usize),
        (size.0 as usize, size.1 as usize)
    );

    let colorizer = SimpleScale::new(0.0, 128.0);
    let imgbuf = colorizer.colorize(&rv.data, size);

    // Save the image
    let ref mut fout = File::create(&Path::new(IMAGE_OUT)).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);

}

