#![feature(plugin, decl_macro, custom_derive, conservative_impl_trait)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate rocket;
extern crate rocket_contrib;

mod router;
mod webapp;

fn main() {
    webapp::rocket().launch();
}
