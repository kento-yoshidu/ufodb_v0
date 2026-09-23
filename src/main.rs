use std::io::{self, BufRead};

use clap::{Parser, Subcommand};
use ufodb_v0::db::self;

mod snapshot;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
struct StartupArgs {
    #[arg(long, default_value = "ufdb")]
    db: String,
}

#[derive(Subcommand)]
#[command(rename_all = "UPPER")]
enum Commands {
    Hello,
    Insert { key: String },
    Same { key_a: String, key_b: String },
    Merge { key_a: String, key_b: String },
    Size { key: String },
    Groups,
    Createdb { db_name: String },
    Unmerge { key_a: String, key_b: String },
    Save,
    Load { db_name: String },
    Snapshot,
    SEED,
    Exit,
}

fn main() {
    let args = StartupArgs::parse();

    let data_dir = match ufodb_v0::storage::data_dir() {
        Ok(dir) => dir,
        Err(e) => panic!("ホームディレクトリを特定できませんでした。: {e}"),
    };

    let mut db = match db::Db::new(&args.db, &data_dir) {
        Ok(db) => db,
        Err(db::LoadError::NotFound) => {
            eprintln!("DB {} は存在しません。", args.db);
            std::process::exit(1);
        }
        Err(db::LoadError::Corrupted(e)) => {
            eprintln!("DBの読み込みに失敗しました: {e}");
            std::process::exit(1);
        }
    };

    let mut lines = io::stdin().lock().lines();

    let db_name = args.db;

    while let Some(line) = lines.next() {
        let line = line.unwrap();

        let mut tokens = vec!["repl"];
        tokens.extend(line.split_whitespace());

        match Cli::try_parse_from(tokens) {
            Ok(cli) => {
                let start = std::time::Instant::now();

                match cli.command {
                    Commands::Hello => {
                        println!("Hello World");
                    },
                    Commands::Insert { key } => {
                        let inserted = db.get_mut(&db_name).unwrap().make_set(&key);
                        println!("{inserted}");
                    },
                    Commands::Merge { key_a, key_b } => {
                        let res = db.get_mut(&db_name).unwrap().unite(&key_a, &key_b);
                        println!("{res}");
                    },
                    Commands::Same { key_a, key_b } => {
                        let res = db.get_mut(&db_name).unwrap().same(&key_a, &key_b);
                        println!("{res}");
                    }
                    Commands::Groups => {
                        let mut groups: Vec<Vec<&String>> = db.get_mut(&db_name).unwrap().groups().into_values().collect();

                        for group in &mut groups {
                            group.sort();
                        }

                        groups.sort_by(|a, b| b.len().cmp(&a.len()));

                        for group in groups {
                            let line: String = group.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");

                            println!("{line}");
                        }
                    },
                    Commands::Size { key } => {
                        match db.get_mut(&db_name).unwrap().size(&key) {
                            Some(size) => println!("{size}"),
                            None => eprintln!("キー {key} は登録されていません"),
                        }
                    },
                    Commands::Createdb { db_name } => {
                        if db.create_db(&db_name) {
                            match ufodb_v0::storage::save(db.get_mut(&db_name).unwrap(), &db_name, &data_dir) {
                                Ok(()) => println!("DB {db_name} を作成しました。"),
                                Err(e) => eprintln!("DBファイルの作成に失敗しました: {e}"),
                            }
                        } else {
                            println!("DB {db_name} は既に存在します。");
                        }
                    },
                    Commands::Unmerge { key_a, key_b } => {
                        db.get_mut(&db_name).unwrap().unmerge(&key_a, &key_b);
                    },
                    Commands::Save => {
                        match ufodb_v0::storage::save(db.get_mut(&db_name).unwrap(), &db_name, &data_dir) {
                            Ok(()) => println!("保存しました"),
                            Err(e) => eprintln!("保存に失敗しました : {e}"),
                        }
                    }
                    Commands::Load { db_name } => {
                        println!("{:?}", ufodb_v0::storage::load(&db_name, &data_dir));
                    },
                    Commands::Snapshot => {
                        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();

                        let addr = server.server_addr();
                        let url = format!("http://{addr}");

                        open::that(url).expect("Server Error");

                        let request = server.recv().unwrap();

                        let html = snapshot::render(db.get_mut(&db_name).unwrap());
                        let response = tiny_http::Response::from_string(html)
                            .with_header(
                                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..]).unwrap()
                            );

                        request.respond(response).unwrap()
                    },
                    Commands::SEED => {
                        if db.get_mut(&db_name).unwrap().is_empty() {
                            db.get_mut(&db_name).unwrap().seed();
                        } else {
                            println!("既存のデータがあります。SEEDを実行しますか？ (y/n)");

                            let ans= lines.next().unwrap().unwrap();

                            if ans.trim() == "y" {
                                db.get_mut(&db_name).unwrap().seed();
                            }
                        }
                    },
                    Commands::Exit => {
                        println!("bye.");
                        break;
                    },
                }

                let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                println!("elapsed: {elapsed_ms:.6}ms\n");
            },
            Err(e) => eprintln!("{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_same_command() {
        let cli = Cli::try_parse_from(["repl", "SAME", "a", "b"]).unwrap();

        assert!(matches!(cli.command, Commands::Same { key_a, key_b } if key_a == "a" && key_b == "b"));
    }

    #[test]
    fn parses_merge_command() {
        let cli = Cli::try_parse_from(["repl", "MERGE", "a", "b"]).unwrap();

        assert!(matches!(cli.command, Commands::Merge { key_a, key_b } if key_a == "a" && key_b == "b"));
    }

    #[test]
    fn parses_size_command() {
        let cli = Cli::try_parse_from(["repl", "SIZE", "a"]).unwrap();

        assert!(matches!(cli.command, Commands::Size { key } if key == "a"));
    }

    #[test]
    fn parses_insert_command() {
        let cli = Cli::try_parse_from(["repl", "INSERT", "a"]).unwrap();

        assert!(matches!(cli.command, Commands::Insert { key } if key == "a"));
    }

    #[test]
    fn parses_createdb_command() {
        let cli = Cli::try_parse_from(["repl", "CREATEDB", "mydb"]).unwrap();
        assert!(matches!(cli.command, Commands::Createdb { db_name } if db_name == "mydb"));
    }

    #[test]
    fn parses_unmerge_command() {
        let cli = Cli::try_parse_from(["repl", "UNMERGE", "a", "b"]).unwrap();

        assert!(matches!(cli.command, Commands::Unmerge { key_a, key_b } if key_a == "a" && key_b == "b"));
    }

    #[test]
    fn parses_no_arg_commands() {
        assert!(matches!(
            Cli::try_parse_from(["repl", "GROUPS"]).unwrap().command,
            Commands::Groups
        ));
        assert!(matches!(
            Cli::try_parse_from(["repl", "SNAPSHOT"]).unwrap().command,
            Commands::Snapshot
        ));
        assert!(matches!(
            Cli::try_parse_from(["repl", "SEED"]).unwrap().command,
            Commands::SEED
        ));
        assert!(matches!(
            Cli::try_parse_from(["repl", "EXIT"]).unwrap().command,
            Commands::Exit
        ));
    }

    #[test]
    fn unknown_or_malformed_input_is_err_not_panic() {
        assert!(Cli::try_parse_from(["repl", "BOGUS"]).is_err());
        assert!(Cli::try_parse_from(["repl", "MERGE", "only_one"]).is_err());
    }
}
