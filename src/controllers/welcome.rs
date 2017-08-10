use mime;

use hyper::StatusCode;
use hyper::server::{Request, Response};

use gotham::state::State;
use gotham::http::response::create_response;

pub fn index(state: State, _req: Request) -> (State, Response) {
    let res = create_response(&state,
                              StatusCode::Ok,
                              Some((html().into_bytes(), mime::TEXT_HTML)));

    (state, res)
}

fn html() -> String {
    "<!doctype html>
     <html>
       <head>
         <title>Izzy's weight challenge and todo list</title>
       </head>
       <body>
         <h1>Izzy's weight challenge and todo list</h1>
         <img src='http://i.imgur.com/ZiCQ72b.jpg' alt='Izzy'>
         <ul>
            <li><a href='/challenge/Rustacean'>Weight challenge</a></li>
            <li><a href='/todo'>My todo list</a></li>
        </ul>
       </body>
     </html>"
            .into()
}
