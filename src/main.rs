use anyhow::Result;
use resp::Value;
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpListener, net::TcpStream};

mod resp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, _)) => {
                println!("Accepted new connection!");
                tokio::spawn(async move {
                    // process(socket).await;
                    handle_conn(stream).await
                });
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}

async fn _process(mut client_socket: TcpStream) {
    let (mut reader, mut writer) = client_socket.split();
    let mut buf = [0; 512];

    loop {
        match reader.read(&mut buf).await {
            Ok(bytes_read) => {
                println!(
                    "bytes read: {} | BUF: {:?}",
                    bytes_read,
                    &buf[0..bytes_read + 1]
                );
                println!("String read: {}", String::from_utf8_lossy(&buf));
                if bytes_read == 0 {
                    break;
                }

                // read lines from the buffer using read_line
                // let mut lines = BufReader::new(&buf[..bytes_read]).lines();
                // while let Some(line) = lines.next_line().await.unwrap() {
                //     println!("line: {}\n", line);
                // }

                if String::from_utf8_lossy(&buf[8..12]) == "echo" {
                    let mut count_str = String::new();
                    let mut idx = 14;
                    loop {
                        if buf[idx] == b'\\' {
                            break;
                        }
                        count_str = count_str + &String::from_utf8_lossy(&buf[idx..idx + 1]);
                        idx = idx + 1;
                    }
                    let count = count_str.parse::<i32>().unwrap();

                    let echo = String::from_utf8_lossy(&buf[idx + 2..idx + 2 + count as usize]);
                    println!("Echo: {}", echo);
                    writer
                        .write_all(format!("{}\r\n", echo).as_bytes())
                        .await
                        .unwrap();
                } else {
                    writer.write_all(b"+PONG\r\n").await.unwrap();
                }
            }
            Err(e) => println!("Error: {:?}", e),
        };
    }
}

async fn handle_conn(stream: TcpStream) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                _ => panic!("Cannot handle command"),
            }
        } else {
            break;
        };

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_string(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Invalid value type")),
    }
}

fn unpack_bulk_string(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Invalid value type")),
    }
}
