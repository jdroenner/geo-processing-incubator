use gdal::errors::Error as GdalError;
use serde_json::error::Error as SerdeJsonError;
use image;
use std::{self, fmt, result};

use failure::{Error, Fail};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum ErrorKind {

    #[fail(display = "IoError")]
    IoError(#[cause] std::io::Error),
    #[fail(display = "FfiNulError")]
    FfiNulError(#[cause] std::ffi::NulError),
    #[fail(display = "StrUtf8Error")]
    StrUtf8Error(#[cause] std::str::Utf8Error),
    #[fail(display = "SerdeJsonError")]
    SerdeJsonError(#[cause] SerdeJsonError),
    #[fail(display = "GdalError")]
    GdalError(#[cause] GdalError),
    #[fail(display = "ImageError")]
    ImageError(#[cause] image::ImageError),
    #[fail(display = "There is no dataset with name: {}", name)]
    UnknownDataset {
        name: String
    },
    #[fail(display = "The following WMS parameter is missing: {}", param)]
    MissingWmsParam {
        param: &'static str
    },
}