use std::path::Path;
use std::fs::File;
use gdal::raster::{Dataset, RasterBand};
use gdal::metadata::Metadata;
use gdal;

use num::Integer;

use chrono::*;
use mappers_handler::BoundingBox;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Tick {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Tick {
    pub fn snap_date<T: Datelike>(&self, date: &T) -> NaiveDate {
        let y = date.year().div_floor(&self.year) * self.year;
        let m = date.month().div_floor(&self.month) * self.month;
        let d = date.day().div_floor(&self.day) * self.day;
        NaiveDate::from_ymd(y,m,d)
    }

    pub fn snap_time<T: Timelike>(&self, time: &T) -> NaiveTime {
        let h = time.hour().div_floor(&self.hour) * self.hour;
        let m = time.minute().div_floor(&self.minute) * self.minute;
        let s = time.second().div_floor(&self.second) * self.second;
        NaiveTime::from_hms(h,m,s)
    }

    pub fn snap_datetime<T: Datelike + Timelike>(&self, datetime: &T) -> NaiveDateTime {
        NaiveDateTime::new(self.snap_date(datetime), self.snap_time(datetime))
    }
}

pub trait Query {
     //TODO
}

pub trait Rasterized: Query {
    fn resolution(&self) -> (u64, u64);
}

pub trait Spatial: Query {
    fn bbox(&self) -> &BoundingBox;
}

pub trait Temporal: Query {
    type DateType;
    type DurationType;
    fn start(&self) -> &Self::DateType;
    fn duration(&self) -> Option<&Self::DurationType> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct SpatioTemporalRasterQuery<T>{
    pub start_time: T,
    pub bbox: BoundingBox,
    pub pixel_size: (u64, u64),
}

impl <T> Query for SpatioTemporalRasterQuery<T> {}

impl <T> Temporal for SpatioTemporalRasterQuery<T> where T: Datelike + Timelike {
    type DateType = T;
    type DurationType = Duration;
    fn start(&self) -> &T {
        &self.start_time
    }
}

impl <T> Spatial for SpatioTemporalRasterQuery<T> {
    fn bbox(&self) -> &BoundingBox {
        &self.bbox
    }
}

impl <T> Rasterized for SpatioTemporalRasterQuery<T> {
    fn resolution(&self) -> (u64, u64) {
        self.pixel_size
    }
}

pub struct SourceParams {
    pub dataset_name: &'static str,
    pub file_name_format: &'static str,
    pub tick: Option<Tick>,
}

pub struct GdalSource {
    base_path: &'static str,
    params: SourceParams,
}

impl GdalSource {
    pub fn new(base_path: &'static str, params: SourceParams) -> Self {
        GdalSource {
            base_path: base_path,
            params: params,
        }
    }

    // A simple mockup source based on GDAL.
    pub fn pull<T>(&self, query: &SpatioTemporalRasterQuery<T>) -> gdal::errors::Result<Vec<f32>> where T: Timelike + Datelike {

        // combine base path and layer name to the full path of the raster and create a Path object.
        let file_name = self.params.tick.map(|t| t.snap_datetime(query.start()).format(self.params.file_name_format).to_string()).unwrap_or(self.params.dataset_name.to_owned());

        let full_file_path = self.base_path.to_string() + &file_name;
        let path = Path::new(&full_file_path);

        // open the dataset at path (or 'throw' an error)
        let dataset = Dataset::open(path)?;
        // get the geo transform (pixel size ...) of the dataset (or 'throw' an error)
        let geo_transform = dataset.geo_transform()?;

        // transform the bounding box to pixel space using the geotransform of the dataset.
        let min = projection_to_raster_space(query.bbox.x(), geo_transform);
        let max = projection_to_raster_space(query.bbox.y(), geo_transform);

        let size = (query.resolution().0 as usize, query.resolution().1 as usize);

        // get the (first) raster band of the dataset (or 'throw' an error) ...
        let rasterband: RasterBand = dataset.rasterband(1)?;
        
        // read the data from the rasterband
        let pixel_origin = (min.0 as isize, min.1 as isize);
        let pixel_size = (max.0 - min.0, max.1 - min.1);
        let buffer = rasterband.read_as::<f32>((min.0 as isize, min.1 as isize), // pixelspace origin
                            (max.0 - min.0, max.1 - min.1), // pixelspace size
                            size /* requested raster size */);
        println!("pixel_origin: {:?}, pixel_size: {:?}, size: {:?}", pixel_origin, pixel_size, size);
        buffer.map(|b| b.data)
                         // map the returned object to the included Vec.
    }
}



fn projection_to_raster_space(coordinate: (f64, f64), geo_transform: [f64; 6]) -> (usize, usize) {
    // calculate the inverse (handling det=0 would also be required)
    let det = geo_transform[1] * geo_transform[5] - geo_transform[2] * geo_transform[4];
    let pixel_x = ((coordinate.0 - geo_transform[0]) * geo_transform[5] -
                   (coordinate.1 - geo_transform[3]) * geo_transform[2]) / det;
    let pixel_y = ((coordinate.1 - geo_transform[3]) * geo_transform[1] -
                   (coordinate.0 - geo_transform[0]) * geo_transform[4]) / det;
    println!("det: {}, pixel_x: {}, pixel_y: {}", det, pixel_x, pixel_y);
    (pixel_x as usize, pixel_y as usize)
}

fn raster_to_projection_space(pixel: (usize, usize), geo_transform: [f64; 6]) -> (f64, f64) {
    let x_projection = geo_transform[0] + pixel.0 as f64 * geo_transform[1] +
                       pixel.1 as f64 * geo_transform[2];
    let y_projection = geo_transform[3] + pixel.0 as f64 * geo_transform[4] +
                       pixel.1 as f64 * geo_transform[5];
    (x_projection, y_projection)
}