pub mod file_types;
pub mod io_control;
pub mod lexical_analysis;
mod serve;

/*
// TODO: Handling Every Single Error: Without Unwrap or ?, Cannot fail for a single file error
fn args_parser() {
    let mut args = std::env::args();
    args.next();

    if let Some(options) = args.next() {
        match options.as_str() {
            /*      Indexing        */
            "index" => {
                let folder_path = args.next().unwrap();
                let json_path = args.next().unwrap_or(default::JSON_PATH.to_string());

                LogLevel::SIGNAL(format!("Indexing...   {}", folder_path).to_string()).show();

                // Creatind Reading
                let entry = PathBuf::from(folder_path);

                if let Err(err) = IOControl::new(entry, &json_path) {
                    println!("{:?}", err);
                }

                // let dir = read_dir(&folder_path, &json_path);

                /*
                if let Err(err) = dir {
                    LogLevel::ERROR(format!("{folder_path}: {err}")).show();
                } else {
                    LogLevel::SIGNAL(format!(
                        "Read the Directory: {folder_path}\nCreated JSON File: {json_path}"
                    ))
                    .show();
                }
                */
            }

            /*      Serving         */
            "serve" => {
                LogLevel::SIGNAL("Serving...".to_string()).show();
                let server = Server::http(default::ADDRESS).unwrap();
                println!("➜  Local:   http://{}", default::ADDRESS);

                loop {
                    let request = match server.recv() {
                        Ok(rq) => rq,
                        Err(e) => {
                            println!("error: {}", e);
                            break;
                        }
                    };

                    let serve = Serve::new(request);
                    serve.handle_connection().unwrap();
                }
            }

            _ => println!("Invalid Option"),
        }
    }
}

mod default {
    pub(crate) const JSON_PATH: &'static str = "./index.json";
    pub(crate) const ADDRESS: &'static str = "127.0.0.1:1919";
}
*/
