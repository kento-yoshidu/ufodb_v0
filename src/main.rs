use std::io::{self, BufRead};

use clap::{Parser, Subcommand};
use ufodb_v0::db;

mod snapshot;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    Use { db_name: String },
    Unmerge { key_a: String, key_b: String },
    Snapshot,
    SEED,
    Exit,
}

fn main() {
    let mut db = db::Db::new();

    let mut lines = io::stdin().lock().lines();

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
                        let inserted = db.current().make_set(&key);
                        println!("{inserted}");
                    },
                    Commands::Merge { key_a, key_b } => {
                        let res = db.current().unite(&key_a, &key_b);
                        println!("{res}");
                    },
                    Commands::Same { key_a, key_b } => {
                        let res = db.current().same(&key_a, &key_b);
                        println!("{res}");
                    }
                    Commands::Groups => {
                        let mut groups: Vec<Vec<&String>> = db.current().groups().into_values().collect();

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
                        match db.current().size(&key) {
                            Some(size) => println!("{size}"),
                            None => eprintln!("キー {key} は登録されていません"),
                        }
                    },
                    Commands::Createdb { db_name } => {
                        if db.create_db(&db_name) {
                            println!("DB {db_name} を作成しました。");
                        } else {
                            println!("DB {db_name} は既に存在します。");
                        }
                    },
                    Commands::Use { db_name } => {
                        if db.use_db(&db_name) {
                            println!("DB {db_name} に切り替えました。");
                        } else {
                            println!("DB {db_name} は存在しません。作成しますか？(y/n)");

                            let ans = lines.next().unwrap().unwrap();

                            if ans.trim() == "y" {
                                db.create_db(&db_name);
                            }
                        }
                    }
                    Commands::Unmerge { key_a, key_b } => {
                        db.current().unmerge(&key_a, &key_b);
                    },
                    Commands::Snapshot => {
                        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();

                        let addr = server.server_addr();
                        let url = format!("http://{addr}");

                        open::that(url).expect("Server Error");

                        let request = server.recv().unwrap();

                        let html = snapshot::render(db.current());
                        let response = tiny_http::Response::from_string(html)
                            .with_header(
                                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..]).unwrap()
                            );

                        request.respond(response).unwrap()
                    },
                    Commands::SEED => {
                        if db.current().is_empty() {
                            db.current().seed();
                        } else {
                            println!("既存のデータがあります。SEEDを実行しますか？ (y/n)");

                            let ans= lines.next().unwrap().unwrap();

                            if ans.trim() == "y" {
                                db.current().seed();
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
}
