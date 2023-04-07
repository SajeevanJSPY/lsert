use crate::file_types::xml::read_xml_file;
use crate::lexical_analysis::Lexer;
use std::collections::HashMap;
use std::fmt::{Display, Error as LogError, Formatter};
use std::fs::{self, File};
use std::io::{Error, ErrorKind, Result as IOResult};
use std::path::PathBuf;

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

pub struct IOControl;

impl IOControl {
    pub fn new(path: PathBuf, json_path: &str, deep: bool, progress: bool) -> IOResult<()> {
        let mut tfi = TermFreqIndex::new();

        if path.is_file() {
            let content = Self::read_file(&path, progress)?;
            tfi.insert(path, content);
        } else if path.is_dir() {
            Self::read_dir(&path, &mut tfi, deep, progress)?;
        } else {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Cannot handle the path type",
            ));
        }

        let file = File::create(json_path)?;
        serde_json::to_writer(file, &tfi)?;
        Ok(())
    }

    //  Possible Errors
    //      path doesn't exist - NotFound
    //      lacks permission to view content - PermissionDenied
    //      points at a non-directory file - NotADirectory
    fn read_dir(
        path: &PathBuf,
        tfi: &mut TermFreqIndex,
        deep: bool,
        progress: bool,
    ) -> IOResult<()> {
        let dir = fs::read_dir(&path)?;

        for dir_entry in dir {
            let dir_path = dir_entry?.path();

            let tf_option = if dir_path.is_file() {
                Some(Self::read_file(&dir_path, progress)?)
            } else {
                None
            };

            if dir_path.is_dir() && deep {
                Self::read_dir(&dir_path, tfi, deep, progress)?;
            }

            if let Some(tf) = tf_option {
                tfi.insert(dir_path, tf);
            }
        }

        Ok(())
    }

    //  Possible Errors:
    //      Not Found (Cannot Tokenize)
    fn read_file(path: &PathBuf, progress: bool) -> std::io::Result<TermFreq> {
        // TODO: Handle Errors
        let path_extension = path.extension();
        let mut tf = TermFreq::new();

        if let Some(extension_osstr) = path_extension {
            if let Some(extension) = extension_osstr.to_str() {
                let content_option = match extension {
                    "xhtml" | "html" | "xml" => {
                        if progress {
                            println!("Indexing {:?}", path);
                        }
                        Some(read_xml_file(path)?)
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
