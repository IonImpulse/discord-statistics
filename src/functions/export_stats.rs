use std::error::*;
use csv;
use csv::Writer;
use std::collections::HashMap;
use plotters::prelude::*;
use chrono::*;

use super::structs::*;

pub fn export_author(path: &String, author: Author) -> Result<(), csv::Error> {
    
    let author_name = &author.names[0];

    let path_to_export = format!("{}{}.csv", path, author_name);

    let mut wtr = Writer::from_path(path_to_export)?;

    // Start all of the tedious data labeling and exporting...

    wtr.write_record(&["Statistics for:", author_name.as_str(), "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Total Messages:", "Total Words:", "Total Characters:", "Total Attachments:", "Total Questions:", "Total Vocabulary:"])?;

    // Write basic stats
    wtr.write_record(author.return_stats())?;

    wtr.flush()?;

    Ok(())
}

pub fn export_time_graph(title: &String, path: &String, author: Author) -> Result<(), Box<dyn Error>> {
    let mut time_ledger: Vec<(String, f32)> = Vec::new();
    let mut string_ledger: Vec<String> = Vec::new();

    for hr in 0..24 {
        for mn in 0..60 {
            string_ledger.push(format!("{}:{}", hr, mn));
            time_ledger.push((format!("{}:{}", hr, mn),0_f32));
        } 
    }

    let mut max: f32 = 0.;

    for point in author.time_ledger {
        let index: usize = (point.minute() + (point.hour() * 60)) as usize;
        time_ledger[index].1 += 1.;

        if time_ledger[index].1 > max { max = time_ledger[index].1; }
    }

    let root =
        BitMapBackend::new(format!("{}{}-timemap.png", path, title).as_str(), (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(
            "Messages Sent by Minute",
            ("sans-serif", 40),
        )
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(
            (NaiveTime::from_hms(0,0,0)..NaiveTime::from_hms(23,59,59)),
            0.0..max,
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(48)
        .y_desc("Messages sent")
        .draw()?;

    chart.draw_series(LineSeries::new(
        DATA.iter().map(|(y, m, t)| (Utc.ymd(*y, *m, 1), *t)),
        &BLUE,
    ))?;

    Ok(())
}

pub fn export_all(path: &String, author_path: &String, graph_path: &String, server: Author, author_hashmap: HashMap<u64, Author>) -> Result<(), csv::Error> {

    let path_to_export = format!("{}Server Statistics.csv", path);

    let mut wtr = Writer::from_path(path_to_export)?;

    // Start all of the tedious data labeling and exporting...

    wtr.write_record(&["Statistics for server:", "", "", "", "", "",])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Total Messages:", "Total Words:", "Total Characters:", "Total Attachments:", "Total Questions:", "Total Vocabulary:"])?;

    // Write basic stats
    wtr.write_record(server.clone().return_stats())?;

    // Write out list of everyone
    wtr.write_record(&["Members of server:", "", "", "", "", ""])?;
    for (_, author) in &author_hashmap {
        let record_to_write = format!("{:?}", author.names);
        wtr.write_record(&[author.id.to_string(), record_to_write, "".to_string(), "".to_string(), "".to_string(), "".to_string()])?;
    }

    // Write out ranking lists
    wtr.write_record(&["Message Count Ranking:", "", "", "", "", ""])?;

    let mut message_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    message_count.sort_by(|a, b| b.1.message_count.cmp(&a.1.message_count));

    for item in message_count {
        wtr.write_record(&[&item.1.names[0], &item.1.message_count.to_string(), "", "", "", ""])?;
    }

    wtr.write_record(&["Word Count Ranking:", "", "", "", "", ""])?;

    let mut word_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    word_count.sort_by(|a, b| b.1.word_count.cmp(&a.1.word_count));

    for item in word_count {
        wtr.write_record(&[&item.1.names[0], &item.1.word_count.to_string(), "", "", "", ""])?;
    }

    wtr.write_record(&["Character Count Ranking:", "", "", "", "", ""])?;

    let mut character_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    character_count.sort_by(|a, b| b.1.character_count.cmp(&a.1.character_count));

    for item in character_count {
        wtr.write_record(&[&item.1.names[0], &item.1.character_count.to_string(), "", "", "", ""])?;
    }

    wtr.write_record(&["Attachment Count Ranking:", "", "", "", "", ""])?;

    let mut attachments_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    attachments_count.sort_by(|a, b| b.1.attachments_ledger.len().cmp(&a.1.attachments_ledger.len()));

    for item in attachments_count {
        wtr.write_record(&[&item.1.names[0], &item.1.attachments_ledger.len().to_string(), "", "", "", ""])?;
    }

    wtr.write_record(&["Question Count Ranking:", "", "", "", "", ""])?;

    let mut question_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    question_count.sort_by(|a, b| b.1.question_count.cmp(&a.1.question_count));

    for item in question_count {
        wtr.write_record(&[&item.1.names[0], &item.1.question_count.to_string(), "", "", "", ""])?;
    }

    wtr.write_record(&["Vocabulary Count Ranking:", "", "", "", "", ""])?;

    let mut vocab_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    vocab_count.sort_by(|a, b| b.1.vocab_dict.len().cmp(&a.1.vocab_dict.len()));

    for item in vocab_count {
        wtr.write_record(&[&item.1.names[0], &item.1.vocab_dict.len().to_string(), "", "", "", ""])?;
    }

    wtr.flush()?;

    for (_, value) in author_hashmap {
        let result = export_author(author_path, value.clone());

        if result.is_err() {
            eprintln!("Error: Could not export {}!", value.names[0]);
        } else {
            eprintln!("Exported {} successfully!", value.names[0]);
        }
    }


    // CSV exporting is done, now time for graphs!

    let server_graph_result = export_time_graph(&"Time Map".to_string(), path, server);

    if server_graph_result.is_err() {
        println!("{}",  server_graph_result.unwrap_err());
    }
    Ok(())
}