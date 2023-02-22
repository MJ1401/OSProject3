use std::{env, path::Path, ffi::CString, fs};
use nix::{unistd::{fork, pipe, close, dup2, execvp, ForkResult}, sys::{wait::waitpid, stat::Mode}, fcntl::{OFlag, open}};

fn main() -> anyhow::Result<()> {
    // From https://doc.rust-lang.org/std/env/fn.current_dir.html
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());
    // Modified from https://github.com/gjf2a/shell/blob/master/src/bin/typing_demo.rs
    loop {
        let user_input = process_line()?;
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
                        Ok(()) => {//assert!(env::set_current_dir(&root).is_ok()); // Will panic if directory doesn't exist
                        println!("Successfully changed working directory to {}!", root.display());
                        let path = env::current_dir()?;
                        println!("The current directory is {}", new_path.display());
                    }
                        Err(e) => {println!("{}", e);}
                    }
                    
                }
            } else {
                match fork_time(user_input.as_str()) {
                    Ok(_) => {println!("Fork was successful");},
                    Err(e) => {println!("Could not fork: {e}");},
                }
            }
        }
    }
    Ok(())
}

// Modified from https://github.com/gjf2a/shell/blob/master/src/bin/typing_demo.rs
fn process_line() -> anyhow::Result<String> {
    println!("Please enter something");
    let mut user_input = String::new();
    print!("$ ");
    std::io::stdin().read_line(&mut user_input)?; //https://www.geeksforgeeks.org/standard-i-o-in-rust/
    Ok(user_input)
}


// From https://docs.rs/nix/0.26.2/nix/unistd/fn.fork.html
fn fork_time(user_input: &str) -> anyhow::Result<()> {
    let mut cmds = Components::new();
    cmds.get_pipes(user_input.to_owned());
    let pipes = cmds.pipe_cmds;
    let out_file = cmds.output_file;
    let in_file = cmds.input_file;
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("Starting a fork, new child has pid: {}", child);
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            let mut fd_out = 1;
            for p in pipes.iter().skip(1).rev() {
                let (p_out, p_in) = pipe()?;
                match unsafe{fork()} {
                    Ok(ForkResult::Parent { child, .. }) => {
                        close(p_in)?;
                        dup2(p_out, 0)?;
                        dup2(fd_out, 1)?;
                        let p2 = externalize(p.as_str());
                        match execvp::<CString>(p2[0].as_c_str(), &p2) {
                            Ok(_) => {println!("Child finished");},
                            Err(e) => {
                                println!("Could not execute: {e}");
                                std::process::exit(1);
                            },
                        }
                    }
                    Ok(ForkResult::Child) => {
                        close(p_out)?;
                        fd_out = p_in;
                    }
                    Err(e) => {println!("Error: {e}")}
                }
            }
            // This isn't quite working how it should be. I get some weird interactions

            // From https://github.com/gjf2a/shell/blob/master/src/bin/fork_ls_demo.rs
            let cmd = externalize(&pipes[0]);
            match in_file {
                Some(ref f) => { // https://doc.rust-lang.org/std/option/
                    // From https://github.com/gjf2a/shell/blob/master/src/bin/pipe_demo_3.rs
                    let flags: OFlag = [OFlag::O_RDONLY, OFlag::O_TRUNC].iter().copied().collect();
                    let mode: Mode = [Mode::S_IRUSR, Mode::S_IWUSR].iter().copied().collect();
                    let file_out = open(f.to_owned().as_str(), flags, mode)?;
                    dup2(file_out, 0)?;
                }
                None => print!(""),
            }
            match out_file {
                Some(ref f) => { // https://doc.rust-lang.org/std/option/
                    // From https://github.com/gjf2a/shell/blob/master/src/bin/pipe_demo_3.rs
                    let flags: OFlag = [OFlag::O_CREAT, OFlag::O_WRONLY, OFlag::O_TRUNC].iter().copied().collect();
                    let mode: Mode = [Mode::S_IRUSR, Mode::S_IWUSR].iter().copied().collect();
                    let file_out = open(f.to_owned().as_str(), flags, mode)?;
                    dup2(file_out, 1)?;
                }
                None => print!(""),
            }
            dup2(fd_out, 1)?;
            match execvp::<CString>(cmd[0].as_c_str(), &cmd) {
                Ok(_) => {println!("Child finished");},
                Err(e) => {
                    println!("Could not execute: {e}");
                    std::process::exit(1);
                },
            }
        }
        Err(_) => println!("Fork failed"),
    }
    Ok(())
}

// From https://github.com/gjf2a/shell/blob/master/src/bin/fork_ls_demo.rs
fn externalize(command: &str) -> Vec<CString> {
    command.split_whitespace()
        .map(|s| CString::new(s).unwrap())
        .collect()
}

// From https://stackoverflow.com/questions/32384594/how-to-check-whether-a-path-exists
fn file_exist(f: String) -> bool {
    fs::metadata(f).is_ok()
}

// Inspired from https://github.com/gjf2a/shell/blob/master/src/bin/struct_demo.rs
struct Components {
    background: bool,
    output_file: Option<String>,
    input_file: Option<String>,
    pipe_cmds: Vec<String>,
}

impl Components {
    fn new() -> Self {
        Components {background: false, output_file: None, input_file: None, pipe_cmds: vec![]}
    }

    fn back_check(&mut self, arg: &str) -> String {
        let last = arg.chars().last().unwrap(); // Modified from https://stackoverflow.com/questions/48642342/how-to-get-the-last-character-of-a-str
        let new_arg = arg.clone();
        if last == '&' {
            self.background = true;
            let new_arg = &arg[0..arg.len()-1];
            return new_arg.to_owned();
        }
        return new_arg.to_owned();
    }

    fn get_pipes(&mut self, line: String) {
        let args = line.split("|");
        for arg in args {
            let mut a = self.back_check(arg);
            a = self.out_check(a.to_owned());
            a = self.in_check(a.to_owned());
            self.pipe_cmds.push(a.to_string());
        }
    }

    fn out_check(&mut self, line: String) -> String {
        let args = line.split_whitespace();
        let mut next = false;
        let mut out = "".to_owned();
        for arg in args {
            if next {
                self.output_file = Some(arg.to_string());
                next = false;
            } else if arg.eq(">") {
                next = true;
            } else {
                out.push_str(&arg);
                out.push_str(" ");
            }
        }
        out
    }

    fn in_check(&mut self, line: String) -> String {
        let args = line.split_whitespace();
        let mut next = false;
        let mut out = "".to_owned();
        for arg in args {
            if next {
                self.input_file = Some(arg.to_string());
                next = false;
            } else if arg.eq("<") {
                next = true;
            } else {
                out.push_str(&arg);
                out.push_str(" ");
            }
        }
        out
    }
}