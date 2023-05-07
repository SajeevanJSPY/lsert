use crate::file_types::{
    read_xml_file,
    read_plain_file
};
use crate::lexical_analysis::Lexer;
use std::collections::HashMap;
use std::fmt::{Display, Error as LogError, Formatter};
use std::path::PathBuf;
use std::fs;
use redis::{Commands, Connection};

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

pub struct IOControl {
    path: PathBuf,
    json_path: String,
    deep: bool,
    progress: bool,
    con: Connection
}

impl IOControl {
    pub fn new(path: PathBuf, json_path: &str, deep: bool, progress: bool, con: Connection) -> Self {
        Self {
            path,
            json_path: json_path.to_string(),
            deep,
            progress,
            con
        }
    }

    pub fn check_file_type(&mut self) -> redis::RedisResult<()> {
        let path = &self.path.clone();

        if path.is_file() {
            self.read_file(path)?;
        } else if path.is_dir() {
            self.read_dir(path)?;
        }

        Ok(())
    }

    //  Possible Errors
    //      path doesn't exist - NotFound
    //      lacks permission to view content - PermissionDenied
    //      points at a non-directory file - NotADirectory
    fn read_dir(&mut self, path: &PathBuf) -> redis::RedisResult<()> {
        let dir = fs::read_dir(&path)?;

        for dir_entry in dir {
            let dir_path = dir_entry?.path();

            if dir_path.is_file() {
                self.read_file(&dir_path)?;
            }

            if dir_path.is_dir() && self.deep {
                self.read_dir(&dir_path)?;
            }
        }

        Ok(())
    }

    //  Possible Errors:
    //      Not Found (Cannot Tokenize)
    fn read_file(&mut self, path: &PathBuf) -> redis::RedisResult<()> {
        let path_extension = path.extension();

        if let Some(extension_osstr) = path_extension {
            if let Some(extension) = extension_osstr.to_str() {
                let content_option = match extension {
                    "xhtml" | "html" | "xml" => {
                        if self.progress {
                            println!("Indexing {:?}", path);
                        }
                        Some(read_xml_file(path)?)
                    }
                    "txt" => {
                        if self.progress {
                            println!("Indexing {:?}", path);
                        }
                        Some(read_plain_file(path)?)
                    }
                    _ => {
                        LogLevel::WARN(format!("Cannot Tokenize {}", path.display())).show();
                        None
                    }
                };

                if let Some(content) = content_option {
                    let path = path.to_str().unwrap();
                    // Add a caching folder
                    self.con.hset(path, ".", 0)?;
                    let char_slice = content.chars().collect::<Vec<_>>();
                    let lexer = Lexer::new(&char_slice);

                    for token in lexer {
                        let token_exists = self.con.hexists::<&str, String, u8>(path, token.clone())?;
                        if token_exists == 1 {
                            self.con.hincr(path, token, 1)?;
                        } else {
                            self.con.hset(path, token, 1)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub enum LogLevel {
    ERROR(String),
    WARN(String),
    SIGNAL(String),
}

impl Display for LogLevel {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), LogError> {
        match self {
            LogLevel::ERROR(err) => eprint!("\x1B[41m\x1B[1mERROR:\x1B[0m {err}"),
            LogLevel::WARN(warn) => eprint!("\x1B[44m\x1B[1mWARN:\x1B[0m {warn} "),
            LogLevel::SIGNAL(signal) => print!("\n\x1B[1m{signal}\x1B[0m\n"),
        };

        Ok(())
    }
}

impl LogLevel {
    pub fn show(&self) {
        println!("{}", self);
    }
}
