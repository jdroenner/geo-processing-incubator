use futures;

use hyper;

use std::path::Path;
use std::fs::File;
use image;

use chrono::*;
use errors;

use gdal_source::*;
use colorizers::gray_scale::{MinMaxScale};
use colorizers::Colorizer;

use serde_json;
use url;
use std::collections::HashMap;

// A simple raster service
#[derive(Debug, Clone)]
pub struct WmsService {
    base_path: String,
}

impl WmsService {
    pub fn new(base_path: String) -> Self {
        WmsService { base_path: base_path }
    }

    fn load_params_from_json<'a>(&self, dataset_name: &'a str) -> errors::Result<SourceParams> {
        println!("load_params_from_json: layer name: {}", dataset_name);
        let config_path = Path::new(&self.base_path).join(dataset_name);
        println!("load_params_from_json: layer config_path: {:?}", config_path);
        let f = File::open(config_path)?;
        let params = serde_json::from_reader(f)?;
        Ok(params)
    }

    fn build_query_from_params(param_map: &HashMap<String, String>) -> SpatioTemporalRasterQuery<DateTime<Utc>> {
        // get other wms params from the params map.
        let wms_width = param_map.get("width").and_then(|w| w.parse().ok()).unwrap_or(256);
        let wms_height = param_map.get("height").and_then(|h| h.parse().ok()).unwrap_or(256);
        let wms_bbox = param_map.get("bbox").and_then(|b| b.parse().ok()).unwrap_or_default();
        let chrono_time = param_map.get("time").and_then(|wms_time| wms_time.parse().ok()).unwrap(); // TODO: remove unwrap
        SpatioTemporalRasterQuery{start_time: chrono_time, bbox: wms_bbox, pixel_size: (wms_width, wms_height)}
    }
}

impl hyper::server::Service for WmsService {
    // boilerplate hooking up hyper's server types
    type Request = hyper::server::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<futures::Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: hyper::server::Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        let req_query_url = req.query().unwrap_or("");
        let parsed_query_url= url::form_urlencoded::parse(req_query_url.as_bytes());
        let query_map: HashMap<String, String> = parsed_query_url.into_owned().collect();
        println!("Query map: {:?}", query_map);
        let mappers_query = WmsService::build_query_from_params(&query_map);
        println!("mappers_query: {:?}", mappers_query);

        let source_params: Result<SourceParams, errors::Error> = query_map.get("layer").ok_or(errors::ErrorKind::MissingWmsParam("layer").into()).and_then(|ref layer_name| self.load_params_from_json(layer_name));
        println!("source_params: {:?}", source_params);

        let x = source_params.map(|sp| GdalSource::new(&self.base_path, sp))
                            .and_then(|source| source.pull(&mappers_query))
                            .and_then(|rd| {
            // create a new simple scaling colorizer
                let colorizer = MinMaxScale::new();
                // use the colorizer to convert the raster data into an Image (u8)
                let imgbuf = colorizer.colorize(&rd, mappers_query.pixel_size);
                // create a buffer where the PNG is stored...
                let mut buffer = vec![];
                // encode the Image as PNG or return a InternalServerError
                image::ImageLuma8(imgbuf).save(&mut buffer, image::PNG).map(|()| buffer).map_err(|e| e.into())
        });

        Box::new(futures::future::ok(
            hyper::server::Response::new()
                //.with_header(hyper::header::ContentLength(req_query.len() as u64))
                .with_header(hyper::header::ContentType::png())
                .with_body(x.unwrap()) // TODO: dont unwrap. move loading into a future!
        ))
    }
}