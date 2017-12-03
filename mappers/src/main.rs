extern crate hyper;
extern crate futures;
extern crate mappers;

use hyper::server::{Http};


use mappers::wms_service::WmsService;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(WmsService::new("/mnt/d/git/geo-processing-incubator/data".to_owned()))).unwrap();
    server.run().unwrap();
}