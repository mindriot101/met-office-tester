#![allow(dead_code)]

extern crate hyper;
extern crate rustc_serialize;
extern crate chrono;
extern crate sqlite;

use std::io::Read;
use hyper::Client;
use rustc_serialize::json;
use std::fs::File;
use chrono::*;
use std::env;

struct Database {
    connection: sqlite::Connection,
}

impl Database {
    fn new(path: &str) -> Database {
        Database { connection: sqlite::open(path).unwrap() }
    }

    fn insert(&mut self, dt: &DateTime<UTC>, temperature: &i64) {
        let mut statement = self.connection
            .prepare("insert into predictions (dt, temperature)
            values (?, ?)")
            .unwrap();
        statement.bind(1, dt.timestamp()).unwrap();
        statement.bind(2, *temperature).unwrap();
        loop {
            match statement.next() {
                Ok(sqlite::State::Done) => break,
                _ => (),
            }
        }
    }

    fn drop_tables(&mut self) {
        self.connection.execute("drop table if exists predictions").unwrap();
    }

    fn create_tables(&mut self) {
        self.connection
            .execute("create table if not exists predictions (
            id integer primary key,
            \
                      dt timestamp not null,
            temperature integer not null
        )")
            .unwrap();
    }
}

fn timestamp_to_dt(timestamp: &str) -> ParseResult<DateTime<UTC>> {
    let timestamp = format!("{} 00:00:00", timestamp);
    UTC.datetime_from_str(&timestamp, "%Y-%m-%dZ %H:%M:%S")
}

fn fetch_json_response() -> String {
    let api_key = env::var("API_KEY").unwrap();
    let location_id = env::var("LOCATION_ID").unwrap();
    let client = Client::new();
    let url = format!("http://datapoint.metoffice.gov.\
     uk/public/data/val/wxfcs/all/json/{location_id}?res=3hourly&key={api_key}",
                      location_id = location_id,
                      api_key = api_key);
    let mut response = client.get(url.as_str()).send().expect("Error sending request");
    let mut buf = String::new();
    response.read_to_string(&mut buf).expect("Error reading response string");
    buf
}

fn fetch_local_response() -> String {
    let mut f = File::open("testdata/response.json").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).expect("Cannot read file");
    buf
}

fn main() {
    let mut database = Database::new("db.sqlite");
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

            database.insert(&dt, &temperature);
        }
    }
}
