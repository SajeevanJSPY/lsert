mod args;
mod file_types;
mod io_control;
mod lexical_analysis;
mod serve;

use args::OsStrExt;
use io_control::{IOControl, LogLevel};
use serve::Serve;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process;
use tiny_http::Server;

pub struct Args {
    options: Options,
    command: Option<Command>,
}

enum Command {
    Index(OsString),
    Serve,
}

enum ArgLogging {
    CommandError,
    OptionError,
    ArgumentError,
}

struct Options {
    deep: bool,
    address: Option<OsString>,
    json: Option<OsString>,
    progress: bool,
}

impl ArgLogging {
    fn error_logging(&self, error_msg: String) {
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
        process::exit(1);
    }
}

fn main() {
    // Getting Arguments
    let mut env = std::env::args_os();

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

    // Skipping bin
    env.next();

    // Main Command
    if let Some(main_command) = env.next() {
        match main_command.to_str().unwrap() {
            "index" => {
                if let Some(arg) = env.next() {
                    if arg.starts_with("-") || arg.starts_with("--") {
                        ArgLogging::ArgumentError
                            .error_logging(format!("{arg:?} is not valid argument for index"));
                    } else {
                        args.command = Some(Command::Index(arg));
                    }
                } else {
                    ArgLogging::ArgumentError.error_logging("Argument not found".to_string());
                };
            }
            "serve" => {
                args.command = Some(Command::Serve);
            }
            "help" => {
                ArgLogging::man_page();
            }
            _ => {
                ArgLogging::CommandError
                    .error_logging(format!("{main_command:?} is not a valid command"));
            }
        }
    } else {
        ArgLogging::CommandError.error_logging("Command Not Found".to_string());
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
                            args.options.deep = true;
                        } else if val == "false" {
                            args.options.deep = false;
                        } else {
                            ArgLogging::OptionError.error_logging(format!(
                                "{:?} not a valid value for {:?}",
                                val,
                                long.unwrap()
                            ));
                        }
                    } else {
                        args.options.deep = true;
                    };
                }
                Ok("progress") => {
                    if let Some(val) = value {
                        if val == "true" {
                            args.options.progress = true;
                        } else if val == "false" {
                            args.options.progress = false;
                        } else {
                            ArgLogging::OptionError.error_logging(format!(
                                "{:?} not a valid value for {:?}",
                                val,
                                long.unwrap()
                            ));
                        }
                    } else {
                        args.options.progress = true;
                    };
                }
                Ok("json") => {
                    if let Some(val) = value {
                        args.options.json = Some(val.to_os_string());
                    } else {
                        ArgLogging::OptionError
                            .error_logging(format!("Provide a value for {:?}", long.unwrap()));
                    };
                }
                Ok("address") => {
                    if let Some(val) = value {
                        args.options.address = Some(val.to_os_string());
                    } else {
                        ArgLogging::OptionError
                            .error_logging(format!("Provide a value for {:?}", long.unwrap()));
                    };
                }
                _ => {
                    ArgLogging::OptionError.error_logging(format!("{:?} not a valid option", long));
                }
            }
        } else if let Some(mut shorts) = arg.to_short() {
            while let Some(short) = shorts.next_flag() {
                match short {
                    Ok('h') => {
                        ArgLogging::man_page();
                    }
                    Ok('d') => {
                        args.options.deep = true;
                    }
                    _ => {
                        println!("Developing On Going");
                    }
                }
            }
        }
    }

    implication(args);
}

/*
    Args { options: Options { deep: false, address: None, json: Some("sajeevan"), progress: false }, command: Some(Serve) }
*/
fn implication(args: Args) {
    let json_path = if let Some(json_path_os) = &args.options.json {
        json_path_os.to_str().unwrap_or(default::JSON_PATH)
    } else {
        // TODO: Printing a message that we are using default json path
        default::JSON_PATH
    };

    match args.command.unwrap() {
        /* Indexing */
        Command::Index(dir_entry) => {
            if let Some(folder_path) = dir_entry.to_str() {
                LogLevel::SIGNAL(format!("Indexing...   {}", folder_path).to_string()).show();
                let entry = PathBuf::from(folder_path);

                if let Err(err) = IOControl::new(entry, &json_path) {
                    println!("{:?}", err);
                }
            } else {
                ArgLogging::ArgumentError
                    .error_logging(format!("Provide a valid argumentfor path"));
            };
        }
        /* Serving */
        Command::Serve => { 
            LogLevel::SIGNAL("Serving...".to_string()).show();

            let address = if let Some(address_os) = &args.options.address {
                address_os.to_str().unwrap_or(default::ADDRESS)
            } else {
                default::ADDRESS
            }; 

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

mod default {
    pub(crate) const JSON_PATH: &'static str = "./index.json";
    pub(crate) const ADDRESS: &'static str = "127.0.0.1:1919";
}
