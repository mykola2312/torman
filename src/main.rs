use std::fs;
use clap::{Parser, Subcommand};

mod bencode;

#[derive(Subcommand)]
enum Command {
    Index {
        #[arg(short='p', long, help="path to transmission dir with torrents and resume")]
        path: String
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Command
}

fn index(path: &String) {

}

fn main() {
    let args = Args::parse();

    match &args.command {
        Command::Index { path } => {
            index(path);
        }
    }
}