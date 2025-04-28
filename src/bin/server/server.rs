use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let listener = TcpListener::bind("localhost:8080").await?;
    println!("Listening on port 8080");

    loop {

        let (socket, peer) = listener.accept().await?;
        println!("Accepted connection from {}", peer);

        tokio::spawn(async move {
                let (r, mut w) = tokio::io::split(socket);
                let mut reader = BufReader::new(r);
                let mut line = String::new();

                loop {
                    line.clear();

                    let n = reader.read_line(&mut line).await.unwrap();
                    if n == 0 {
                        println!("{} decided this chatting app isn't good enough", peer);
                        break;
                    }

                    let trimmed = line.trim_end();
                    println!("{}: {}", peer, trimmed);

                    if trimmed == "!exit" {
                        println!("{} decided to exit", peer);
                        break;
                    }

                    w.write_all(trimmed.as_bytes()).await.unwrap();
                    w.write_all(b"\n").await.unwrap();
                    w.flush().await.unwrap();
                }

        });

    }


}