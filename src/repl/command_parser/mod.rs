pub struct CommandParser {}

impl CommandParser {
    pub fn tokenize(input: &str) -> Vec<&str> {
        let split = input.split_whitespace();
        let vec: Vec<&str> = split.collect();
        vec
    }
}
