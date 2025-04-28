use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
  };
  use std::io::{self, Write};
  
  /// A tiny prompt‐and‐read helper
  macro_rules! prompt {
    ($fmt:expr $(, $arg:expr )* ) => {{
      print!($fmt $(, $arg )*);
      io::stdout().flush().unwrap();
      let mut buf = String::new();
      io::stdin().read_line(&mut buf).unwrap();
      buf.trim_end().to_string()
    }};
  }
  
  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // enter host:port
    let addr = prompt!("server addr> ");
    let nick = prompt!("name> ");
    println!("connecting to {}", addr);
    let stream = TcpStream::connect(addr).await?;
  
    // split into a read‐half and a write‐half
    let (read_half, mut write_half) = tokio::io::split(stream);
  
    // TASK A: read *any* server‐pushed lines and print them
    let mut server_lines = BufReader::new(read_half).lines();
    tokio::spawn(async move {
      while let Ok(Some(line)) = server_lines.next_line().await {
        println!("\r<< {}", line);
        print!("you> ");
        let _ = io::stdout().flush();
      }
      eprintln!("\n[-] server closed connection");
    });
  
    // TASK B (main): prompt the user & send each line to the server
    loop {
      let line = prompt!("> ");
      let formatted_line = format!("{nick}: {line}");

      write_half.write_all(formatted_line.as_bytes()).await?;
      write_half.write_all(b"\n").await?;
      write_half.flush().await?;

      if line == "!exit" { break; }
    }
  
    Ok(())
  }
  