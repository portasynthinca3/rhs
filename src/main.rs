mod config;
mod log;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // parse arguments assigning default values
    let (dir, port) = match args.len() {
        // double-argument variant: <dir> <port>
        3 => {
            let dir = args[1].clone();
            match args[2].parse::<u16>() {
                Ok(port) => (dir, port),
                Err(_) => {
                    log::error(&format!("Invalid port number: {}", args[2]));
                    return;
                }
            }
        },

        // single-argument variant: <dir> (default to ".") OR <port> (default to 80)
        2 => {
            match args[1].parse::<u16>() {
                Ok(port) => {
                    log::info(&format!("assuming `{}` is a port. Pass both <dir> and <port> if you meant it as the directory", port));
                    (String::from("."), port)
                },
                Err(_) => {
                    (args[1].clone(), 80u16)
                }
            }
        },

        // \(-_-)/
        _ => {
            log::error(&format!("Usage: {} [directory] [port]", args[0]));
            return;
        }
    };

    let config = config::Config { dir, port };
    dbg!(config);
}
