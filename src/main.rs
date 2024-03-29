#[macro_use]
extern crate rocket;

// use zstd compression
// also use the libraries for receiving files
use rocket::{fs::TempFile, http::{Header, Status}, response::Response, tokio::io::AsyncReadExt};
use std::io::{Cursor, Read, Write};
use tempfile::tempfile;
use uuid::Uuid;
use zstd::stream::write::Encoder;

fn consume<'a, T>(r: &'a T) -> &'static T {
    r
}

#[post("/compress", data = "<file>")]
async fn compress<'a>(file: TempFile<'_>) -> Result<Response<'a>, Status> {
    // let a temporary file that the buffer will be saved into
    let mut buffer = match tempfile() {
        Ok(file) => file,
        Err(_) => return Err(Status::InternalServerError),
    };

    let mut buffer1 = Vec::new();

    if let Ok(mut file) = file.open().await {
        file.read_to_end(&mut buffer1);
    } else {
        return Err(Status::InternalServerError);
    }

    if let Err(_) = buffer.write_all(&mut buffer1) {
        return Err(Status::InternalServerError);
    }

    let encoder = match Encoder::new(buffer, 19) {
        Ok(e) => e,
        Err(_) => return Err(Status::InternalServerError),
    };

    let mut compressed = vec![];

    if let Err(_) = encoder.finish().unwrap().read_to_end(&mut compressed) {
        return Err(Status::InternalServerError);
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
        .streamed_body(Cursor::new(consume(&compressed)))
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
