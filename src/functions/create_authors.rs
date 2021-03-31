use std::collections::HashMap;

use super::structs::*;

pub fn create_authors(messages: Vec<Message>) -> HashMap<u64, Author> {
    
    let mut authors: HashMap<u64, Author> = HashMap::new();
    
    for message in messages {
        // Get entry for author in question, or create a new author struct from ID
        let author = authors.entry(message.author_id).or_insert(Author::new(message.author_id));

        // Update information
        *author = author.clone().process_message(message);
    }

    return authors;    
}