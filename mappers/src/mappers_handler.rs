use std::path::Path;
use std::fs::File;

use iron::prelude::*;
use iron::{Handler, status};
use iron::headers::ContentType;
use image;

use colorizers::gray_scale::{MinMaxScale};
use colorizers::Colorizer;

use params::{Value, FromValue};

use chrono::*;
use errors::*;

use gdal_source::*;

use serde_json;

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

    fn load_params_from_json<'a>(&self, dataset_name: &'a str) -> Result<SourceParams> {
        println!("load_params_from_json: layer name: {}", dataset_name);
        let config_path = Path::new(&self.base_path).join(dataset_name);
        println!("load_params_from_json: layer config_path: {:?}", config_path);
        let f = File::open(config_path)?;
        let params = serde_json::from_reader(f)?;
        Ok(params)
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


        // construct the query
        let query = SpatioTemporalRasterQuery{start_time: chrono_time, bbox: wms_bbox, pixel_size: (wms_width, wms_height)};

        // get source params        
        let source_params = match map.find(&["layer"]) {
            Some(&Value::String(ref name)) => self.load_params_from_json(name),
            _ => Err(ErrorKind::MissingWmsParam("layer").into())
        };

        // construct the source, request and then create a response
        match source_params.map(|sp| GdalSource::new(&self.base_path, sp))
            .and_then(|source| source.pull(&query)) {
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
                let resp = Response::with((status::Ok, format!("Error: {}", e)));
                Ok(resp)
            },
        }
    }    
}


