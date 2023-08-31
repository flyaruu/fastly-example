//! Default Compute@Edge template program.

use fastly::http::{header, Method, StatusCode};
use fastly::mime::Mime;
use fastly::{mime, Error, Request, Response, Body, Backend};
use serde::{Deserialize, Serialize};
use simplehttp::simplehttp_fastly::SimpleHttpClientFastly;
use surrealdb_http::surreal::SurrealDbClient;

/// The entry point for your application.
///
/// This function is triggered when your service receives a client request. It could be used to
/// route based on the request properties (such as method or path), send the request to a backend,
/// make completely new requests, and/or generate synthetic responses.
///
/// If `main` returns an error, a 500 error response will be delivered to the client.

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    // Log service version
    println!(
        "FASTLY_SERVICE_VERSION: {}",
        std::env::var("FASTLY_SERVICE_VERSION").unwrap_or_else(|_| String::new())
    );
    let host = "http://gorilla.codemonkey.nl:8000";
    let username = "root";
    let password = "root";
    let namespace = "myns";
    let database = "mydb";
    // let client = SimpleHttpClientReqwest::new_reqwest().unwrap();
    let http_client = SimpleHttpClientFastly::new("surreal");
    let mut client = SurrealDbClient::new(&username, &password, &host, &namespace, &database, Box::new(     http_client));

    
    #[derive(Deserialize,Serialize,Debug)]
    struct ActorWithFilms {
        films: Vec<String>,
        first_name: String,
        last_name: String,
        actor_id: usize,
    }

    // Filter request methods...
    match req.get_method() {
        // Block requests with unexpected methods
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD, PURGE")
                .with_body_text_plain("This method is not allowed\n"))
        }

        // Let any other requests through
        _ => (),
    };

    // Pattern match on the path...
    match req.get_path() {
        // If request is to the `/` path...
        "/" => {
            // Below are some common patterns for Compute@Edge services using Rust.
            // Head to https://developer.fastly.com/learning/compute/rust/ to discover more.

            // Create a new request.
            let mut bereq = Request::get("http://httpbin.org/headers")
                .with_header("X-Custom-Header", "Welcome to Compute@Edge!")
                .with_ttl(60);

            // Add request headers.
            bereq.set_header(
                "X-Another-Custom-Header",
                "Recommended reading: https://developer.fastly.com/learning/compute",
            );

            // Forward the request to a backend.
            // let mut beresp = bereq.send("backend_name")?;

            // Remove response headers.
            // beresp.remove_header("X-Another-Custom-Header");

            // Log to a Fastly endpoint.
            // use std::io::Write;
            // let mut endpoint = fastly::log::Endpoint::from_name("my_endpoint");
            // writeln!(endpoint, "Hello from the edge!").unwrap();

            // Send a default synthetic response.
            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::TEXT_HTML_UTF_8)
                .with_body(include_str!("welcome-to-compute@edge.html")))
        },
        "/surreal" => {
            let result = client.query_single::<ActorWithFilms>("SELECT *,->played_in->film.title as films FROM actor WHERE id=actor:1").unwrap();
            let first = result.result.first().unwrap();
            println!("Result: {:?}",first);
            let serialized = serde_json::to_string_pretty(first).unwrap();
            let body = Body::from(serialized.as_bytes());
            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::APPLICATION_JSON)
                .with_body(body)
        )

        },
        "/swanson" => {
            let swanson = Backend::from_name("swanson").unwrap();
            let bereq = Request::get("https://ron-swanson-quotes.herokuapp.com/v2/quotes");
            println!("Sending reuest: {:?}",bereq);
            let resp = bereq.send(swanson).unwrap();
            println!("...Received response: {:?}",resp);
            Ok(resp)
            // Request::get("https://ron-swanson-quotes.herokuapp.com/v2/quotes").
            // "https://ron-swanson-quotes.herokuapp.com/v2/quotes"
        },
        "/trivia" => {
            let trivia = Backend::from_name("trivia")?;
            let bereq = Request::get("http://jservice.io/api/random");
            println!("Sending reuest: {:?}",bereq);
            let resp = bereq.send(trivia)?;
            println!("Received response: {:?}",resp);
            Ok(resp)
            // Request::get("https://ron-swanson-quotes.herokuapp.com/v2/quotes").
            // "https://ron-swanson-quotes.herokuapp.com/v2/quotes"
        }
        

        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}
