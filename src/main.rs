#[macro_use]
extern crate log;
extern crate hyper;
extern crate rustc_serialize;
extern crate chrono;
extern crate docopt;
extern crate libtempmonitor;

use rustc_serialize::json;
use chrono::*;
use std::env;
use docopt::Docopt;

use libtempmonitor::database::Database;
use libtempmonitor::data_source::FetchData;
use libtempmonitor::data_source::json::JsonSource;
use libtempmonitor::data_source::file::FileSource;
use libtempmonitor::logging;

const USAGE: &'static str = "
Usage:
    tempmonitor [options] <output>
    tempmonitor (--help | --version)

Options:
    -h, --help          Show this message
    --version           Show this version
    -R, --recreate      Recreate the database tables
";

#[derive(RustcDecodable)]
struct Args {
    arg_output: String,
    flag_recreate: bool,
}

fn timestamp_to_dt(timestamp: &str) -> ParseResult<DateTime<UTC>> {
    let timestamp = format!("{} 00:00:00", timestamp);
    UTC.datetime_from_str(&timestamp, "%Y-%m-%dZ %H:%M:%S")
}

fn fetch_json_response() -> String {
    info!("Fetching remote response");
    let api_key = env::var("API_KEY").unwrap();
    let location_id = env::var("LOCATION_ID").unwrap();
    JsonSource::new(api_key, location_id).data()
}

fn fetch_local_response() -> String {
    info!("Fetching local response");
    let filename = "testdata/response.json";
    FileSource::new(&filename).data()
}

fn main() {
    logging::init().unwrap();

    let argv = || env::args();
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv().into_iter()).decode())
        .unwrap_or_else(|e| e.exit());

    let db_name = args.arg_output;
    let recreate = args.flag_recreate;

    info!("Rendering to database \"{}\"", db_name.as_str());
    if recreate {
        info!("Recreating database from scratch");
    }

    let utc: DateTime<UTC> = UTC::now();
    debug!("Current time: {}", &utc);

    let mut database = Database::new(db_name.as_str());
    if recreate {
        debug!("Dropping tables");
        database.drop_tables();
    }
    debug!("Creating tables");
    database.create_tables();

    // let buf = fetch_json_response();
    let buf = fetch_local_response();

    let result = json::Json::from_str(&buf).expect("Error parsing JSON response");
    let entries =
        result["SiteRep"]["DV"]["Location"]["Period"].as_array().expect("Cannot get results");

    for entry in entries {
        let timestamp_str =
            String::from(entry["value"].as_string().expect("Cannot convert to string"));
        let timestamp = timestamp_to_dt(&timestamp_str).expect("Cannot convert to DateTime");
        for report in entry.find("Rep").unwrap().as_array().unwrap() {
            let minutes_after_midnight: i32 =
                report.find("$").unwrap().as_string().unwrap().parse().unwrap();
            let temperature: i64 = report.find("F").unwrap().as_string().unwrap().parse().unwrap();

            let dt = timestamp + Duration::minutes(minutes_after_midnight as i64);

            database.insert(&dt, &temperature, &utc);
        }
    }
}
