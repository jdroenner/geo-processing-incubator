extern crate iron;
extern crate router;
extern crate mappers;

use iron::prelude::*;
use iron::status;
use router::Router;

use mappers::mappers_handler::MappersHandler;

static DATA_DIR: &'static str = "/mnt/d/git/geo-processing-incubator/data/";

fn main() {

    let mut router = Router::new();

    router.get("/", hello_world, "index");
    router.get("/:query", MappersHandler::new(DATA_DIR), "query");

    fn hello_world(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Meh!")))
    }
    
    Iron::new(router).http("localhost:3000").unwrap();
}