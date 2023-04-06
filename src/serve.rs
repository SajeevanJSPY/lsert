use crate::lexical_analysis::Lexer;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use tiny_http::{Header, Method, Request, Response, StatusCode};

pub struct Serve {
    req: Request,
}

#[allow(dead_code)]
impl Serve {
    pub fn new(req: Request) -> Self {
        Self { req }
    }

    //  Possible Errors:
    //          Read: Interrupted(Non Utf8)
    //          Empty Query: InvalidInput
    fn handle_post_method(mut self, json_path: impl AsRef<Path>) -> io::Result<String> {
        let mut body_data = String::new();
        self.req.as_reader().read_to_string(&mut body_data)?;

        let body_len = body_data.len();
        let body_data = body_data[1..(body_len - 1)].to_string();

        // If User Input is Empty
        if body_data.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Provide a valid query!",
            ));
        }

        let lexem = body_data.chars().collect::<Vec<_>>();
        let lexer = Lexer::new(&lexem);

        let vec_post_data = lexer.collect::<Vec<_>>();
        let vec = tf(vec_post_data, json_path);

        let response_data = serde_json::to_string(&vec).unwrap();

        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
        let response = Response::from_string(response_data).with_header(header);
        self.req.respond(response).unwrap();
        Ok(body_data)
    }

    //  Possible Errors ->
    //      File Open: NotFound, Permission Denied, AlreadyExists, InvalidInput
    //      Read: Interrupted(Non Utf8)
    // TODO: Implement Respond Properly
    pub fn handle_connection(self, json_path: impl AsRef<Path>) -> io::Result<()> {
        const WEB_FILE_DIR: &'static str = "files/web_files";
        println!("{} {:?}", self.req.method(), self.req.url());

        // TODO: Handling Requests Properly

        // Method Handling
        match self.req.method() {
            Method::Get => {
                println!("Handling Get Method");
                let (status_code, filename) = if self.req.url() == "/" {
                    (200, "index.html")
                } else {
                    (404, "404.html")
                };

                let filename = format!("{WEB_FILE_DIR}/{filename}");

                let contents = fs::read_to_string(filename)?;
                let readable_stream = contents.as_bytes();
                let content_length = contents.len();

                let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap();

                let response = Response::new(
                    StatusCode(status_code),
                    vec![header],
                    readable_stream,
                    Some(content_length),
                    None,
                );

                self.req.respond(response).unwrap();
            }
            Method::Post => {
                let post_data = self.handle_post_method(json_path);
                println!("Handling Post Method {:?}", post_data);
            }
            _ => {
                // TODO: Handle this methods
                println!("Not implemented for this method {:?}", self.req.method());
            }
        }

        Ok(())
    }

}


fn tf(post_data: Vec<String>, json_path: impl AsRef<Path>) -> Vec<(PathBuf, usize)> {
    type TermFreq = HashMap<String, usize>;
    type TermFreqIndex = HashMap<PathBuf, TermFreq>;
    type TermFreqPath = HashMap<PathBuf, usize>;

    let file = std::fs::File::open(json_path).unwrap();

    let u: TermFreqIndex = serde_json::from_reader(file).unwrap();

    let vec_term = u.into_iter().collect::<Vec<_>>();

    let mut documents = TermFreqPath::new();
    for (path, termfreq) in vec_term {
        let iter = post_data.iter();
        // Checking Paths to how many numbers
        let mut count = 0;

        for word in iter {
            if termfreq.contains_key(word.as_str()) {
                count += termfreq.get(word.as_str()).unwrap();
            }
        }

        if count != 0 {
            documents.insert(path, count);
        }
    }

    let mut documents_vec = documents.into_iter().take(20).collect::<Vec<_>>();
    documents_vec.sort_by_key(|(_, c)| *c);
    documents_vec.reverse();

    documents_vec
}
