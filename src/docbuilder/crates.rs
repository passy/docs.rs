
use ::DocBuilderError;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs;
use std::path::PathBuf;
use rustc_serialize::json::Json;


fn crates_from_file<F>(path: &PathBuf, func: &mut F) -> Result<(), DocBuilderError>
    where F: FnMut(&str, &str) -> () {

    let reader = try!(fs::File::open(path)
                      .map(|f| BufReader::new(f)));

    let mut name = String::new();
    let mut versions = Vec::new();

    for line in reader.lines() {
        // some crates have invalid UTF-8 (nanny-sys-0.0.7)
        // skip them
        let line = match line {
            Ok(l) => l,
            Err(_) => continue
        };
        let data = match Json::from_str(line.trim()) {
            Ok(d) => d,
            Err(_) => continue
        };

        let obj = try!(data.as_object().ok_or(DocBuilderError::JsonNotObject));
        let crate_name = try!(obj.get("name")
                              .and_then(|n| n.as_string())
                              .ok_or(DocBuilderError::JsonNameNotFound));
        let vers = try!(obj.get("vers")
                        .and_then(|n| n.as_string())
                        .ok_or(DocBuilderError::JsonVersNotFound));

        name.clear();
        name.push_str(crate_name);
        versions.push(format!("{}", vers));
    }

    if !name.is_empty() {
        versions.reverse();
        for version in versions {
            func(&name[..], &version[..]);
        }
    }

    Ok(())
}



pub fn crates_from_path<F>(path: &PathBuf, func: &mut F) -> Result<(), DocBuilderError>
    where F: FnMut(&str, &str) -> () {

    if !path.is_dir() {
        return Err(DocBuilderError::FileNotFound);
    }

    for file in try!(path.read_dir()) {
        let file = try!(file);
        let path = file.path();
        // skip files under .git and config.json
        if path.to_str().unwrap().contains(".git") ||
            path.file_name().unwrap() == "config.json" {
                continue;
            }

        if path.is_dir() {
            try!(crates_from_path(&path, func));
        } else {
            try!(crates_from_file(&path, func));
        }
    }

    Ok(())
}
