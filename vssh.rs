use std::{env, path::Path, ffi::CString};
use nix::{sys::wait::waitpid,unistd::{fork, ForkResult, execvp}};

fn main() -> std::io::Result<()> {
    // From https://doc.rust-lang.org/std/env/fn.current_dir.html
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());
    // Modified from https://github.com/gjf2a/shell/blob/master/src/bin/typing_demo.rs
    loop {
        let user_input = process_line();
        if user_input.trim() == "exit" {
            println!("You have exited the vssh");
            break;
        } else if user_input.trim() == "" {
            println!("You didn't enter anything")
        } else {
            let words: Vec<&str> = user_input.split_whitespace().collect();
            if words[0].trim() == "cd" {
                if words.len() > 1 {
                    let path = env::current_dir()?;
                    // Modified from https://doc.rust-lang.org/std/env/fn.set_current_dir.html
                    let new_path = &path.join(path.clone()).join(words[1].trim()); // https://doc.rust-lang.org/rust-by-example/std_misc/path.html
                    let root = Path::new(new_path); 
                    match env::set_current_dir(&root) {
                        Ok(()) => {}
                        Err(e) => {println!("{}", e);}
                    }
                    //assert!(env::set_current_dir(&root).is_ok()); // Will panic if directory doesn't exist
                    println!("Successfully changed working directory to {}!", root.display());
                    let path = env::current_dir()?;
                    println!("The current directory is {}", new_path.display());
                }
            } else {
                fork_time(user_input.as_str());
            }
        }
    }
    Ok(())
}

// Modified from https://github.com/gjf2a/shell/blob/master/src/bin/typing_demo.rs
fn process_line() -> String {
    println!("Please enter something");
    let mut user_input = String::new();
    print!("$ ");
    std::io::stdin().read_line(&mut user_input).expect("Failed to read line"); //https://www.geeksforgeeks.org/standard-i-o-in-rust/
    user_input
}


// From https://docs.rs/nix/0.26.2/nix/unistd/fn.fork.html
fn fork_time(user_input: &str) {
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("Continuing execution in parent process, new child has pid: {}", child);
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            // From https://github.com/gjf2a/shell/blob/master/src/bin/fork_ls_demo.rs
            let cmd = externalize(user_input);
            println!("{cmd:?}");
            match execvp::<CString>(cmd[0].as_c_str(), &cmd) {
                Ok(_) => {println!("Child finished");},
                Err(e) => {println!("Could not execute: {e}");},
            }
        }
        Err(_) => println!("Fork failed"),
     }
}

// From https://github.com/gjf2a/shell/blob/master/src/bin/fork_ls_demo.rs
fn externalize(command: &str) -> Vec<CString> {
    command.split_whitespace()
        .map(|s| CString::new(s).unwrap())
        .collect()
}