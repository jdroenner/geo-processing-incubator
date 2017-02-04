extern crate iron;
extern crate params;
extern crate router;
extern crate image;
extern crate gdal;
extern crate colorizers;
extern crate num;
extern crate chrono;

pub mod mappers_handler;
//pub mod raster_traits;
//pub mod spatial_reference;
pub mod gdal_source;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
