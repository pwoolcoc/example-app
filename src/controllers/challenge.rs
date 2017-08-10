use mime;

use hyper;
use hyper::StatusCode;
use hyper::server::{Request, Response};

use gotham;
use gotham::state::{State, FromState};
use gotham::http::response::create_response;

#[derive(StateData, FromState, PathExtractor, StaticResponseExtender)]
pub struct ChallengeRequestPath {
    pub name: String,
}

#[derive(StateData, FromState, QueryStringExtractor, StaticResponseExtender)]
pub struct ChallengeQueryString {
    pub count: Option<u8>,
}

pub fn index(state: State, _req: Request) -> (State, Response) {
    let res = {
        let crp = ChallengeRequestPath::borrow_from(&state);
        let name = &crp.name;

        let cqs = ChallengeQueryString::borrow_from(&state);
        let count = cqs.count;

        let hello = match count {
            Some(count) => {
                let video = r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/88M9-rBXubA" frameborder="0" allowfullscreen></iframe>"#;
                let ih = format!("<p>Izzy Mandelbaum can lift <strong>{} kgs</strong>, so more weight than you!!.</p>{}",
                                 count + 1,
                                 video);
                html(&name, &ih)
            }
            None => {
                html(&name,
                     "<p>Izzy Mandelbaum can lift more weight than you.</p><p><small>Tip: Append `?count=x`, where x is a number, to tell Izzy how much weight you can lift. Maybe you can lift more weight more than him?</small></p>")
            }
        };

        create_response(&state,
                        StatusCode::Ok,
                        Some((hello.into_bytes(), mime::TEXT_HTML)))
    };

    (state, res)
}

fn html(name: &str, internal_html: &str) -> String {
    format!("
            <html>
              <head>
                <title>Izzy's weight challenge</title>
              </head>
              <body>
                <p>Hello, <strong>{}</strong>.</p>
                {}
                <br><br>
                <a href='/'>Go Home</a>
              </body>
            </html>",
            name,
            internal_html)
}
