use std::{io::{BufReader, BufRead}, fs::File};

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.len() {
        0 => {
            println!("Usage: findtext text filenames");
        }
        _ => {
            let s = &arguments[0];
            let mut part = arguments.clone();
            part.remove(0);
            for arg in part {
                let buffer = get_buffer(arg);
                match_lines(buffer, s);
            }
        }
    }
}


fn get_buffer(filename: String) -> BufReader<File> {
    let f = File::open(filename).unwrap();
    let buffer = BufReader::new(f);
    buffer
}


fn match_lines(buffer: BufReader<File>, s: &String) {
    for line in buffer.lines() {
        let line = line.unwrap();
        if line.contains(s) {
            println!("{}", line);
        }
    }
}