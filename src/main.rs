use tcpproxy::ThreadPool;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Entry point of the application.
///
/// This function sets up a TCP listener on the local host at the specified port,
/// creates a thread pool with 4 threads, and then enters a loop where it
/// accepts incoming connections and handles them in separate threads.
///
/// The function expects three command-line arguments:
/// 1. The domain of the web server to proxy requests to.
/// 2. The port of the web server to proxy requests to.
/// 3. The port on which the proxy server should listen for incoming connections.
fn main() {
    // Get the command-line arguments.
    let args: Vec<String> = env::args().collect();

    // Check if the correct number of arguments were provided.
    if args.len() != 4 {
        println!("Usage: tcpproxy <web_server_domain> <web_server_port> <proxy_port>");
        return;
    }

    // Extract the arguments.
    let web_server_domain = &args[1];
    let web_server_port = &args[2];
    let proxy_port = &args[3];

    // Create a TCP listener at the specified address and port.
    let listener = TcpListener::bind(format!("127.0.0.1:{}", proxy_port)).unwrap();

    // Create a thread pool with 4 threads.
    let pool = ThreadPool::build(4).unwrap();

    // Loop over the incoming connections from the listener.
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let web_server_domain = web_server_domain.clone();
        let web_server_port = web_server_port.clone();

        // For each connection, spawn a job in the thread pool to handle it.
        // The job is a closure that calls the handle_connection function.
        pool.execute(move || {
            handle_connection(stream, &web_server_domain, &web_server_port);
        });
    }

    println!("Shutting down.");
}

/// Handle a single connection from a client.
///
/// This function reads the request from the client, forwards it to the backend server,
/// reads the response from the backend server, and then writes the response back to the client.
///
/// # Arguments
///
/// * `client_stream` - A TCP stream that represents the client connection.
/// * `web_server_domain` - The domain of the web server to proxy requests to.
/// * `web_server_port` - The port of the web server to proxy requests to.
fn handle_connection(mut client_stream: TcpStream, web_server_domain: &str, web_server_port: &str) {
    // Buffer to store the incoming request data.
    let mut buffer = [0; 1024];
    // Read the incoming request data into the buffer.
    client_stream.read(&mut buffer).unwrap();

    // Create a TCP stream to the backend server.
    let mut backend_stream = TcpStream::connect(format!("{}:{}", web_server_domain, web_server_port)).unwrap();
    // Write the client's request to the backend server.
    backend_stream.write_all(&buffer).unwrap();

    // Buffer to store the response data from the backend server.
    let mut response = String::new();
    // Read the response data from the backend server into the buffer.
    backend_stream.read_to_string(&mut response).unwrap();

    // Write the backend server's response back to the client.
    client_stream.write_all(response.as_bytes()).unwrap();
    // Flush the client stream to ensure the response is sent.
    client_stream.flush().unwrap();
}