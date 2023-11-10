use tokio::{io::AsyncWriteExt, net::TcpListener, net::TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("Failed to bind");

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(mut client_socket: TcpStream) {
    let (mut ___, mut writer) = client_socket.split();
    let _ = writer.write(b"+PONG\r\n").await;
}
