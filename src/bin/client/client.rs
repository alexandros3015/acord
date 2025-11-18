use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
  };
use std::{io::{self, Write}};

use acord::{derive_key_with_salt, encrypt, decrypt};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};


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
  
    let addr = prompt!("server addr> ");
    let nick = prompt!("name> ");
    let key_input = prompt!("key> ");


    println!("connecting to {}", addr);
    let stream = TcpStream::connect(addr).await?;

    let (read_half, mut write_half) = tokio::io::split(stream);
  
    let mut server_lines = BufReader::new(read_half).lines();
    
    let salt_line = server_lines.next_line().await?.ok_or("no salt")?;

    let salt = parse_salt(&salt_line.as_str())?;
    let key = derive_key_with_salt(&key_input.as_bytes(), &salt);
    let key_rx = key.clone();


    tokio::spawn(async move {
      while let Ok(Some(line)) = server_lines.next_line().await {
        let (ciphertext, nonce) = line.split_once(", ").unwrap();
        let ciphertext: Vec<u8> = B64.decode(ciphertext).unwrap();
        let nonce: Vec<u8> = B64.decode(nonce).unwrap();

        let entered_line = decrypt(&key_rx, &nonce, &ciphertext).unwrap();

        println!("\r<< {}", String::from_utf8(entered_line).unwrap());
        print!("you> ");
        let _ = io::stdout().flush();
      }
      eprintln!("\n[-] server closed connection");
    });
  
   
    loop {
      let line = prompt!("> ");
      let formatted_line = format!("{nick}: {line}");
      let (nonce, ciphertext) = encrypt(&key, formatted_line.as_bytes()).unwrap();
      let data = format!("{}, {}", B64.encode(ciphertext), B64.encode(nonce));

      write_half.write_all(data.as_bytes()).await?;
      write_half.write_all(b"\n").await?;
      write_half.flush().await?;

      if line == "!exit" { break; }
    }
  
    Ok(())
  }
  
  fn parse_salt(line: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let line = line.trim();
    let rest = line.strip_prefix("SALT,").ok_or("missing SALT prefix")?;
    let salt = B64.decode(rest.trim())?;

    if salt.len() < 16 {
      return Err("salt too short".into());
    }
    Ok(salt)
  }