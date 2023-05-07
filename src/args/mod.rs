mod args;
mod osstr_ext;
mod redis_client;

use crate::io_control::{IOControl, LogLevel};
use crate::serve::Serve;
use osstr_ext::OsStrExt;
use std::ffi::OsString;
use std::path::PathBuf;
use tiny_http::Server;

pub struct Args {
    options: Options,
    command: Option<Command>,
}

impl Args {
    pub fn new() -> Self {
        // default values for options
        let options = Options {
            deep: false,
            address: None,
            json: None,
            progress: false,
        };

        let mut args = Args {
            options,
            command: None,
        };

        args.implementation();
        args
    }

    pub fn implementation(&mut self) {
        // env
        let mut env = std::env::args_os();

        // Skip bin
        env.next();

        // Main Command
        if let Some(main_command) = env.next() {
            match main_command.to_str().unwrap() {
                "index" => {
                    if let Some(arg) = env.next() {
                        if arg.starts_with("-") || arg.starts_with("--") {
                            ArgLogging::error_log(format!(
                                "{arg:?} is not valid argument for index"
                            ));
                        } else {
                            self.command = Some(Command::Index(arg));
                        }
                    } else {
                        ArgLogging::error_log("Argument not found".to_string());
                    };
                }
                "serve" => {
                    self.command = Some(Command::Serve);
                }
                "help" => {
                    ArgLogging::man_page();
                }
                _ => {
                    ArgLogging::error_log(format!("{main_command:?} is not a valid command"));
                }
            }
        } else {
            ArgLogging::error_log("Command Not Found".to_string());
        };

        let raw = args::RawArgs::new(env.collect::<Vec<_>>());
        let mut cursor = raw.cursor();

        while let Some(arg) = raw.next(&mut cursor) {
            if arg.is_escape() {
            } else if arg.is_stdio() {
            } else if let Some((long, value)) = arg.to_long() {
                match long {
                    Ok("help") => {
                        ArgLogging::man_page();
                    }
                    Ok("deep") => {
                        if let Some(val) = value {
                            if val == "true" {
                                self.options.deep = true;
                            } else if val == "false" {
                                self.options.deep = false;
                            } else {
                                ArgLogging::error_log(format!(
                                    "{:?} not a valid value for {:?}",
                                    val,
                                    long.unwrap()
                                ));
                            }
                        } else {
                            self.options.deep = true;
                        };
                    }
                    Ok("progress") => {
                        if let Some(val) = value {
                            if val == "true" {
                                self.options.progress = true;
                            } else if val == "false" {
                                self.options.progress = false;
                            } else {
                                ArgLogging::error_log(format!(
                                    "{:?} not a valid value for {:?}",
                                    val,
                                    long.unwrap()
                                ));
                            }
                        } else {
                            self.options.progress = true;
                        };
                    }
                    Ok("json") => {
                        if let Some(val) = value {
                            self.options.json = Some(val.to_os_string());
                        } else {
                            ArgLogging::error_log(format!(
                                "Provide a value for {:?}",
                                long.unwrap()
                            ));
                        };
                    }
                    Ok("address") => {
                        if let Some(val) = value {
                            self.options.address = Some(val.to_os_string());
                        } else {
                            ArgLogging::error_log(format!(
                                "Provide a value for {:?}",
                                long.unwrap()
                            ));
                        };
                    }
                    _ => {
                        ArgLogging::error_log(format!("{:?} not a valid option", long));
                    }
                }
            } else if let Some(mut shorts) = arg.to_short() {
                while let Some(short) = shorts.next_flag() {
                    match short {
                        Ok('h') => {
                            ArgLogging::man_page();
                        }
                        Ok('d') => {
                            self.options.deep = true;
                        }
                        Ok('p') => {
                            self.options.progress = true;
                        }
                        _ => {
                            println!("Developing On Going");
                        }
                    }
                }
            }
        }
    }

    pub fn implication(self) {
        let json_path = self.options.json_path();
        let client = redis::Client::open(redis_client::CLIENT_INFO).unwrap();
        let con = client.get_connection().unwrap();

        match self.command.unwrap() {
            /* Indexing */
            Command::Index(dir_entry) => {
                if let Some(folder_path) = dir_entry.to_str() {
                    LogLevel::SIGNAL(format!("Indexing...   {}", folder_path).to_string()).show();
                    let entry = PathBuf::from(folder_path);

                    let mut io_control =
                        IOControl::new(entry, &json_path, self.options.deep, self.options.progress, con);
                    if let Err(err) = io_control.check_file_type() {
                        println!("{:?}", err);
                    }
                } else {
                    ArgLogging::error_log(format!("Provide a valid argumentfor path"));
                };
            }
            /* Serving */
            Command::Serve => {
                LogLevel::SIGNAL("Serving...".to_string()).show();
                let address = self.options.ip_address();

                let server = Server::http(address).unwrap();
                println!("➜  Local:   http://{}", address);

                loop {
                    let request = match server.recv() {
                        Ok(rq) => rq,
                        Err(e) => {
                            println!("error: {}", e);
                            break;
                        }
                    };

                    let serve = Serve::new(request);
                    serve.handle_connection(json_path).unwrap();
                }
            }
        }
    }
}

enum Command {
    Index(OsString),
    Serve,
}

struct Options {
    deep: bool,
    address: Option<OsString>,
    json: Option<OsString>,
    progress: bool,
}

impl Options {
    fn ip_address(&self) -> &str {
        let address = if let Some(address_os) = &self.address {
            address_os.to_str().unwrap_or(default::ADDRESS)
        } else {
            default::ADDRESS
        };

        address
    }

    fn json_path(&self) -> &str {
        let json_path = if let Some(json_path_os) = &self.json {
            json_path_os.to_str().unwrap_or(default::JSON_PATH)
        } else {
            // TODO: Printing a message that we are using default json path
            default::JSON_PATH
        };

        json_path
    }
}

// ArgLogging::CommandError.error_logging("Command Not Found".to_string());
struct ArgLogging;

impl ArgLogging {
    fn error_log(error_msg: String) {
        if !error_msg.is_empty() {
            LogLevel::ERROR(error_msg).show();
        }

        Self::man_page();
    }

    fn man_page() {
        println!(
            "
    Local Search Engine (Lsert)

    USAGE:
        <program> <command> [arguments] [options]

    command:  
        index
        serve
        help

    valid:
        index [file | folder] --json=<json_file.json>
        serve --json=<json_file>

    options:
        --json=<json_file>: JSON file to parse and get the data
        --address=<valid_ip_address>: Give an address to expose
        --deep=[true | false] | --deep | -d: Recursive the folder and try to get all data
        --progress=[true | false] | --progress | -p: Show the process
        "
        );

        LogLevel::SIGNAL("Exiting the Program...".to_string()).show();
        std::process::exit(1);
    }
}

mod default {
    pub(crate) const JSON_PATH: &'static str = "./index.json";
    pub(crate) const ADDRESS: &'static str = "127.0.0.1:1919";
}
