use std::io::{self, Write};

pub fn approve_tool() -> bool {
    println!("REPLY YES TO RUN, NO TO REJECT");
    let _ = io::stdout().flush();

    let mut input = String::new();

    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    input.trim().eq_ignore_ascii_case("yes")
}
