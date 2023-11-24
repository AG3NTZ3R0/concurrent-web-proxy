use hello::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Entry point of the application.
fn main() {
    // Bind a TCP listener to the specified address and port.
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Create a thread pool with 4 threads.
    let pool = ThreadPool::build(4).unwrap();

    // Accept incoming connections, but only take the first 2.
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // For each connection, spawn a job in the thread pool to handle it.
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

/// Handle a single connection from a client.
///
/// This function reads the request from the client, determines the appropriate response,
/// and writes the response back to the client.
fn handle_connection(mut stream: TcpStream) {
    // Buffer to store the incoming request data.
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Define the HTTP methods and routes we will accept.
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // Determine the status line and filename based on the request.
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        // If the request is for /sleep, pause for 5 seconds before responding.
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        // If the request is for an unknown route, respond with a 404 status.
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    // Read the contents of the file to include in the response.
    let contents = fs::read_to_string(filename).unwrap();

    // Format the HTTP response.
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    // Write the response to the client and flush the stream to ensure it is sent.
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}