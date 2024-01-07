use std::{env::Args, collections::HashMap};

#[derive(Default)]
pub enum UTF8Strategy {
    #[default]
    Panic,
    Lossy,
    SkipLine
}

#[derive(Default)]
pub struct PatchSections {
    pub diff: bool,
    pub context: bool,
    pub file_header: bool,
    pub patch_header: bool,
}

#[derive(Default)]
pub struct Config {
    pub match_on: PatchSections,
    pub print_sections: PatchSections,
    pub search_string: String,
    pub decode_strategy: UTF8Strategy
}

fn parse_patch_sections(input: &str) -> PatchSections {
    let mut sections = PatchSections {
        diff: false,
        context: false,
        file_header: false,
        patch_header: false,
    };
    for section_str in input.split(',') {
        match section_str {
            "diff" => sections.diff = true,
            "context" => sections.context = true,
            "file_header" => sections.file_header = true,
            "patch_header" => sections.patch_header = true,
            other => panic!("Unknown patch section '{}'. Run `hunk -h` for help", other),
        }
    }

    sections
}

fn print_help() {
    struct HelpData {
        two_arg_params: HashMap<&'static str, &'static str>,
        one_arg_params: HashMap<&'static str, &'static str>,
        positional_params: HashMap<&'static str, &'static str>,
    }

    let help_data = HelpData {
        two_arg_params: HashMap::from([
            ("--match-fields", "Which fields of the patch to search for the string. Takes a comma-separated list of values. Valid values are 'diff', 'context', 'file_header', and 'patch_header'"),
            ("--print-fields", "Which fields of the patch to print to stdout when a match is found. Takes a comma-separated list of values. Valid values are 'diff', 'context', 'file_header', and 'patch_header'"),
            ("--invalid-utf8", "How to handle invalid UTF-8 lines. Specify one of 'lossy', 'panic', or 'skip-line'")
        ]),
        one_arg_params: HashMap::from([
            ("--help, -h", "Show this message and exit")
        ]),
        positional_params: HashMap::from([
            ("PATTERN", "The string to search for")
        ]),
    };
    println!("Usage: hunk PATTERN [OPTION...]");
    println!("");
    for (k, v) in help_data.positional_params {
        println!("{:15}: {}", k, v)
    }
    println!("");
    println!("OPTIONS:");
    for (k, v) in help_data.one_arg_params {
        println!("{:15}: {}", k, v)
    }
    for (k, v) in help_data.two_arg_params {
        println!("{:15}: {}", k, v)
    }
    panic!();
}

pub fn parse_args(args: &[&str]) -> Config {
    struct ParsingState {
        has_search_string: bool,
    }

    fn parse_slice(args: &[&str], state: &mut ParsingState, config: &mut Config) {
        match &args {
            ["--match-fields", match_fields, rest @ ..] => {
                config.match_on = parse_patch_sections(match_fields);
                parse_slice(rest, state, config);
            }
            ["--match-fields"] => panic!("Expected argument for 'match-fields'. Run `hunk -h` for help"),

            ["--print-fields", print_fields, rest @ ..] => {
                config.print_sections = parse_patch_sections(print_fields);
                parse_slice(rest, state, config);
            }
            ["--print-fields"] => panic!("Expected argument for 'print-fields'. Run `hunk -h` for help"),
            ["--invalid-utf8", decode_strategy_str, rest @ ..] => {
                config.decode_strategy = match decode_strategy_str {
                    &"lossy" => UTF8Strategy::Lossy,
                    &"panic" => UTF8Strategy::Panic,
                    &"skip-line" => UTF8Strategy::SkipLine,
                    other => panic!("Unknown value '{}'. Run `hunk -h` for help", other)
                };
                parse_slice(rest, state, config);
            }
            ["--invalid-utf8"] => panic!("Expected argument for 'invalid-utf8'. Run `hunk -h` for help"),
            ["--help"] | ["-h"] => print_help(),

            [arg, rest @ ..] if !state.has_search_string => {
                config.search_string = arg.to_string();
                state.has_search_string = true;
                parse_slice(rest, state, config);
            }
            [arg, ..] => panic!("Unexpected arg: {}. Run `hunk -h` for help", arg),
            [] if !state.has_search_string => panic!("Expected a string to search for. Run `hunk -h` for help"),
            [] => {}
        }
    }

    let mut config = Config {
        decode_strategy: UTF8Strategy::Panic,
        match_on: PatchSections {
            diff: true,
            context: false,
            file_header: false,
            patch_header: false,
        },
        print_sections: PatchSections {
            diff: false,
            context: false,
            file_header: false,
            patch_header: true,
        },
        search_string: "".to_string(),
    };
    let mut parsing_state = ParsingState {
        has_search_string: false,
    };
    parse_slice(args, &mut parsing_state, &mut config);

    config
}

pub fn parse_program_args(args: &mut Args) -> Config {
    let args_strings: Vec<String> = args.collect();
    let args: Vec<&str> = args_strings.iter().map(|s| s.as_str()).collect();

    parse_args(&args[1..])
}
