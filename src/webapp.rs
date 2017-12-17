use std::collections::HashMap;
use std::sync::Mutex;
use std::io;

use rocket::fairing::{AdHoc, Fairing};
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, Form, FlashMessage, FromRequest, Request};
use rocket::response::{Flash, NamedFile, Redirect};
use rocket::{self, Rocket, State};
use rocket_contrib::Template;
use router;
use database;

type Db = Mutex<database::Db>;

#[derive(FromForm)]
struct Login {
    username: String,
    password: String
}

#[derive(Debug)]
struct Token;

#[derive(Debug)]
struct Config {
    username: String,
    password: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for Token {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Token, ()> {
        request.cookies()
            .get_private("token")
            .map(|_| Token)
            .or_forward(())
    }
}

#[post("/login", data = "<login>")]
fn login(mut cookies: Cookies,
         login: Form<Login>,
         token_config: State<Config>) -> Flash<Redirect> {
    if login.get().username == token_config.username &&
        login.get().password ==  token_config.password {
        cookies.add_private(Cookie::new("token", "Token"));
        Flash::success(Redirect::to("/"), "Successfully logged in.")
    } else {
        Flash::error(Redirect::to("/login"), "Invalid username/password.")
    }
}

#[post("/logout")]
fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("token"));
    Flash::success(Redirect::to("/login"), "Successfully logged out.")
}

#[get("/login")]
fn login_token(_token: Token) -> Redirect {
    Redirect::to("/")
}

#[get("/login", rank = 2)]
fn login_page(flash: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg());
    }

    Template::render("login", &context)
}

#[get("/")]
fn dashboard(_token: Token,
             config: State<router::Config>,
             db: State<Db>) -> Template {
    let mac_addresses = router::run(&config).unwrap_or(Vec::new());

    let mut ctx = HashMap::new();
    let db = db.lock().expect("Db connection");

    let vec = db.query(&mac_addresses);
    ctx.insert("items", vec);
    Template::render("index", &ctx)
}

#[get("/", rank = 2)]
fn dashboard_no_token() -> Redirect {
    Redirect::to("/login")
}

#[get("/style.css")]
fn stylesheet() -> io::Result<NamedFile> {
    NamedFile::open("templates/style.css")
}

fn fairing() -> impl Fairing {
    fn get_config(rocket: Rocket, name: &str)
                  -> Result<(String, Rocket), Rocket> {
        let value = rocket.config()
            .get_str(name)
            .map(|s| s.to_string());
        match value {
            Ok(s) => Ok((s, rocket)),
            Err(_) => Err(rocket),
        }
    }
    AdHoc::on_attach(|rocket| {
        // app config
        let (token_username, rocket) = get_config(rocket, "token_username")?;
        let (token_password, rocket) = get_config(rocket, "token_password")?;
        let config = Config {
            username: token_username,
            password: token_password,
        };
        // router config
        let (router_host, rocket) = get_config(rocket, "router_host")?;
        let (router_username, rocket) = get_config(rocket, "router_username")?;
        let (router_password, rocket) = get_config(rocket, "router_password")?;
        let router_config = router::Config {
            host: router_host,
            username: router_username,
            password: router_password,
        };
        // db config
        let (db_file, rocket) = get_config(rocket, "db_file")?;
        let db = database::Db::new(&db_file);
        Ok(rocket
           .manage(config)
           .manage(router_config)
           .manage(Mutex::new(db)))
    })
}

pub fn rocket() -> Rocket {
    rocket::ignite()
        .attach(Template::fairing())
        .attach(fairing())
        .mount("/", routes![
            stylesheet,
            dashboard,
            dashboard_no_token,
            login,
            login_token,
            login_page,
            logout,
        ])
}
