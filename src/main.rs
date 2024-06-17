use std::fs;
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

fn index(path: &String) {

}

fn main() {
    let args = Args::parse();

    let db = Connection::open(args.db_path).unwrap();
    db.execute("INSERT INTO `test` (test) VALUES (?1)", (1337,)).unwrap();

    match &args.command {
        Command::Index { path } => {
            index(path);
        }
    }
}