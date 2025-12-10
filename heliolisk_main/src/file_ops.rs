use std::path::PathBuf;

use crate::buffer::HBuffer;

#[allow(dead_code)]
pub fn buffer_to_string(bf: HBuffer) -> String {
    let mut str = String::new();
    bf.lines.iter().for_each(|s| {
        str.push_str(&s.text);
        str.push_str("\n");
    });
    str
}

#[allow(dead_code)]
pub fn write_string_to_file(file_name: Option<String>) -> Result<(), String> {
    match file_name {
        Some(s) => {
            let file_path = PathBuf::from(s);
            if file_path.is_dir() {
                return Err(String::from("Couldn't open path, file is a directory!"));
            }
        }
        None => {
            return Err(String::from("Error occurred while reading "));
        }
    }

    Ok(())
}
