use std::io;

use crate::Ufdb;

pub fn save(ufdb: &Ufdb, db_name: &str) -> io::Result<()> {
    std::fs::create_dir_all("./ufo_data")?;

    let json = serde_json::to_string_pretty(ufdb)?;

    let path = format!("./ufo_data/{db_name}.json");

    std::fs::write(path, json)?;

    Ok(())
}

pub fn load(db_name: &str) -> io::Result<Option<Ufdb>> {
    let path = format!("./ufo_data/{db_name}.json");

    let json = match std::fs::read_to_string(path) {
        Ok(json) => json,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };

    let ufdb: Ufdb = serde_json::from_str(&json)?;

    Ok(Some(ufdb))
}
