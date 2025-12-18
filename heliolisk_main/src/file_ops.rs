use std::{fs, path::PathBuf};

use crate::buffer::HBuffer;

#[allow(dead_code)]
pub fn buffer_to_string(bf: &HBuffer) -> String {
    bf.text.to_string()
}

use std::fs::File;
use std::io::BufWriter;

pub fn write_buffer_to_file(bf: &HBuffer, file_name: Option<String>) -> Result<(), String> {
    let actual_name = file_name.unwrap_or_else(|| "helios_test.txt".to_string());
    let file_path = PathBuf::from(&actual_name);

    if file_path.is_dir() {
        return Err(String::from("Couldn't open path, file is a directory!"));
    }

    let parent = file_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let file_stem = file_path.file_name().unwrap_or_default().to_string_lossy();
    let temp_name = format!(".{}.tmp", file_stem);
    let temp_path = parent.join(temp_name);

    let file = File::create(&temp_path).map_err(|e| e.to_string())?;
    // Use BufWriter for better performance
    let writer = BufWriter::new(file);

    if let Err(e) = bf.text.inner.write_to(writer) {
        let _ = fs::remove_file(&temp_path);
        return Err(e.to_string());
    }

    fs::rename(&temp_path, &file_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        e.to_string()
    })?;

    Ok(())
}

use crate::rope::HeliosRope;

pub fn load_file(file_path: &PathBuf) -> Result<HBuffer, String> {
    if !file_path.exists() {
        return Err(format!("File not found: {:?}", file_path));
    }

    // Read the file content
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;

    // Create buffer
    let buffer = HBuffer {
        text: HeliosRope::from_str(&content),
        file_format: file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("txt")
            .to_string(),
        file_path: Some(file_path.to_string_lossy().to_string()),
    };

    dbg!(buffer.text.len_lines());

    Ok(buffer)
}
