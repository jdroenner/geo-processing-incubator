extern crate iron;
extern crate router;
extern crate mappers;
extern crate config;

use iron::prelude::*;
use iron::status;
use router::Router;

use mappers::mappers_handler::MappersHandler;

fn main() {
	
	config::merge(config::File::new("mappers", config::FileFormat::Toml)).unwrap();

    let mut router = Router::new();

    router.get("/", hello_world, "index");
    
    let data_directory : String = String::from(config::get_str("data_directory").unwrap());
    router.get("/:query", MappersHandler::new(data_directory), "query");

    fn hello_world(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Meh!")))
    }
    
    let port = config::get_int("port").unwrap();
    let bind_addr = format!("localhost:{}", port);
    Iron::new(router).http(bind_addr).unwrap();
}