#[macro_use]
extern crate rocket;
use std::fs::File;

// use zstd compression
// also use the libraries for receiving files
use rocket::data::{ByteUnit, Limits};
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::http::Status;
use rocket::response::stream::ByteStream;
use rocket::response::Response;
use std::io::{self, BufReader, Cursor, Read, Write};
use tempfile::tempfile;
use uuid::Uuid;
use zstd::stream::copy_encode;
use zstd::stream::read::Decoder;
use zstd::stream::write::Encoder;
use zstd::stream::zio::Reader;

#[post("/compress", data = "<file>")]
fn compress<'a>(file: File) -> Result<Response<'a>, Status> {
    let mut encoder = match Encoder::new(file, 19) {
        Ok(e) => e,
        Err(_) => return Err(Status::InternalServerError),
    };

    let mut compressed = Vec::new();
    let mut not_compressed = Vec::new();

    match file.read_to_end(&mut not_compressed) {
        Ok(_) => (),
        Err(_) => return Err(Status::InternalServerError),
    };

    match copy_encode(file, &mut encoder, 19) {
        Ok(_) => (),
        Err(_) => return Err(Status::InternalServerError),
    };

    // write the compressed data to a temporary file
    let mut temp = match tempfile() {
        Ok(t) => t,
        Err(_) => return Err(Status::InternalServerError),
    };

    match encoder.finish().unwrap().read_to_end(&mut compressed) {
        Ok(()) => (),
        Err(_) => return Err(Status::InternalServerError),
    }

    // derive a file name from the original file name
    // and the compression level
    // generate a random filename
    let file_name = Uuid::new_v4();
    let file_name = format!("{}-compressed.zst", file_name);

    // return the temporary file
    let mut response = rocket::response::Builder::new(Response::new());
    let response = response
        .header(Header::new(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file_name),
        ))
        .header(Header::new("Content-Type", "application/octet-stream"))
        .streamed_body(Cursor::new(&compressed))
        .finalize();

    Ok(response)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![compress])
}
