use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
  };
  use std::io::{self, Write};
  
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
    
    let ip = prompt!("Enter host: ");

    let listener = TcpListener::bind(&ip).await?;
    println!("listening on {}", ip);
 
    let (tx, _) = broadcast::channel::<String>(100);
  
    loop {
      
      let (socket, peer) = listener.accept().await?;
      println!(">> {} connected", peer);
      let tx_for_send = tx.clone();
      let mut rx = tx.subscribe();
  
      let (read_half, mut write_half) = tokio::io::split(socket);
      let mut client_lines = BufReader::new(read_half).lines();
 
      tokio::spawn(async move {
        loop {
          tokio::select! {
            
            result = client_lines.next_line() => match result {
              Ok(Some(line)) => {
                
                println!("{}", line.trim_end());
  
                let _ = tx_for_send.send(format!("{}", line.trim_end()));
              }
              _ => {
                println!("<< {} disconnected", peer);
                break;
              }
            },
 
            result = rx.recv() => match result {
              Ok(msg) => {
            
                if write_half.write_all(msg.as_bytes()).await.is_err() { break; }
                if write_half.write_all(b"\n").await.is_err()    { break; }
                let _ = write_half.flush().await;
              }
              Err(_) => {
            
                break;
              }
            },
          }
        }
      });
    }
  }
  