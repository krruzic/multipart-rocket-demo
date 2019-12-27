#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket;
extern crate rocket_multipart_form_data;

mod middleware;
use crate::middleware::MultipartError;
use crate::middleware::NewUser;

type Result<T> = std::result::Result<T, MultipartError>;

#[post("/create_user", data = "<multipart>")]
fn new_user(multipart: Result<NewUser>) -> String {
    match multipart {
        Ok(m) => format!("Hello, {} year old named {}!", m.user.age, m.user.name),
        Err(e) => format!("Error: {}", e.reason),
    }
}

fn main() {
    rocket::ignite().mount("/", routes![new_user]).launch();
}
