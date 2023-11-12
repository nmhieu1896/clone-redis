// use std::{io::Write, net::TcpListener};
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpListener, net::TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("Failed to bind");

    loop {
        let (socket, _) = listener.accept().await?;

        // tokio::spawn(async move {
        process(socket).await;
        // });
    }
}

async fn process(mut client_socket: TcpStream) {
    let (mut reader, mut writer) = client_socket.split();
    let mut buf = [0; 1024];

    loop {
        match reader.read(&mut buf).await {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                writer.write_all(b"+PONG\r\n").await.unwrap();
            }
            Err(e) => println!("Error: {:?}", e),
        };
    }
}
