#![feature(proc_macro_hygiene, decl_macro)]

use log::{debug, error, info, warn};
use noted2xero_core::n2x_core::parse_noted_csv;
use noted2xero_core::n2x_core::read_file;
use noted2xero_core::n2x_core::{init_logging, map_noted_to_xero};
use rocket::fairing::AdHoc;

#[macro_use]
extern crate rocket;
extern crate rocket_multipart_form_data;
use chrono::{DateTime, Duration, Utc};
use noted2xero_core::xero::XeroType;
use rocket::http::{ContentType, Header};
use rocket::response::NamedFile;
use rocket::Data;
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::path::Path;
use uuid::Uuid;

#[get("/healthcheck")]
fn index() -> String {
    info!("Got an incoming request :: healthcheck");
    "Hello, world!".to_string()
}

#[post("/noted", data = "<data>")]
fn noted(data: Data, content_type: &ContentType) -> Option<NamedFile> {
    info!("Got an incoming request :: noted");
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("data")
            .content_type_by_string(Some(format!("{}/{}", mime::TEXT, mime::CSV)))
            .unwrap(),
        MultipartFormDataField::text("text"),
    ]);

    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options).unwrap();
    let noted_section = multipart_form_data.files.get("data").unwrap();

    let file_fields = noted_section;
    let dataset = &file_fields[0];
    info!("Read from data set: {:?}", dataset);
    let local_path = &dataset.path;
    let start_invoice_number = multipart_form_data.texts.remove("text");
    let invoice_number;
    match start_invoice_number {
        None => {
            warn!("Could not parse the invoice number, defaults to 0");
            invoice_number = 0;
        }
        Some(mut val) => {
            let v = val.remove(0);
            invoice_number = v.text.parse::<i32>().unwrap_or(0);
            debug!("Parsed invoice number into INV-{}", invoice_number);
        }
    }
    let xero_data = process_noted_file(local_path, invoice_number);
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
            NamedFile::open(&target_path).ok()
        }
        Err(err) => {
            error!(
                "Could not save xero file {} because: {:?}",
                &target_path, err
            );
            NamedFile::open(target_path).ok()
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
    rocket::ignite()
        .attach(AdHoc::on_response("Put header", |req, mut res| {
            info!("I am in the fairing on_response");
            let path = req.uri();
            info!("Value of the path: {}", path.path().to_string());

            if path.path() == "/noted" {
                info!("Starting to enrich the noted path processing part.");
                let current_time = Utc::now() + Duration::hours(13);
                let date_format = current_time.format("%Y%m%d_%H%M%S");
                res.set_header(Header::new(
                    "Content-Disposition",
                    format!(
                        "attachment; filename=\"xero_import_candidate_{}.csv\"",
                        date_format
                    ),
                ));
            }
        }))
        .mount("/", routes![index, noted])
        .launch();
    info!("I am done");
}
