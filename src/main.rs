mod parse_args;
mod test;

use console::strip_ansi_codes;
use parse_args::{parse_program_args, UTF8Strategy};
use std::io;

struct Chunk {
    lines: Vec<String>,
}

struct HunkDiffWithTail {
    diff: Chunk,
    context_tail: Chunk,
}

struct Hunk {
    header: String,
    context_head: Chunk,
    diffs: Vec<HunkDiffWithTail>,
}

struct FileDiff {
    file_header: Chunk,
    hunks: Vec<Hunk>,
}

struct Patch {
    patch_header: Chunk,
    files: Vec<FileDiff>,
}

enum State {
    Start,
    PatchHeader,
    FileHeader,
    HunkHead,
    HunkBodyDiff,
    HunkBodyTail,
}

fn chunk_from(line: String) -> Chunk {
    Chunk { lines: vec![line] }
}

fn chunk_empty() -> Chunk {
    Chunk { lines: Vec::new() }
}

fn print_hunk<'a>(
    hunk: &Hunk,
    writer: &mut Box<dyn std::io::Write + 'a>
) -> std::io::Result<()> {
    write!(writer, "{}", hunk.header)?;
    for line in &hunk.context_head.lines {
        write!(writer, "{}", line)?;
    }
    for diff in &hunk.diffs {
        for line in &diff.diff.lines {
            write!(writer, "{}", line)?;
        }
        for line in &diff.context_tail.lines {
            write!(writer, "{}", line)?;
        }
    }
    Ok(())
}

fn process_hunk<'a>(
    pattern: &String,
    hunk: &Hunk, writer: &mut Box<dyn io::Write + 'a>
) -> std::io::Result<()> {
    if hunk.header.contains(pattern) {
        print_hunk(hunk, writer)?;
    }
    for line in &hunk.context_head.lines {
        if line.contains(pattern) {
            print_hunk(hunk, writer)?;
        }
    }
    for diff in &hunk.diffs {
        for line in &diff.diff.lines {
            if line.contains(pattern) {
                print_hunk(hunk, writer)?;
            }
        }
        for line in &diff.context_tail.lines {
            if line.contains(pattern) {
                print_hunk(hunk, writer)?;
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let config = parse_program_args(&mut std::env::args());
    process_lines(
        Box::new(io::stdin().lock()), 
        Box::new(io::stdout().lock()), 
        config.search_string, 
        config.decode_stategy)
}

fn process_lines<'a>(
        mut reader: Box<dyn io::BufRead + 'a>,
        mut writer: Box<dyn io::Write + 'a>,
        search_pattern: String,
        decode_strategy: UTF8Strategy
) -> std::io::Result<()> {
    let mut _line_num = 0;
    let mut state = State::Start;
    // store only 1 patch worth of context
    let mut patch = Patch {
        patch_header: Chunk { lines: Vec::new() },
        files: Vec::new(),
    };

    loop {
        let mut line_buf: Vec<u8> = Vec::new();
        if reader.read_until(b'\n', &mut line_buf).expect("Failed to read.") == 0 {
            break;
        }
        _line_num += 1;
        
        let line = match decode_strategy {
            UTF8Strategy::Lossy => String::from_utf8_lossy(&line_buf).into_owned(),
            UTF8Strategy::Panic => String::from_utf8(line_buf).expect(
                &format!("Invalid UTF-8 on line {_line_num}")),
            UTF8Strategy::SkipLine => {
                // Choose the default value based on the state to avoid taking
                // an unnecesary state transition or adding extra rules to the
                // state machine to handle this edge case.
                let default_value = match state {
                    State::HunkHead => "+",
                    _ => " "
                }.to_string();
                String::from_utf8(line_buf).unwrap_or(default_value)
            }
        };
        
        let line_ansi_stripped = strip_ansi_codes(&line);
        let mut line_stripped = line_ansi_stripped.into_owned();
        if line_stripped.ends_with("\n") {
            line_stripped.pop();
            if line_stripped.ends_with("\r") {
                line_stripped.pop();
            }
        }

        match state {
            State::Start => {
                if line_stripped.starts_with("commit ") {
                    patch = Patch {
                        patch_header: chunk_from(line),
                        files: Vec::new(),
                    };
                    state = State::PatchHeader;
                } else {
                    panic!("Invalid patch. Expected commit message");
                }
            }
            State::PatchHeader => {
                if line_stripped.starts_with("diff --git") {
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
                } else {
                    patch.patch_header.lines.push(line);
                }
            }
            State::FileHeader => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                if line_stripped.starts_with("@@") {
                    file.hunks.push(Hunk {
                        header: line,
                        context_head: chunk_empty(),
                        diffs: Vec::new(),
                    });
                    state = State::HunkHead;
                } else {
                    file.file_header.lines.push(line);
                }
            }
            State::HunkHead => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                let hunk = file.hunks.last_mut().expect("Expected a hunk");
                if line_stripped.starts_with(" ") {
                    hunk.context_head.lines.push(line);
                } else if line_stripped.starts_with("+") || line_stripped.starts_with("-") {
                    hunk.diffs.push(HunkDiffWithTail {
                        diff: chunk_from(line),
                        context_tail: chunk_empty(),
                    });
                    state = State::HunkBodyDiff;
                } else {
                    panic!("Unknown state in hunk head");
                }
            }
            State::HunkBodyDiff => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                let hunk = file.hunks.last_mut().expect("Expected a hunk");
                let hunk_diff = hunk.diffs.last_mut().expect("Expected a hunk diff");
                if line_stripped.starts_with("+") || line_stripped.starts_with("-") {
                    hunk_diff.diff.lines.push(line);
                } else if line_stripped.starts_with(" ")
                    || line_stripped.is_empty()
                    || line_stripped == "\\ No newline at end of file"
                {
                    hunk_diff.context_tail.lines.push(line);
                    state = State::HunkBodyTail;
                } else if line_stripped.starts_with("diff --git") {
                    process_hunk(&search_pattern, &hunk, &mut writer)?;
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
                } else if line_stripped.starts_with("commit ") {
                    process_hunk(&search_pattern, &hunk, &mut writer)?;
                    patch = Patch {
                        patch_header: chunk_from(line),
                        files: Vec::new(),
                    };
                    state = State::PatchHeader;
                } else {
                    panic!("Unknown state in hunk body");
                }
            }
            State::HunkBodyTail => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                let hunk = file.hunks.last_mut().expect("Expected a hunk");
                let hunk_diff = hunk.diffs.last_mut().expect("Expected a hunk diff");
                if line_stripped.starts_with(" ")
                    || line_stripped.is_empty()
                    || line_stripped == "\\ No newline at end of file"
                {
                    hunk_diff.context_tail.lines.push(line);
                } else if line_stripped.starts_with("+") || line_stripped.starts_with("-") {
                    hunk_diff.diff.lines.push(line);
                    state = State::HunkBodyDiff;
                } else if line_stripped.starts_with("@@") {
                    process_hunk(&search_pattern, &hunk, &mut writer)?;
                    file.hunks.push(Hunk {
                        header: line,
                        context_head: chunk_empty(),
                        diffs: Vec::new(),
                    });
                    state = State::HunkHead;
                } else if line_stripped.starts_with("diff --git") {
                    process_hunk(&search_pattern, &hunk, &mut writer)?;
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
                } else if line_stripped.starts_with("commit ") {
                    process_hunk(&search_pattern, &hunk, &mut writer)?;
                    patch = Patch {
                        patch_header: chunk_from(line),
                        files: Vec::new(),
                    };
                    state = State::PatchHeader;
                } else {
                    panic!("Unknown state in hunk tail");
                }
            }
        };
    }
    patch
        .files
        .last_mut()
        // merge commits can have an empty file section - skip processing those
        .map(|file| {
            file
                .hunks
                .last_mut()
                // file section can have empty hunks - skip those
                .map(|hunk| process_hunk(&search_pattern, &hunk, &mut writer))
        });
    Ok(())
}
