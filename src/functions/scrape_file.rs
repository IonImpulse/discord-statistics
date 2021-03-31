use std::fs::*;
use csv::*;

use super::structs::*;

pub fn scrape_file(csv_path: DirEntry) -> Vec<Message> {
    let mut rdr = Reader::from_path(csv_path.path().as_path()).unwrap();
    let mut message_vec: Vec<Message> = Vec::new();

    for result in rdr.records() {
        message_vec.push(Message::from_csv_string(result.unwrap()));
    }

    return message_vec;
}