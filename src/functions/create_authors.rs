use std::collections::HashMap;

use super::structs::*;

pub fn create_authors(messages: Vec<Message>) -> HashMap<u64, Author> {
    
    let mut authors: HashMap<u64, Author> = HashMap::new();
    
    // Create a "server author" to count everything
    let mut server_author: Author = Author::new(0);

    for message in messages {
        // Get entry for author in question, or create a new author struct from ID
        let author = authors.entry(message.author_id).or_insert(Author::new(message.author_id));

        // Update information
        *author = author.clone().process_message(message.clone());

        // Process every message for the server as a server author
        server_author = server_author.process_message(message);
    }

    authors.insert(0, server_author);
    
    return authors;    
}