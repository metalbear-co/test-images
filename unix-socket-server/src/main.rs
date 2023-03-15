use tokio::net::{UnixStream, UnixListener};

async fn handle_stream(mut stream: UnixStream) {
    let (mut reader, mut writer) = stream.split();
    tokio::io::copy(&mut reader, &mut writer).await.unwrap();
}

#[tokio::main]
async fn main() {
    let listener = UnixListener::bind("/app/unix-socket-server.sock").unwrap();
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("Incoming connection from {addr:?}");
        tokio::spawn(handle_stream(stream));
    }
}
