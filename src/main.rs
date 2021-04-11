#![feature(proc_macro_hygiene, decl_macro)]

use noted2xero_core::n2x_core::{init_logging, map_noted_to_xero};
use noted2xero_core::n2x_core::read_file;
use noted2xero_core::n2x_core::parse_noted_csv;

use log::{info};


use rocket::http::{RawStr};


#[macro_use] extern crate rocket;
extern crate rocket_multipart_form_data;
use rocket::Data;
use rocket::http::ContentType;
use rocket_multipart_form_data::{mime, MultipartFormDataOptions, MultipartFormDataField, MultipartFormData};
use std::path::PathBuf;
use noted2xero_core::xero::XeroType;
use rocket::response::Stream;
use csv::WriterBuilder;


#[get("/healthcheck")]
fn index() -> String {
    info!("Got an incoming request :: healthcheck");
    let retval = format!("Hello, world!");
    retval
}

#[post("/noted/<start_invoice_number>", data = "<data>")]
fn noted( start_invoice_number: &RawStr, data: Data, content_type: &ContentType) -> String {
    info!("Got an incoming request :: noted");
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            MultipartFormDataField::file("data").content_type_by_string(Some(format!("{}/{}",mime::TEXT, mime::CSV))).unwrap(),
        ]
    );
    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options).unwrap();
    let noted_section = multipart_form_data.files.get("data").unwrap();

    let file_fields = noted_section;
    let dataset = &file_fields[0];
    info!("Read from data set: {:?}", dataset);
    let local_path = &dataset.path;

    let xero_data = process_noted_file(local_path, start_invoice_number.parse::<i32>().unwrap_or(0) );
    let mut writer = WriterBuilder::new()
        .flexible(true)
        .from_writer(xero_data);;


    let headers = XeroType::get_headers();
    let mut return_string = "".to_string();

    return_string = headers.join(",");

    return_string



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