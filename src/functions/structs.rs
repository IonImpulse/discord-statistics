use chrono::NaiveDateTime;
use std::collections::HashMap;
use csv::StringRecord;

pub const DATE_FORMAT: &str = "%d-%b-%y %I:%M %p";

#[derive(Clone)]
pub struct Author {
    // Numerical Author ID
    pub id: u64,
    // List of all discord tags associated with the ID
    pub names: Vec<String>,
    // Total Message Count
    pub message_count: u128,
    // Total Word Count
    pub word_count: u128,
    // Total Character Count
    pub character_count: u128,
    // Total Number of questions asked
    pub question_count: u128,
    // Times they have been in the majority in polls
    pub times_majority: u128,
    // Times they have been in the minority in polls
    pub times_minority: u128,
    // Ledger of timestamps for each of their messages
    pub time_ledger: Vec<NaiveDateTime>,
    // Ledger of all attachments they have sent
    pub attachments_ledger: Vec<String>,
    // Dictionary of their vocabulary
    pub vocab_dict: HashMap<String,u128>,
    // A hashmap of who they have agreed with in polls
    pub agreement_dict: HashMap<String,u128>,
}

impl Author {
    pub fn new(id: u64) -> Author {
        return Author {
            id: id,
            names: Vec::new(),
            message_count: 0,
            word_count: 0,
            character_count: 0,
            question_count: 0,
            times_majority: 0,
            times_minority: 0,
            time_ledger: Vec::new(),
            attachments_ledger: Vec::new(),
            vocab_dict: HashMap::new(),
            agreement_dict: HashMap::new(),
        }
    }

    pub fn process_message(mut self, mut msg: Message) -> Self {
        // Add to all known names
        if !self.names.contains(&msg.author_name) {
            self.names.push(msg.author_name.clone());
        }

        let word_list: Vec<&str> = msg.content.split(" ").collect();

        // Update counters
        self.message_count += 1;

        self.word_count += word_list.len() as u128;

        self.character_count += msg.content.len() as u128;

        if msg.content.contains("?") { self.question_count += 1; }

        // Add to time ledger
        self.time_ledger.push(msg.date);

        // Add to attachment ledger
        self.attachments_ledger.append(&mut msg.attachments);

        // Add to vocab dict
        for word in word_list {
            let vocab = self.vocab_dict.entry(String::from(word)).or_insert(0);
            *vocab += 1;
        }

        return self;
    }

    pub fn merge(mut self, mut other: Author) -> Self {
        // Add to all known names

        for name in other.names.clone() {
            if !self.names.contains(&name) {
                self.names.push(name);
            }
        }

        // Update counters
        self.message_count += other.message_count;

        self.word_count += other.word_count;

        self.character_count += other.character_count;

        self.question_count += other.question_count;

        // Add to time ledger
        self.time_ledger.append(&mut other.time_ledger);

        // Add to attachment ledger
        self.attachments_ledger.append(&mut other.attachments_ledger);

        // Add to vocab dict
        for (key, value) in other.vocab_dict {
            let vocab = self.vocab_dict.entry(key).or_insert(0);
            *vocab += value;
        }

        return self;
    }
    pub fn print_stats(self) {
        println!("ID: {}", self.id);
        println!("Name(s): {:?}", self.names);
        println!("Messages: {} Words: {} Characters: {}", self.message_count, self.word_count, self.character_count);
        println!("Attachments: {}", self.attachments_ledger.len());
        println!("Vocabulary: {}", self.vocab_dict.len());
        println!("Time Ledger Length: {}", self.time_ledger.len());
    }
}

#[derive(Clone)]
pub struct Message {
    pub author_id: u64,
    pub author_name: String,
    pub date: NaiveDateTime,
    pub content: String,
    pub attachments: Vec<String>,
    pub reactions: HashMap<String, u128>,
}

impl Message {
    pub fn from_csv_string(record: StringRecord) -> Message {
        let author_id: u64 = record.get(0).unwrap().parse().unwrap();
        let author_name: String = String::from(record.get(1).unwrap());
        let date: NaiveDateTime = NaiveDateTime::parse_from_str(record.get(2).unwrap(), DATE_FORMAT).unwrap();
        let content: String = String::from(record.get(3).unwrap());
        let mut attachments: Vec<String> = Vec::new();
        
        if record.get(4).unwrap().len() > 5 {
            let attachments_split = record.get(4).unwrap().split(',');
        
        
            for s in attachments_split {
                attachments.push(String::from(s));
            }
        } 
        

        let reactions_string = record.get(5).unwrap();
        let mut reactions: HashMap<String, u128> = HashMap::new();

        if reactions_string.len() > 5 {
            let reactions_split = reactions_string.split(',');

            for s in reactions_split {
                let reaction_key_value = s.split(' ').collect::<Vec<&str>>();

                reactions.insert(String::from(reaction_key_value[0]), 
                                reaction_key_value[1].replace("(", "").replace(")", "").parse().unwrap_or_default());

            }
        }

        return Message{
            author_id: author_id,
            author_name: author_name,
            date: date,
            content: content,
            attachments: attachments,
            reactions: reactions,
        }
    }
}