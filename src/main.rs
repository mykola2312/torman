use std::{fs::{self, DirEntry}, path::Path};
use clap::{Parser, Subcommand};
use rusqlite::Connection;

mod bencode;

#[derive(Subcommand)]
enum Command {
    Index {
        #[arg(short='p', long, default_value="torman.db", help="path to transmission dir with torrents and resume")]
        path: String
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short='d', long, help="path to sqlite database")]
    db_path: String,

    #[command(subcommand)]
    command: Command
}

fn index(db: Connection, path: &String) {
    // iterate resume files
    let entries: Vec<DirEntry> = fs::read_dir(Path::new(path).join("resume"))
        .unwrap()
        .filter_map(|f| f.ok())
        .collect();
    for entry in entries {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue
        };
        if !file_type.is_file() {
            continue;
        }

        let hash = match Path::new(&entry.file_name()).file_stem() {
            Some(stem) => match stem.to_os_string().into_string() {
                Ok(str) => str,
                Err(os_str) => {
                    eprintln!("failed to convert file name for {:#?}", os_str);
                    continue;
                }
            },
            None => {
                eprintln!("file {:#?} has no extension or conversion failed", entry.file_name());
                continue;
            }
        };

        println!("{}", hash);
    }
}

fn main() {
    let args = Args::parse();
    let db = Connection::open(args.db_path).unwrap();

    match &args.command {
        Command::Index { path } => {
            index(db, path);
        }
    }
}