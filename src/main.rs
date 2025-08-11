use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufReader};
use std::fs;
use std::path::Path;
use std::thread;

struct Headers {
    method: String,
    path: String,
    version: String,
}

fn parse_request(request: &str) -> Headers {
    let method = request
        .lines()
        .next()
        .unwrap_or("GET")
        .to_string()
        .split_whitespace()
        .next()
        .unwrap_or("GET")
        .to_string();

    let path = request
        .lines()
        .next()
        .unwrap_or("GET")
        .to_string()
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string();

    let version = request
        .lines()
        .next()
        .unwrap_or("GET")
        .to_string()
        .split_whitespace()
        .nth(2)
        .unwrap_or("HTTP/1.1")
        .to_string();

    Headers{
        method,
        path,
        version,
    }

}

fn get_content_type(file_type: &str) -> &str {
    let content_types = HashMap::from([
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("gif", "image/gif"),
        ("mp4", "video/mp4"),
        ("pdf", "application/pdf")
    ]);

    match content_types.get(file_type) {
        None => {
            //return png type if doesnt match the ones i could be bothered to add
            return "image/png"
        }
        Some(content_type) => {
            return content_type
        }
    };
}

fn serve_html(headers: &Headers) -> Vec<u8>{
    println!("{}", headers.path);

    let file = match fs::read_to_string(&headers.path) {
        Ok(file) => file,
        Err(_) => {
            return format!("{} 404 Not found\r\nContent-Type: text/plain\r\n\r\nFile not found\n", headers.version).into_bytes();
        }
    };

    let status = "200 OK";

    format!("{} {}\r\nContent-Type: text/html\r\n\r\n{}\n", headers.version, status, file).into_bytes()

}

//i hate Vec<u8>
fn serve_other_file(headers: &Headers, file_type: String) -> Vec<u8> {

    let content_type = get_content_type(&file_type);

    let file = match fs::File::open(&headers.path) {
        Ok(file) => file,
        Err(_) => {
            return format!("{} 404 Not found\r\nContent-Type: text/plain\r\n\r\nFile not found\n", headers.version).into_bytes()
        }
    };

    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    //rust why do you make me actually do error checking
    reader.read_to_end(&mut buffer)
        .expect("Couldnt read buffer, not my problem");

    let buffer_length = buffer.len();

    let mut response = format!(
        "{} 200 Ok\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n", 
        headers.version,
        content_type,
        buffer_length)
        .into_bytes();

    response.append(&mut buffer);

    response
}

fn handle_client(mut stream: &TcpStream) -> Vec<u8> {

    //1kb buffer to read the incoming data from the stream
    let mut buffer = [0; 1024];
    // read bytes
    let bytes = stream.read(&mut buffer[..]).expect("Unable to read from buffer");
    //bytes to string
    let request = String::from_utf8_lossy(&buffer[..bytes]).to_string();

    let mut request = parse_request(&request);

    //getting file extension requested
    match Path::new(&request.path).extension() {
        //path is / or /example, http path, so need to append .html to end and send that
        None => {
            if request.path == "/" {
                request.path = format!("./pages/index.html");
            } else {
                request.path = format!("./pages/sub{}.html", request.path);
            }

            println!("{}", request.path);

            serve_html(&request)
        }
        Some(file_type) => {
            let file_type = file_type
                .to_str()
                .unwrap_or("png")
                .to_string();

            request.path = format!("./pages/assets/{}", request.path);

            serve_other_file(
                &request, 
                file_type
            )
        }
    }

}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {

                thread::spawn(move || {
                    let ret = handle_client(&mut stream);
                    stream.write(&ret)
                        .expect("Couldnt write to socket");
                });

            },

            Err(e) => eprintln!("Connection failed: {e}"),
        }
    }
}
