pub mod functions;

use functions::*;

use chrono::*;
use csv::Reader;
use std::collections::*;
use std::env;
use std::fs;
use std::thread;
use std::time;
use num_cpus;

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

    let mut SOURCE_PATH: &str = "";
    let mut EXPORT_PATH: &str = "";
    let mut SCRAPE_ATTACHMENTS = false;
    let mut POLLS_PATH: &str = "";
    let mut PROCESS_WORDS: Vec<&str>;

    if &arguments.len() == &1 {
        SOURCE_PATH = arguments[0].as_str();
        EXPORT_PATH = SOURCE_PATH.clone();
    } else if &arguments.len() == &2 {
        SOURCE_PATH = arguments[1].as_str();
        EXPORT_PATH = SOURCE_PATH.clone();
    } else {
        let s_flag = arguments.iter().position(|r| r == "-s");
        let e_flag = arguments.iter().position(|r| r == "-e");
        let i_flag = arguments.iter().position(|r| r == "-i");
        let p_flag = arguments.iter().position(|r| r == "-p");
        let t_flag = arguments.iter().position(|r| r == "-t");

        if let Some(value) = s_flag {
            if value < arguments.len() {
                SOURCE_PATH = arguments[value + 1].as_str();
                EXPORT_PATH = SOURCE_PATH.clone();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }
        if let Some(value) = e_flag {
            if value < arguments.len() {
                EXPORT_PATH = arguments[value + 1].as_str();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }

        if let Some(value) = p_flag {
            if value < arguments.len() {
                POLLS_PATH = arguments[value + 1].as_str();
            } else {
                println!("Error in path arguments!");
                panic!();
            }
        }

        if let Some(value) = t_flag {
            if value < arguments.len() {
                PROCESS_WORDS = arguments[value + 1]
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
                SCRAPE_ATTACHMENTS = true;
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
    let paths = fs::read_dir(SOURCE_PATH).unwrap();

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
                threads.push(thread::spawn(move || scrape_file::scrape_file(path)));
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
        threads.push(thread::spawn(move || create_authors::create_authors(workload)));
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

    eprint!("Done! in {} ms\n", start.elapsed().as_millis());
    
}
