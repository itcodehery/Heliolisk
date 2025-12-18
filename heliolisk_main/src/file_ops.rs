use std::{fs, path::PathBuf};

use crate::buffer::HBuffer;

#[allow(dead_code)]
pub fn buffer_to_string(bf: &HBuffer) -> String {
    let mut str = String::new();
    bf.lines.iter().for_each(|s| {
        str.push_str(&s.text);
        str.push_str("\n");
    });
    str
}

#[allow(dead_code)]
pub fn write_string_to_file(contents: String, file_name: Option<String>) -> Result<(), String> {
    match file_name {
        Some(s) => {
            let file_path = PathBuf::from(s);
            if file_path.is_dir() {
                return Err(String::from("Couldn't open path, file is a directory!"));
            }
            match fs::write(file_path, contents) {
                Ok(()) => {}
                Err(s) => return Err(s.to_string()),
            }
        }
        None => match fs::write("helios_test.txt", contents) {
            Ok(()) => {}
            Err(s) => return Err(s.to_string()),
        },
    }

    Ok(())
}
