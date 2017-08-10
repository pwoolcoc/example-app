extern crate hyper;
extern crate mime;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate fern;
#[macro_use]
extern crate log;
extern crate chrono;

extern crate gotham;
#[macro_use]
extern crate gotham_derive;

mod boot;
mod session;
mod controllers;

use log::LogLevelFilter;
use hyper::server::Http;
use gotham::handler::NewHandlerService;

use boot::router::router;

pub fn start() {
    set_logging();
    // Message recieved @ag_dubs!
    //
    // https://twitter.com/ag_dubs/status/852559264510070784
    let addr = "127.0.0.1:7878".parse().unwrap();

    let server = Http::new()
        .bind(&addr, NewHandlerService::new(router()))
        .unwrap();

    println!("Listening on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}

fn set_logging() {
    fern::Dispatch::new()
        .level(LogLevelFilter::Error)
        .level_for("gotham", log::LogLevelFilter::Trace)
        .level_for("gotham::state", log::LogLevelFilter::Error)
        .level_for("todo_session", log::LogLevelFilter::Error)
        .chain(std::io::stdout())
        .format(|out, message, record| {
                    out.finish(format_args!("{}[{}][{}]{}",
                                            chrono::UTC::now().format("[%Y-%m-%d %H:%M:%S%.9f]"),
                                            record.target(),
                                            record.level(),
                                            message))
                })
        .apply()
        .unwrap();
}
