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
    // 1) bind the listener
    let ip = prompt!("Enter host: ");

    let listener = TcpListener::bind(&ip).await?;
    println!("listening on {}", ip);
  
    // 2) create a broadcast channel for all client messages
    let (tx, _) = broadcast::channel::<String>(100);
  
    loop {
      // 3) accept a new client
      let (socket, peer) = listener.accept().await?;
      println!(">> {} connected", peer);
  
      // 4) clone the Sender for *this* client-handler,
      //    and subscribe to receive everything
      let tx_for_send = tx.clone();
      let mut rx = tx.subscribe();
  
      let (read_half, mut write_half) = tokio::io::split(socket);
      let mut client_lines = BufReader::new(read_half).lines();
  
      // 5) spawn one task per client
      tokio::spawn(async move {
        loop {
          tokio::select! {
            // a) client → server
            result = client_lines.next_line() => match result {
              Ok(Some(line)) => {
                // print server-side
                println!("[{}] {}", peer, line.trim_end());
  
                // broadcast *this* client's line out to ALL subscribers
                let _ = tx_for_send.send(format!("[{}] {}", peer, line.trim_end()));
              }
              _ => {
                println!("<< {} disconnected", peer);
                break;
              }
            },
  
            // b) server broadcast → this client
            result = rx.recv() => match result {
              Ok(msg) => {
                // write the broadcasted msg + newline
                if write_half.write_all(msg.as_bytes()).await.is_err() { break; }
                if write_half.write_all(b"\n").await.is_err()    { break; }
                let _ = write_half.flush().await;
              }
              Err(_) => {
                // channel closed or lagged
                break;
              }
            },
          }
        }
      });
    }
  }
  