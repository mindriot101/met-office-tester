extern crate hyper;

use self::hyper::Client;
use std::io::Read;
use super::FetchData;

pub struct JsonSource {
    api_key: String,
    location_id: String,
    client: Client,
}

impl JsonSource {
    pub fn new(api_key: String, location_id: String) -> JsonSource {
        JsonSource {
            api_key: api_key,
            location_id: location_id,
            client: Client::new(),
        }
    }
}

impl FetchData for JsonSource {
    fn data(&self) -> String {
        let url = format!("http://datapoint.metoffice.gov.\
        uk/public/data/val/wxfcs/all/json/{location_id}?res=3hourly&key={api_key}",
                        location_id = self.location_id,
                        api_key = self.api_key);
        let mut response = self.client.get(url.as_str()).send().expect("Error sending request");
        let mut buf = String::new();
        response.read_to_string(&mut buf).expect("Error reading response string");
        buf
    }
}
