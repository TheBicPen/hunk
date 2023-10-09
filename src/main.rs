mod test;

use console::strip_ansi_codes;
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

fn print_hunk(hunk: &Hunk) {
    println!("{}", hunk.header);
    for line in &hunk.context_head.lines {
        println!("{}", line);
    }
    for diff in &hunk.diffs {
        for line in &diff.diff.lines {
            println!("{}", line);
        }
        for line in &diff.context_tail.lines {
            println!("{}", line);
        }
    }
}

fn process_hunk(pattern: &String, hunk: &Hunk) {
    if hunk.header.contains(pattern) {
        print_hunk(hunk);
    }
    for line in &hunk.context_head.lines {
        if line.contains(pattern) {
            print_hunk(hunk);
        }
    }
    for diff in &hunk.diffs {
        for line in &diff.diff.lines {
            if line.contains(pattern) {
                print_hunk(hunk);
            }
        }
        for line in &diff.context_tail.lines {
            if line.contains(pattern) {
                print_hunk(hunk);
            }
        }
    }
}

fn main() {
    let search_pattern = std::env::args()
        .nth(1)
        .expect("Provide a string to search hunks for.");
    process_lines(search_pattern, Box::new(io::stdin()))
}

fn process_lines(search_pattern: String, mut input: Box<dyn io::Read>) {
    let mut _line_num = 0;
    let mut content = String::new();
    input.read_to_string(&mut content).expect("failed to read");
    let lines = content.lines();
    let mut state = State::Start;
    // store only 1 patch worth of context
    let mut patch = Patch {
        patch_header: Chunk { lines: Vec::new() },
        files: Vec::new(),
    };
    for unowned_line in lines {
        _line_num += 1;
        let line = unowned_line.to_string();
        let line_stripped = strip_ansi_codes(&line);
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
                    process_hunk(&search_pattern, &hunk);
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
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
                    process_hunk(&search_pattern, &hunk);
                    file.hunks.push(Hunk {
                        header: line,
                        context_head: chunk_empty(),
                        diffs: Vec::new(),
                    });
                    state = State::HunkHead;
                } else if line_stripped.starts_with("diff --git") {
                    process_hunk(&search_pattern, &hunk);
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
                } else if line_stripped.starts_with("commit ") {
                    process_hunk(&search_pattern, &hunk);
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
    let file = patch
        .files
        .last_mut()
        .expect("Expected a file diff. Was the input empty?");
    let hunk = file
        .hunks
        .last_mut()
        .expect("Expected a hunk. Was the input empty?");
    process_hunk(&search_pattern, &hunk);
}
