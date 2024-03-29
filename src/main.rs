pub mod functions;

use functions::*;

use num_cpus;
use regex;
use std::collections::*;
use std::env;
use std::fs;
use std::thread;
use std::time;
use std::io::Write;
use chrono::Datelike;

fn main() {
    /*
    Usage for running on the command line:

    -s [PATH]               set source path (required)
    -e [PATH]               set export path (will default to source path if not given)
    -i                      scrape for attachments
    -p [PATH]               process polls channel when given path
    -t [WORD1, WORD2..]     Process popularity of word[s] over time

    */

    let arguments: Vec<String> = env::args().collect();

    let sep;

    if env::consts::OS == "windows" {
        sep = "\\";
    } else {
        sep = "/"
    }

    let mut source_path: &str = "";
    let mut export_path: &str = "";
    let mut scrape_attachments = false;
    let mut polls_path: &str = "";
    let mut process_words: Vec<&str>;
    let mut start_year: i32 = 0;
    let mut end_year: i32 = 0;

    if &arguments.len() == &1 {
        source_path = arguments[0].as_str();
        export_path = source_path.clone();
    } else if &arguments.len() == &2 {
        source_path = arguments[1].as_str();
        export_path = source_path.clone();
    } else {
        let s_flag = arguments.iter().position(|r| r == "-s");
        let e_flag = arguments.iter().position(|r| r == "-e");
        let i_flag = arguments.iter().position(|r| r == "-i");
        let p_flag = arguments.iter().position(|r| r == "-p");
        let t_flag = arguments.iter().position(|r| r == "-t");
        let start_flag = arguments.iter().position(|r| r == "--start");
        let end_flag = arguments.iter().position(|r| r == "--end");

        if let Some(value) = s_flag {
            if value < arguments.len() {
                source_path = arguments[value + 1].as_str();
                export_path = source_path.clone();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }
        if let Some(value) = e_flag {
            if value < arguments.len() {
                export_path = arguments[value + 1].as_str();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }

        if let Some(value) = p_flag {
            if value < arguments.len() {
                polls_path = arguments[value + 1].as_str();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }

        if let Some(value) = t_flag {
            if value < arguments.len() {
                process_words = arguments[value + 1]
                    .clone()
                    .split(',')
                    .collect::<Vec<&str>>();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }

        if let Some(value) = i_flag {
            if value < arguments.len() {
                scrape_attachments = true;
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }

        if let Some(value) = start_flag {
            if value < arguments.len() {
                start_year = arguments[value + 1]
                    .clone()
                    .parse::<i32>()
                    .unwrap();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }

        if let Some(value) = end_flag {
            if value < arguments.len() {
                end_year = arguments[value + 1]
                    .clone()
                    .parse::<i32>()
                    .unwrap();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }
    }

    // Now that we're done with all of the arguments,
    // we can start to use parallel processing to import
    // and process all of the data

    eprint!("\nImporting data... ");
    let start = time::Instant::now();

    // First, we get all paths
    let paths = fs::read_dir(source_path).unwrap();

    // Setup a dictionary to store the ID of each channel
    // and link it to the path and name
    let mut channel_id_dict: HashMap<u64, String> = HashMap::new();

    // Setup where the threads will send their data
    let mut message_parts: Vec<structs::Message> = Vec::new();
    let mut threads = Vec::new();

    // Go over every path, making sure that it first
    // has an extension, and then seeing if it's a
    // csv file. If so, create a thread to scrape it
    for path in paths {
        let path = path.unwrap();

        if let Some(value) = path.path().extension() {
            if value == "csv" {
                let string_path = String::from(path.path().to_str().unwrap());

                // First, get the channel ID from the file name
                let channel_id_regex = regex::Regex::new(r#"(\[[0-9]{18}\])"#).unwrap();

                let channel_id = channel_id_regex.captures(&string_path);

                if channel_id.is_some() {
                    let channel_id: u64 = channel_id
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .as_str()
                        .replace("[", "")
                        .replace("]", "")
                        .parse::<u64>()
                        .unwrap();

                    let channel_path_words: String = string_path
                        .replace(" ", "")
                        .split("-")
                        .enumerate()
                        .filter(|&(i, _)| i >= 2)
                        .map(|(_, e)| format!("{}-", e))
                        .collect();

                    let channel_name: String = channel_path_words
                        .split("[")
                        .enumerate()
                        .filter(|&(i, _)| i < 1)
                        .map(|(_, e)| e)
                        .collect();

                    channel_id_dict.insert(channel_id.clone(), channel_name);

                    threads.push(thread::spawn(move || {
                        scrape_file::scrape_file(string_path, channel_id)
                    }));
                }
            }
        }
    }

    // Join all the threads together, appending the vecs
    // together into one large one that we can later
    // process into authors
    for thread in threads {
        message_parts.append(&mut thread.join().unwrap());
    }

    eprint!("Done! in {} ms\n", start.elapsed().as_millis());

    // Now that we have imported all of the data, we can
    // evenly distribute the load among all threads
    eprint!("\nProcessing Authors... ");
    let start = time::Instant::now();

    let num_of_threads = num_cpus::get();

    // Setup where the threads will send their data
    let mut thread_workloads: Vec<Vec<structs::Message>> = Vec::with_capacity(num_of_threads);

    for _ in 0..num_of_threads {
        thread_workloads.push(Vec::new());
    }

    let mut threads = Vec::new();
    let mut author_parts: Vec<HashMap<u64, structs::Author>> = Vec::new();

    // Evenly distribute messages between each threadpool
    // while making sure to not unnecessarily copy data
    let all_messages: Vec<structs::Message>;

    if (start_year == 0) && (end_year == 0) {
        all_messages = message_parts.clone();
    }  else {
        all_messages = message_parts
            .iter()
            .filter(|m| {
                let date = m.date;

                let year = date.year();

                year >= start_year && year <= end_year
            }).cloned().collect();
    }

    let mut message_parts = all_messages.clone();

    let mut i = 0;
    while message_parts.len() > 1 {
        thread_workloads[i].push(message_parts.pop().unwrap());
        i += 1;
        if i == num_of_threads {
            i = 0;
        }
    }

    // Go over every path, making sure that it first
    // has an extension, and then seeing if it's a
    // csv file. If so, create a thread to scrape it
    for workload in thread_workloads {
        threads.push(thread::spawn(move || {
            create_authors::create_authors(workload)
        }));
    }

    // Join all the threads together again, and then we
    // can wait to merge all of these together
    for thread in threads {
        author_parts.push(thread.join().unwrap());
    }

    eprint!("Done! in {} ms\n", start.elapsed().as_millis());

    // Now that we have all of the authors in parts, we
    // can consolidate them all into a single HashMap
    eprint!("\nConsolidating Authors... ");
    let start = time::Instant::now();

    let mut master_author_map: HashMap<u64, structs::Author> = HashMap::new();
    for part in author_parts {
        for (key, value) in part {
            let mut temp_author: structs::Author;

            if master_author_map.contains_key(&key) {
                // If the author is already in the master map,
                // get that value and merge it with the other
                // part of the author
                temp_author = master_author_map.get(&key).unwrap().clone();
                temp_author = temp_author.merge(value);
            } else {
                // If the author isn't in the master map,
                // we can simply insert it.
                temp_author = value;
            }
            // Insert it into the map
            master_author_map.insert(key, temp_author);
        }
    }

    // Take out the server author as we don't want to export
    // it along with all of the real authors
    let server_author = master_author_map.remove(&0).unwrap();

    eprint!("Done! in {} ms\n", start.elapsed().as_millis());
    // We are basically done now! Just need to
    // export everything as csv documents,
    // and then generate cool graphs!
    eprint!("\nExporting Stats...\n");
    let start = time::Instant::now();

    // First, create the export directory
    let export_main_dir = format!("{}{}{}{}", export_path, sep, "Discord Stats", sep);
    let authors_dir = format!("{}{}{}", export_main_dir, "Authors", sep);
    let graphs_dir = format!("{}{}{}", export_main_dir, "Graphs", sep);

    // One function will create the base directory and the inner Author dir
    let author_dir_created = fs::create_dir_all(&authors_dir);

    // Next, we can create the graph dir also
    let graph_dir_created = fs::create_dir_all(&graphs_dir);

    // Create the text file of all message for use as a simple dataset
    let mut server_text_file = fs::File::create(format!("{}All_Messages.txt", &export_main_dir)).unwrap();
    
    for msg in all_messages {
        server_text_file.write_all(format!("[{}] {}\n", msg.author_name, msg.content).as_bytes());
    }

    if author_dir_created.is_ok() && graph_dir_created.is_ok() {
        // Export all of the csv and graph files
        let stats_exported = export_stats::export_all(
            &export_main_dir,
            &authors_dir,
            &graphs_dir,
            server_author,
            master_author_map,
            channel_id_dict,
        );

        if stats_exported.is_err() {
            panic!(
                "ERROR: Could not export csv files! {}",
                stats_exported.unwrap_err()
            );
        }
    } else {
        panic!(
            "ERROR: Could not create the export directory! \n{}\n{}",
            author_dir_created.unwrap_err(),
            graph_dir_created.unwrap_err()
        );
    }

    eprint!("Done! in {} ms\n", start.elapsed().as_millis());

    // Now onto the conditionals
}
