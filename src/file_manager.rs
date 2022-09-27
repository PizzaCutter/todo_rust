use std::fs::File;
use std::io::prelude::*;
use std::fs;

pub struct FileManager {
    pub data : String
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            data : String::default()
        }
    }

    pub fn initialize(&self) {
        // TODO[rsmekens]: read all files from specific directory

        self.load_files();
    }

    fn load_files(&self) {
        let paths = fs::read_dir("./data").unwrap();

        for path in  paths {
            println!("Name: {}", path.unwrap().path().display());
        }

        let file_to_open = String::from("data/2022_09_27.todo");
        let file_open= File::open(&file_to_open);
        let mut file_open_result;
        match file_open {
            Result::Ok(val) => { 
                file_open_result = val;
                println!("Successfully loaded file {}", file_to_open);
            }
            Result::Err(err) => {
                println!("Failed to load file! {:?}", err);
                return;
            }
        }

        let mut contents = String::new();
        file_open_result.read_to_string(&mut contents).unwrap();

        println!("Contents from file: \n{}", contents);
    }
}