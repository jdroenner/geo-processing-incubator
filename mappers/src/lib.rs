extern crate image;
extern crate gdal;
extern crate colorizers;
extern crate num;
extern crate chrono;
extern crate hyper;
extern crate futures;
extern crate url;

#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

pub mod gdal_source;
pub mod errors;
pub mod wms_service;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
