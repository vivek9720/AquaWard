use std::env;
use std::fs;

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: aquaward-dump <bundle>");
        std::process::exit(2);
    };
    let data = match fs::read(&path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{path}: {err}");
            std::process::exit(1);
        }
    };
    match aquaward::decode_and_analyze_bundle(&data) {
        Ok(report) => print!("{}", aquaward::report::render_text(&report)),
        Err(err) => {
            eprintln!("{path}: {err}");
            std::process::exit(1);
        }
    }
}
