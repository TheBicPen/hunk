#[cfg(test)]
mod tests {
    use crate::{parse_args::parse_args, process_lines};
    use std::{fs, io::BufReader};

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

    #[test]
    fn test_invalid_unicode_1() {
        // file may not actually contain invalid unicode
        let file = fs::File::open("test_data/invalid_unicode.diff").unwrap();
        process_lines("changeLog".to_string(), Box::new(BufReader::new(file)))
    }

    #[test]
    #[ignore = "Invalid unicode handling is not implemented"]
    fn test_invalid_unicode_2() {
        let file = fs::File::open("test_data/invalid_unicode_2.diff").unwrap();
        process_lines("changeLog".to_string(), Box::new(BufReader::new(file)))
    }

    #[test]
    #[ignore = "Empty file section handling is not implemented"]
    fn test_empty_file_section() {
        let file = fs::File::open("test_data/empty_file_section.diff").unwrap();
        process_lines("mingw".to_string(), Box::new(BufReader::new(file)))
    }

    #[test]
    fn test_parse_args() {
        let config = parse_args(&vec!["asd"]);
        assert_eq!(config.search_string, "asd");
    }

    #[test]
    #[should_panic]
    fn test_parse_extra_positional() {
        parse_args(&vec!["asd", "qwe"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_no_program_name() {
        parse_args(&vec![]);
    }

    #[test]
    fn test_parse_match_fields() {
        let config = parse_args(&vec!["asd", "--match-fields", "diff,context"]);
        assert_eq!(config.search_string, "asd");
        assert_eq!(config.match_on.diff, true);
        assert_eq!(config.match_on.context, true);
        assert_eq!(config.match_on.file_header, false);
        assert_eq!(config.match_on.patch_header, false);
    }

    #[test]
    #[should_panic]
    fn test_parse_match_fields_repeat_positional_after() {
        parse_args(&vec!["asd", "--match-fields", "diff,context", "qwe"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_match_on_invalid() {
        parse_args(&vec!["asd", "--match-on", "qwe"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_help() {
        parse_args(&vec!["asd", "-h"]);
    }
}
