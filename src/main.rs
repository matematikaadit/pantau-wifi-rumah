#![feature(plugin, decl_macro, custom_derive, conservative_impl_trait)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate regex;
extern crate reqwest;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate rusqlite;

mod router;
mod webapp;
mod database;

fn main() {
    webapp::rocket().launch();
}
