use iron::prelude::*;
use iron::{Handler, status};
use iron::headers::ContentType;
use image;

use colorizers::gray_scale::{MinMaxScale};
use colorizers::Colorizer;

use params::{Value, FromValue};

use chrono::*;

use gdal_source::*;

// A struct representing a bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn x(&self) -> (f64, f64) {
        (self.min_x, self.max_x)
    }

    pub fn y(&self) -> (f64, f64) {
        (self.min_y, self.max_y)
    }
}

// implement a default value for a bounding box
impl Default for BoundingBox {
    fn default() -> BoundingBox {
        BoundingBox {
            min_x: -180.,
            min_y: -90.,
            max_x: 180.,
            max_y: 90.,
        }
    }
}

// implement conversion from a params Value to a bounding box
impl FromValue for BoundingBox {
    fn from_value(value: &Value) -> Option<BoundingBox> {
        // if the vaule is
        match *value {
            // a string, than split it into a vec at ',' and map te results into f64.
            Value::String(ref string) => {
                let buf: Vec<f64> = string.split(",").filter_map(|w| w.parse().ok()).collect();
                // return a new bounding box with the values from the params string.
                Some(BoundingBox {
                    min_x: buf[0],
                    min_y: buf[1],
                    max_x: buf[2],
                    max_y: buf[3],
                })
            }
            // return None
            _ => None,
        }
    }
}

// A struct representing a mappers iron handler.
#[derive(Debug, Clone)]
pub struct MappersHandler {
    base_path: String,
}

impl MappersHandler {
    pub fn new(base_path: String) -> Self {
        MappersHandler { base_path: base_path }
    }
}

impl Handler for MappersHandler {
    // implement Handler for MappersHandler
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        use params::Params;
        // get a reference to the map holding the params (e.g. url encoded)
        let map = req.get_ref::<Params>().unwrap();



        // get other wms params from the params map.
        let wms_width = map.find(&["width"]).and_then(|w| u64::from_value(w)).unwrap_or(256);
        let wms_height = map.find(&["height"]).and_then(|h| u64::from_value(h)).unwrap_or(256);
        let wms_bbox = map.find(&["bbox"]).and_then(|b| BoundingBox::from_value(b)).unwrap_or_default();
        let chrono_time = map.find(&["time"]).and_then(|val| {
            match val {
                &Value::String(ref wms_time) => wms_time.parse::<DateTime<UTC>>().ok(),
                _ => None,
            }            
        }).unwrap();
        println!("chrono_time: {:?}", chrono_time);       

        // get source params
        /*
        let source_params = match map.find(&["layer"]) {
            Some(&Value::String(ref name)) => name,
            _ => "meh.tif",
        };
        */

        // TODO: get this from JSON -> add serde for Params...
        let source_params = SourceParams {
                dataset_name: "Meteosat",
                file_name_format: "msg_%Y%m%d_%H%M.tif",
                tick: Some(Tick{
                    year: 1,
                    month: 1,
                    day: 1,
                    hour: 1,
                    minute: 15,
                    second: 60,
                }),
        };

        // construct the source
        let source = GdalSource::new(self.base_path.clone(), source_params);

        // construct the query
        let query = SpatioTemporalRasterQuery{start_time: chrono_time, bbox: wms_bbox, pixel_size: (wms_width, wms_height)};
        
        // call a (raster) source and handle the result.
        match source.pull(&query) {
            // result is ok...
            Ok(data) => {
                // create a new simple scaling colorizer
                let colorizer = MinMaxScale::new();
                // use the colorizer to convert the raster data into an Image (u8)
                let imgbuf = colorizer.colorize(&data, (wms_width as u32, wms_height as u32));
                // create a buffer where the PNG is stored...
                let mut buffer = vec![];
                // encode the Image as PNG or return a InternalServerError
                if let Err(e) = image::ImageLuma8(imgbuf).save(&mut buffer, image::PNG) {
                    return Err(IronError::new(e, status::InternalServerError));
                }
                // create a new Iron response where the content is the encoded image (PNG)
                let mut resp = Response::with((status::Ok, buffer));
                // set the HTML ContentType header to PNG
                resp.headers.set(ContentType::png());
                // return the response to Iron
                Ok(resp)
            }
            // result is an error. Return an error to Iron.
            Err(e) => {
                let resp = Response::with((status::Ok, format!("source error: {}", e)));
                Ok(resp)
            },
        }
    }    
}


