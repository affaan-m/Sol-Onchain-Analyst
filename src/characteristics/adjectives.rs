use std::fs;
use std::io;

use crate::core::characteristics::Characteristic;

pub struct Adjectives;

impl Characteristic for Adjectives {
    fn get_header(&self) -> String {
        "These are the adjectives.".to_string()
    }

    fn get_traits(&self, character_name: &str) -> io::Result<String> {
        let path = format!("./characters/{}/adjectives.txt", character_name);
        fs::read_to_string(&path)
    }
}
