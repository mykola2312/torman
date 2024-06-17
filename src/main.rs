use std::{fs::{self, DirEntry}, path::Path};
use bencode::{decode, Value};
use clap::{Parser, Subcommand};
use rusqlite::Connection;

mod bencode;

#[derive(Subcommand)]
enum Command {
    Index {
        #[arg(short='p', long, default_value="torman.db", help="path to transmission dir with \"torrents\" and \"resume\" dirs")]
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

enum TorrentLogicError {
    NoTorrentName,
    NoInfoDict,
    FileValueIsNotList,
    NoPathList
}

fn get_torrent_files(torrent: &Value, hash: &str) -> Result<Vec<String>, TorrentLogicError> {
    let info = torrent.get_value("info")
        .ok_or(TorrentLogicError::NoInfoDict)?;

    // TODO: when torrent has single file in "files" that means "name" of the torrent is dir name
    let mut files: Vec<String> = Vec::new();
    if let Some(file_list_value) = info.get_value("files") {
        let file_list = file_list_value.to_list()
            .ok_or(TorrentLogicError::FileValueIsNotList)?;
        
        for file_object in file_list {
            let path_list = file_object.get_string_list("path")
                .ok_or(TorrentLogicError::NoPathList)?;
            // we will join with unixy / path separator since I dont want to make clusterfuck tables
            // for multi-dimensional file list nor it would be ran on windows
            let file_name = path_list.join("/");
            files.push(file_name);
        }
    } else {
        // when we don't have "files" list in "info" block,
        // that means "name" of torrent is the file name
        files.push(torrent.get_string("name")
            .ok_or(TorrentLogicError::NoTorrentName)?);
    }

    Ok(files)
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

        // parse the resume file
        let (resume, _) = {
            let resume_data = match fs::read(entry.path()) {
                Ok(data) => data,
                Err(_) => {
                    eprintln!("failed to read {} resume file", hash);
                    continue
                }
            };

            match decode(&resume_data) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("failed to parse {} resume file: {:#?}", hash, e);
                    continue
                }
            }
        };

        // parse the torrent file
        let torrent_path = {
            let mut torrent_name = hash.to_owned();
            torrent_name.push_str(".torrent");

            Path::new(path).join("torrents").join(torrent_name)
        };

        let (torrent, _) = {
            let torrent_data = match fs::read(torrent_path) {
                Ok(data) => data,
                Err(_) => {
                    eprintln!("failed to read {} torrent file", hash);
                    continue
                }
            };

            match decode(&torrent_data) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("failed to parse {} torrent file: {:#?}", hash, e);
                    continue
                }
            }
        };

        // make table row
        let hash = hash;
        let name = resume.get_string("name");
        let destination = resume.get_string("destination");
        let downloaded = resume.get_integer("downloaded");
        let uploaded = resume.get_integer("uploaded");

        let announce = torrent.get_string("announce");
        let comment = torrent.get_string("comment");
        let created_by = torrent.get_string("created_by");
        let creation_date = torrent.get_integer("creation_date");
        let publisher = torrent.get_string("publisher");
        let publisher_url = torrent.get_string("publisher-url");

        // get torrent files
        // TODO: when torrent has single file in "files" that means "name" of the torrent is dir name
        let files = match get_torrent_files(&torrent, &hash) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("can't get file list for {}", hash);
                continue;
            }
        };

        dbg!(hash, name, destination, downloaded, uploaded, announce, comment, created_by, creation_date, publisher, publisher_url);
        dbg!(files);

        break
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