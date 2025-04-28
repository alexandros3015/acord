use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::net::TcpStream;
use std::io::{Write, self};


macro_rules! input {
    () => {{
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim_end().to_string()
    }};

    ($($arg:tt)*) => {{
        print!($($arg)*);
        io::stdout().flush().unwrap();

        input!()
    }};
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("localhost:8080").await?;
    let (read_half, mut write_half) = tokio::io::split(stream);

    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        let msg = input!("Enter a message: ");
        write_half.write_all(msg.as_bytes()).await?;
        write_half.write_all(b"\n").await?;
        write_half.flush().await?;

        line.clear();
        let n = reader.read_line(&mut line).await?;

        if n == 0 {
            println!("Connection closed");
            break;
        }

        if msg == "!exit" {
            println!("Exiting");
            break;
        }
        
        println!("Received: {}", line);
    }

    Ok(())
}