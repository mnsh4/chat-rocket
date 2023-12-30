#[macro_use]
extern crate rocket;

#[get("/")]
fn hello() -> &'static str {
    "Hello World from Buenos Aires"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello])
}
