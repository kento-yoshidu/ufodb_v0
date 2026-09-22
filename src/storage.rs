use std::{io, path::Path};

use crate::Ufdb;

pub fn data_dir() -> io::Result<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "ufodb")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "ホームディレクトリを特定できませんでした"))?;

    Ok(dirs.data_local_dir().to_path_buf())
}

pub fn save(ufdb: &Ufdb, db_name: &str, dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;

    let json = serde_json::to_string_pretty(ufdb)?;

    let path = dir.join(format!("{db_name}.json"));

    std::fs::write(path, json)?;

    Ok(())
}

pub fn load(db_name: &str, dir: &Path) -> io::Result<Option<Ufdb>> {
    let path = dir.join(format!("{db_name}.json"));

    let json = match std::fs::read_to_string(path) {
        Ok(json) => json,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };

    let ufdb: Ufdb = serde_json::from_str(&json)?;

    Ok(Some(ufdb))
}
