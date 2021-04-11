#![feature(proc_macro_hygiene, decl_macro)]

use noted2xero_core::n2x_core::{init_logging, map_noted_to_xero};
use noted2xero_core::n2x_core::read_file;
use noted2xero_core::n2x_core::parse_noted_csv;
use std::fs;
use log::{info,error};


use rocket::http::{RawStr};


#[macro_use] extern crate rocket;
extern crate rocket_multipart_form_data;
use rocket::Data;
use rocket::http::ContentType;
use rocket_multipart_form_data::{mime, MultipartFormDataOptions, MultipartFormDataField, MultipartFormData};
use std::path::PathBuf;
use noted2xero_core::xero::XeroType;
use rocket::response::{Stream, Content};

use uuid::Uuid;
use std::fs::File;
use std::io::{Read, Cursor};


#[get("/healthcheck")]
fn index() -> String {
    info!("Got an incoming request :: healthcheck");
    let retval = format!("Hello, world!");
    retval
}

#[post("/noted/<start_invoice_number>", data = "<data>")]
fn noted( start_invoice_number: &RawStr, data: Data, content_type: &ContentType) -> Content<Stream<Cursor<Vec<u8>>>> {
    info!("Got an incoming request :: noted");
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            MultipartFormDataField::file("data").content_type_by_string(Some(format!("{}/{}",mime::TEXT, mime::CSV))).unwrap(),
        ]
    );
    let multipart_form_data = MultipartFormData::parse(content_type, data, options).unwrap();
    let noted_section = multipart_form_data.files.get("data").unwrap();

    let file_fields = noted_section;
    let dataset = &file_fields[0];
    info!("Read from data set: {:?}", dataset);
    let local_path = &dataset.path;

    let xero_data = process_noted_file(local_path, start_invoice_number.parse::<i32>().unwrap_or(0) );
    let target_path = format!("/{}/{}.csv", "tmp", Uuid::new_v4());
    let mut writer = csv::Writer::from_path(&target_path).unwrap();
    let headers = XeroType::get_headers();
    writer.write_record(headers).unwrap();
    for item in xero_data.iter() {
        writer.write_record(item.get_item_as_vector()).expect("Could save this line");
    }
    let flush_result = writer.flush();
    match flush_result {
        Ok(_) => {
            info!("Stored Xero csv at {}",target_path);
            let mut f = File::open(&target_path).expect("no file found");
            let metadata = fs::metadata(&target_path).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            f.read(&mut buffer).expect("buffer overflow");
            fs::read(target_path).unwrap();
            let mut cursor = Cursor::new(buffer);

            cursor.set_position(0);
            Content(ContentType::CSV, Stream::from(cursor))
        }
        Err(err) => {
            error!("Could not save xero file {} because: {:?}",&target_path, err);
            let cursor_error = Cursor::new(vec![]);
            Content(ContentType::CSV, Stream::from(cursor_error))
        }

    }





}

fn process_noted_file(p0: &PathBuf, xero_invoice_number: i32 ) -> Vec<XeroType>{
    let noted_contents = read_file(format!("{}",p0.display()));
    let noted_collection = parse_noted_csv(&noted_contents.unwrap());
    let xero_collection = map_noted_to_xero(&noted_collection, Option::from(xero_invoice_number));
    xero_collection
}


fn main() {
    init_logging();
    info!("I am starting");
    rocket::ignite().mount("/", routes![index,noted]).launch();
    info!("I am done");
}