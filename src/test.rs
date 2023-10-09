
#[cfg(test)]
mod tests {
    use std::{fs, io::BufReader};
    use crate::process_lines;

    #[test]
    fn test_1() {
        let file = fs::File::open("test_data/1.diff").unwrap();
        process_lines("AIPlayer".to_string(), Box::new(BufReader::new(file)))
    }

    #[test]
    fn test_1_color() {
        let file = fs::File::open("test_data/1_color.diff").unwrap();
        process_lines("AIPlayer".to_string(), Box::new(BufReader::new(file)))
    }

}