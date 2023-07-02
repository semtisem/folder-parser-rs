use csv::Writer;
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io;
use walkdir::WalkDir;

const MAX_LENGTH: usize = 150;

#[derive(Debug, Serialize)]
struct NodeCsv<'a> {
    path: String,
    restriction: &'a str,
}

#[derive(Debug, Serialize)]
struct PathCSV {
    path: String,
    should_be_data_room: bool,
}

#[derive(Debug, Serialize)]
enum RestrictionError {
    MaximumCharacters,
    CsvWriterError,
}

impl RestrictionError {
    fn as_str(&self) -> &'static str {
        match self {
            RestrictionError::MaximumCharacters => "Maximum characters of 150 reached",
            &RestrictionError::CsvWriterError => "CSV Error",
        }
    }
}
impl Display for RestrictionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl Error for RestrictionError {}

fn check_restrictions(
    node_name: String,
    path: String,
    writer: &mut Writer<File>,
) -> Result<(), RestrictionError> {
    match node_name {
        node_name if node_name.len() > MAX_LENGTH => writer
            .serialize(NodeCsv {
                path,
                restriction: RestrictionError::MaximumCharacters.as_str(),
            })
            .map_err(|_| RestrictionError::CsvWriterError)?,
        _ => (),
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // Get the target directory path from command-line arguments
    let args: Vec<String> = env::args().collect();
    let target_dir = args
        .get(1)
        .expect("Please provide a directory path as an argument.");

    // Create CSV writers
    let file_folder_struct = fs::File::create("folder_structure.csv")?;
    let file_folder_name_restrictions = fs::File::create("folder_name_restrictions.csv")?;
    let file_filder_name_restrictions = fs::File::create("file_name_restrictions.csv")?;

    let mut writer_struct = csv::Writer::from_writer(file_folder_struct);
    let mut writer_folder_restrictions = csv::Writer::from_writer(file_folder_name_restrictions);
    let mut writer_file_name_restrictions = csv::Writer::from_writer(file_filder_name_restrictions);
    // Parse the folder structure and write to CSV

    for entry in WalkDir::new(&target_dir).into_iter().filter_map(Result::ok) {
        let folder_path = entry
            .path()
            .display()
            .to_string()
            .replace("\\", "/")
            .replace("//", "\\");
        if entry.file_type().is_dir() {
            writer_struct.serialize(PathCSV {
                path: folder_path.clone(),
                should_be_data_room: false,
            })?;
            let folder_name = entry.file_name().to_string_lossy().to_string();
            check_restrictions(
                folder_name,
                folder_path.clone(),
                &mut writer_folder_restrictions,
            )
            .unwrap();
        }
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            check_restrictions(
                file_name,
                folder_path.clone(),
                &mut writer_file_name_restrictions,
            )
            .unwrap();
        }
    }

    writer_struct.flush()?;
    println!("Folder structure exported to 'folder_structure.csv'.");
    println!("To long folder names are exported to 'folder_name_to_long.csv'.");
    Ok(())
}
