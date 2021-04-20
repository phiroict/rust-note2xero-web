#![feature(proc_macro_hygiene, decl_macro)]

use log::{error, info};
use noted2xero_core::n2x_core::parse_noted_csv;
use noted2xero_core::n2x_core::read_file;
use noted2xero_core::n2x_core::{init_logging, map_noted_to_xero};
use std::fs;

use rocket::http::RawStr;

#[macro_use]
extern crate rocket;
extern crate rocket_multipart_form_data;
use noted2xero_core::xero::XeroType;
use rocket::http::ContentType;
use rocket::response::{Content, Stream};
use rocket::Data;
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::path::Path;

use std::fs::File;
use std::io::{Cursor, Read};
use uuid::Uuid;

#[get("/healthcheck")]
fn index() -> String {
    info!("Got an incoming request :: healthcheck");
    "Hello, world!".to_string()
}

#[post("/noted/<start_invoice_number>", data = "<data>")]
fn noted(
    start_invoice_number: &RawStr,
    data: Data,
    content_type: &ContentType,
) -> Content<Stream<Cursor<Vec<u8>>>> {
    info!("Got an incoming request :: noted");
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("data")
            .content_type_by_string(Some(format!("{}/{}", mime::TEXT, mime::CSV)))
            .unwrap(),
    ]);
    let multipart_form_data = MultipartFormData::parse(content_type, data, options).unwrap();
    let noted_section = multipart_form_data.files.get("data").unwrap();

    let file_fields = noted_section;
    let dataset = &file_fields[0];
    info!("Read from data set: {:?}", dataset);
    let local_path = &dataset.path;

    let xero_data =
        process_noted_file(local_path, start_invoice_number.parse::<i32>().unwrap_or(0));
    let target_path = format!("/{}/{}.csv", "tmp", Uuid::new_v4());
    info!("Store result in a temp file at {}", target_path);
    let mut writer = csv::Writer::from_path(&target_path).unwrap();
    let headers = XeroType::get_headers();
    writer.write_record(headers).unwrap();
    for item in xero_data.iter() {
        writer
            .write_record(item.get_item_as_vector())
            .expect("Could save this line");
    }
    let flush_result = writer.flush();
    match flush_result {
        Ok(_) => {
            info!("Stored Xero csv at {}", target_path);
            let mut f = File::open(&target_path).expect("no file found");
            let metadata = fs::metadata(&target_path).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            f.read_exact(&mut buffer).expect("buffer overflow");
            fs::read(target_path).unwrap();
            let mut cursor = Cursor::new(buffer);

            cursor.set_position(0);
            Content(ContentType::CSV, Stream::from(cursor))
        }
        Err(err) => {
            error!(
                "Could not save xero file {} because: {:?}",
                &target_path, err
            );
            let cursor_error = Cursor::new(vec![]);
            Content(ContentType::CSV, Stream::from(cursor_error))
        }
    }
}

fn process_noted_file(p0: &Path, xero_invoice_number: i32) -> Vec<XeroType> {
    let noted_contents = read_file(format!("{}", p0.display()));
    let noted_collection = parse_noted_csv(&noted_contents.unwrap());
    map_noted_to_xero(&noted_collection, Option::from(xero_invoice_number))
}

fn main() {
    init_logging();
    info!("I am starting");
    rocket::ignite().mount("/", routes![index, noted]).launch();
    info!("I am done");
}
