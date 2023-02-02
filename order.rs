use std::{ fs::File, io::{BufReader, BufRead}};

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.len() {
        0 => { println!("Usage: orders filenames");
        println!("Can add '-r' after orders if you want lines listed in reverse order.");
        }
        _ => {
            let r = match arguments[0].as_str() {
                "-r" => true,
                _ => false,
            };
            match list_of_lines(&&arguments[0..], r) {
                Ok(()) => {},
                Err(e) => println!("Error when reading files: {e}")
            };
        }
    }
}

fn list_of_lines(filenames: &[String], r: bool) -> anyhow::Result<()> {
    let mut vec= Vec::new();
    for filename in filenames {
        if filename.starts_with("-") {
            continue;
        }
        let f = File::open(filename)?;
        let buffer = BufReader::new(f);
        for line in buffer.lines() {
            let line = line?;
            vec.push(line);
        }
    }
    vec.sort();
    if r == true {
        vec.reverse();
    }
    for ele in vec {
        println!("{ele}");
    }
    Ok(())
}