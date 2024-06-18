use std::{fs::{self, DirEntry}, path::Path, env};
use bencode::{decode, Value};
use clap::{Parser, Subcommand};
use rusqlite::Connection;
use dotenv::dotenv;

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

#[derive(Debug)]
enum TorrentLogicError {
    NoTorrentName,
    NoInfoDict,
    NoPathList
}

fn get_torrent_files(torrent: &Value) -> Result<Vec<String>, TorrentLogicError> {
    let info = torrent.get_value("info")
        .ok_or(TorrentLogicError::NoInfoDict)?;

    let mut files: Vec<String> = Vec::new();
    if let Some(file_list_v1) = info.get_list("files") {
        // multiple files v1
        let root = info.get_string("name")
            .ok_or(TorrentLogicError::NoTorrentName)?;
        for file_object in file_list_v1 {
            let mut path_list = file_object.get_string_list("path")
                .ok_or(TorrentLogicError::NoPathList)?;
            
            path_list.insert(0, root.clone());
            files.push(path_list.join("/"));
        }
    } else if let Some(file_dict_v2) = info.get_dict("file tree") {
        // single file / multiple files v2
        let root = info.get_string("name")
            .ok_or(TorrentLogicError::NoTorrentName)?;
        
        let files_v2: Vec<String> = file_dict_v2
            .keys()
            .into_iter()
            .filter_map(|k| k.to_string())
            .collect();
        if files_v2.len() > 1 {
            // multiple
            for file in files_v2 {
                let mut path = root.clone();
                path.push_str("/");
                path.push_str(&file);
                
                files.push(path); 
            }
        } else {
            // single
            files.push(files_v2.first().unwrap().clone());
        }
    } else {
        // single file v1
        let single = info.get_string("name")
            .ok_or(TorrentLogicError::NoTorrentName)?;
        files.push(single);
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
        let files = match get_torrent_files(&torrent) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("can't get file list for {}: {:#?}", hash, e);
                continue;
            }
        };

        // create torrent record
        let id = db.query_row("INSERT INTO torrent (
            hash, name, destination,
            downloaded, uploaded,
            announce, comment,
            created_by, creation_date,
            publisher, publisher_url)
            VALUES (?,?,?,?,?,?,?,?,?,?,?)
            RETURNING id;", (
                hash, name, destination,
                downloaded, uploaded,
                announce, comment,
                created_by, creation_date,
                publisher, publisher_url
            ),
            |row| Ok(row.get::<usize, i64>(0)?) 
        ).expect("insert failed");
        // we're using here unwrap/expect since we want full program crash
        // to debug any sql bugs

        // insert torrent files
        for file in files {
            db.execute("INSERT INTO file (torrent_id,file_name) VALUES (?,?);", (id, file))
            .expect("failed to insert file!");
        }
    }
}

fn main() {
    dotenv().ok();
    let args = Args::parse();
    let db = Connection::open(args.db_path).unwrap();

    match &args.command {
        Command::Index { path } => {
            index(db, path);
        }
    }
}