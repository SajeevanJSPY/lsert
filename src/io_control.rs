use crate::file_types::{
    read_xml_file,
    read_plain_file
};
use crate::lexical_analysis::Lexer;
use std::collections::HashMap;
use std::fmt::{Display, Error as LogError, Formatter};
use std::fs::{self, File};
use std::io::{Error, ErrorKind, Result as IOResult};
use std::path::PathBuf;

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

pub struct IOControl {
    path: PathBuf,
    json_path: String,
    deep: bool,
    progress: bool,
}

impl IOControl {
    pub fn new(path: PathBuf, json_path: &str, deep: bool, progress: bool) -> Self {
        Self {
            path,
            json_path: json_path.to_string(),
            deep,
            progress,
        }
    }

    pub fn check_file_type(&self) -> IOResult<()> {
        let mut tfi = TermFreqIndex::new();
        let path = &self.path;

        if path.is_file() {
            let content = self.read_file(path)?;
            tfi.insert(path.clone(), content);
        } else if path.is_dir() {
            self.read_dir(path, &mut tfi)?;
        } else {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Cannot handle the path type",
            ));
        }

        let file = File::create(&self.json_path)?;
        serde_json::to_writer(file, &tfi)?;
        Ok(())
    }

    //  Possible Errors
    //      path doesn't exist - NotFound
    //      lacks permission to view content - PermissionDenied
    //      points at a non-directory file - NotADirectory
    fn read_dir(&self, path: &PathBuf, tfi: &mut TermFreqIndex) -> IOResult<()> {
        let dir = fs::read_dir(&path)?;

        for dir_entry in dir {
            let dir_path = dir_entry?.path();

            let tf_option = if dir_path.is_file() {
                Some(self.read_file(&dir_path)?)
            } else {
                None
            };

            if dir_path.is_dir() && self.deep {
                self.read_dir(&dir_path, tfi)?;
            }

            if let Some(tf) = tf_option {
                tfi.insert(dir_path, tf);
            }
        }

        Ok(())
    }

    //  Possible Errors:
    //      Not Found (Cannot Tokenize)
    fn read_file(&self, path: &PathBuf) -> std::io::Result<TermFreq> {
        // TODO: Handle Errors
        let path_extension = path.extension();
        let mut tf = TermFreq::new();

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
                    let char_slice = content.chars().collect::<Vec<_>>();
                    let lexer = Lexer::new(&char_slice);

                    for token in lexer {
                        if let Some(tok) = tf.get_mut(&token) {
                            *tok += 1;
                        } else {
                            tf.insert(token, 1);
                        }
                    }
                }
            }
        }

        Ok(tf)
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
