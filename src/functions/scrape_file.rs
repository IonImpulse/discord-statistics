use csv::*;

use super::structs::*;

pub fn scrape_file(string_path: String, channel_id: u64) -> Vec<Message> {
    let mut rdr = Reader::from_path(string_path).unwrap();

    let mut message_vec: Vec<Message> = Vec::new();

    for result in rdr.records() {
        message_vec.push(Message::from_csv_string(result.unwrap(), channel_id));
    }

    return message_vec;
}
