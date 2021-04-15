use chrono::*;
use csv;
use csv::Writer;
use std::collections::HashMap;
use std::error::*;

use plotly::common::{TickFormatStop, Title};
use plotly::layout::{Axis, RangeSelector, RangeSlider, SelectorButton, SelectorStep, StepMode};
use plotly::{Candlestick, ImageFormat, Layout, Ohlc, Plot, Scatter};
use sanitize_filename;

use super::structs::*;

pub fn export_author(
    path: &String,
    sorted_server_words: Vec<(&String, &u128)>,
    author: Author,
) -> Result<(), csv::Error> {
    let author_name = &author.names[0];

    let path_to_export = format!("{}{}.csv", path, sanitize_filename::sanitize(author_name));

    let mut wtr = Writer::from_path(path_to_export)?;

    // Start all of the tedious data labeling and exporting...

    wtr.write_record(&["Statistics for:", author_name.as_str(), "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&[
        "Total Messages:",
        "Total Words:",
        "Total Characters:",
        "Total Attachments:",
        "Total Questions:",
        "Total Vocabulary:",
    ])?;
    // Write basic stats
    wtr.write_record(author.clone().return_stats())?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Top 50 Words not in Server Top 50", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;

    let mut author_words: Vec<(&String, &u128)> = author.vocab_dict.iter().collect();

    // Do not sort in reverse, as we'll be poping each element
    // from the end of the stack
    author_words.sort_by(|a, b| a.1.cmp(b.1));

    let num_words;

    if author_words.len() > 50 {
        num_words = 50;
    } else {
        num_words = author_words.len();
    }

    for index in 0..num_words {
        let mut found = false;
        let mut word_found: (&String, &u128) = (&"".to_string(), &0);

        while found == false && author_words.len() > 0 {
            let temp_word_found = author_words.pop();

            if temp_word_found.is_some() {
                found = true;
                word_found = temp_word_found.unwrap();

                for j in 0..50 {
                    if word_found.0 == sorted_server_words[j].0 {
                        found = false;
                    }
                }
            }
        }

        wtr.write_record(&[
            &format!("{}:", index + 1),
            &word_found.0,
            &word_found.1.to_string(),
            &"".to_string(),
            &"".to_string(),
            &"".to_string(),
        ])?;
    }
    wtr.flush()?;

    Ok(())
}

pub fn export_time_graph(
    title: &String,
    path: &String,
    author: Author,
) -> Result<(), Box<dyn Error>> {
    let mut time_range: Vec<NaiveTime> = Vec::new();
    let mut num_messages: Vec<u128> = Vec::new();

    for hr in 0..24 {
        for mn in 0..60 {
            time_range.push(NaiveTime::from_hms(hr, mn, 0));
            num_messages.push(0);
        }
    }

    let mut max: u128 = 0;

    for (point, _) in author.time_ledger {
        let index: usize = (point.minute() + (point.hour() * 60)) as usize;
        num_messages[index] += 1;

        if num_messages[index] > max {
            max = num_messages[index];
        }
    }

    let output_path = format!(
        "{}{}-timemap.html",
        path.clone(),
        sanitize_filename::sanitize(title.clone())
    );
    let mut plot = Plot::new();
    let trace = Scatter::new(time_range, num_messages);
    plot.add_trace(trace);

    let layout = Layout::new()
        .x_axis(Axis::new().range_slider(RangeSlider::new().visible(true)))
        .title(Title::new(title));
    plot.set_layout(layout);

    // Uncomment line below to show plot when exporting
    //plot.show();

    plot.to_html(output_path);

    Ok(())
}

pub fn export_channel_graph(
    title: &String,
    path: &String,
    author: Author,
    channel_id_dict: HashMap<u64, String>,
) -> Result<(), Box<dyn Error>> {
    let output_path = format!(
        "{}{}-timemap.html",
        path.clone(),
        sanitize_filename::sanitize(title.clone())
    );
    let mut plot = Plot::new();

    for (channel_id, channel_name) in channel_id_dict {
        let mut time_range: Vec<NaiveTime> = Vec::new();
        let mut num_messages: Vec<u128> = Vec::new();

        for hr in 0..24 {
            for mn in 0..60 {
                time_range.push(NaiveTime::from_hms(hr, mn, 0));
                num_messages.push(0);
            }
        }

        let mut max: u128 = 0;

        for (point, id) in author.time_ledger.clone() {
            if id == channel_id {
                let index: usize = (point.minute() + (point.hour() * 60)) as usize;
                num_messages[index] += 1;

                if num_messages[index] > max {
                    max = num_messages[index];
                }
            }
        }
        let trace = Scatter::new(time_range, num_messages).name(&channel_name);
        plot.add_trace(trace);
    }

    let layout = Layout::new()
        .x_axis(Axis::new().range_slider(RangeSlider::new().visible(true)))
        .title(Title::new(title));
    plot.set_layout(layout);

    // Uncomment line below to show plot when exporting
    //plot.show();

    plot.to_html(output_path);

    Ok(())
}

pub fn export_channel_timemap_graph(
    title: &String,
    path: &String,
    author: Author,
    channel_id_dict: HashMap<u64, String>,
) -> Result<(), Box<dyn Error>> {
    let output_path = format!(
        "{}{}-timemap.html",
        path.clone(),
        sanitize_filename::sanitize(title.clone())
    );
    let mut plot = Plot::new();
    let layout = Layout::new()
        .x_axis(Axis::new().range_slider(RangeSlider::new().visible(true)))
        .title(Title::new(title));
    plot.set_layout(layout);

    // Uncomment line below to show plot when exporting
    //plot.show();

    plot.to_html(output_path);
    Ok(())
}
pub fn export_all(
    path: &String,
    author_path: &String,
    graph_path: &String,
    server: Author,
    author_hashmap: HashMap<u64, Author>,
    channel_id_dict: HashMap<u64, String>,
) -> Result<(), csv::Error> {
    let path_to_export = format!("{}Server Statistics.csv", path);

    let mut wtr = Writer::from_path(path_to_export)?;

    // Start all of the tedious data labeling and exporting...
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Statistics for server:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&[
        "Total Messages:",
        "Total Words:",
        "Total Characters:",
        "Total Attachments:",
        "Total Questions:",
        "Total Vocabulary:",
    ])?;

    // Write basic stats
    wtr.write_record(server.clone().return_stats())?;

    // Write out list of everyone
    wtr.write_record(&["Members of server:", "", "", "", "", ""])?;
    for (_, author) in &author_hashmap {
        let record_to_write = format!("{:?}", author.names);
        wtr.write_record(&[
            author.id.to_string(),
            record_to_write,
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ])?;
    }

    // Write out ranking lists
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Message Count Ranking:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    let mut message_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    message_count.sort_by(|a, b| b.1.message_count.cmp(&a.1.message_count));

    for item in message_count {
        wtr.write_record(&[
            &item.1.names[0],
            &item.1.message_count.to_string(),
            "",
            "",
            "",
            "",
        ])?;
    }
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Word Count Ranking:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    let mut word_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    word_count.sort_by(|a, b| b.1.word_count.cmp(&a.1.word_count));

    for item in word_count {
        wtr.write_record(&[
            &item.1.names[0],
            &item.1.word_count.to_string(),
            "",
            "",
            "",
            "",
        ])?;
    }
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Character Count Ranking:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    let mut character_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    character_count.sort_by(|a, b| b.1.character_count.cmp(&a.1.character_count));

    for item in character_count {
        wtr.write_record(&[
            &item.1.names[0],
            &item.1.character_count.to_string(),
            "",
            "",
            "",
            "",
        ])?;
    }
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Attachment Count Ranking:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    let mut attachments_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    attachments_count.sort_by(|a, b| {
        b.1.attachments_ledger
            .len()
            .cmp(&a.1.attachments_ledger.len())
    });

    for item in attachments_count {
        wtr.write_record(&[
            &item.1.names[0],
            &item.1.attachments_ledger.len().to_string(),
            "",
            "",
            "",
            "",
        ])?;
    }
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Question Count Ranking:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    let mut question_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    question_count.sort_by(|a, b| b.1.question_count.cmp(&a.1.question_count));

    for item in question_count {
        wtr.write_record(&[
            &item.1.names[0],
            &item.1.question_count.to_string(),
            "",
            "",
            "",
            "",
        ])?;
    }
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Vocabulary Count Ranking:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    let mut vocab_count: Vec<(&u64, &Author)> = author_hashmap.iter().collect();
    vocab_count.sort_by(|a, b| b.1.vocab_dict.len().cmp(&a.1.vocab_dict.len()));

    for item in vocab_count {
        wtr.write_record(&[
            &item.1.names[0],
            &item.1.vocab_dict.len().to_string(),
            "",
            "",
            "",
            "",
        ])?;
    }

    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    wtr.write_record(&["Top 1000 Words:", "", "", "", "", ""])?;
    wtr.write_record(&["-----------------------------", "", "", "", "", ""])?;
    let mut all_words: Vec<(&String, &u128)> = server.vocab_dict.iter().collect();
    all_words.sort_by(|a, b| b.1.cmp(a.1));

    let num_words;

    if all_words.len() > 1000 {
        num_words = 1000;
    } else {
        num_words = all_words.len();
    }

    for index in 0..num_words {
        wtr.write_record(&[
            &format!("{}:", index + 1),
            &all_words[index].0,
            &all_words[index].1.to_string(),
            &"".to_string(),
            &"".to_string(),
            &"".to_string(),
        ])?;
    }

    // Write buffer to file
    wtr.flush()?;

    let mut count = 0;

    for (_, value) in author_hashmap {
        count += 1;

        eprintln!("{}: Exporting {}...", count, value.names[0]);

        let csv_result = export_author(author_path, all_words.clone(), value.clone());

        if csv_result.is_err() {
            eprintln!("- Error: Could not export csv! {}", csv_result.unwrap_err());
        } else {
            eprintln!("- Exported csv successfully!");
        }

        let title = format!("Time Map for {}", value.names[0]);

        let graph_result = export_time_graph(&title, graph_path, value.clone());

        if graph_result.is_err() {
            eprintln!(
                "- Error: Could not export graph! {}",
                graph_result.unwrap_err()
            );
        } else {
            eprintln!("- Exported graph successfully!");
        }
    }

    // Author exporting is done, now time for server graphs!

    let server_timemap_graph_result =
        export_time_graph(&"Server Time Graph".to_string(), path, server.clone());

    if server_timemap_graph_result.is_err() {
        println!("{}", server_timemap_graph_result.unwrap_err());
    }

    let server_channel_graph_result = export_channel_graph(
        &"Channel Time Graph".to_string(),
        path,
        server.clone(),
        channel_id_dict.clone(),
    );

    if server_channel_graph_result.is_err() {
        println!("{}", server_channel_graph_result.unwrap_err());
    }

    let server_channel_timemap_graph_result = export_channel_timemap_graph(
        &"Channel Timemap Graph".to_string(),
        path,
        server.clone(),
        channel_id_dict.clone(),
    );

    if server_channel_timemap_graph_result.is_err() {
        println!("{}", server_channel_timemap_graph_result.unwrap_err());
    }
    Ok(())
}
