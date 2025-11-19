use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use acord::{derive_key_with_salt, encrypt, decrypt};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use std::sync::{Arc, Mutex};

use crossterm::{
    cursor::{MoveToColumn},
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    execute,
};
use std::io::{self, Write};

use zeroize::Zeroize;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = prompt("server addr> ");
    let nick = prompt("name> ");
    let mut pass = prompt("key> ");

    println!("connecting to {}", addr);
    let stream = TcpStream::connect(addr).await?;
    let (read_half, mut write_half) = tokio::io::split(stream);

    let mut server_lines = BufReader::new(read_half).lines();

    let salt_line = server_lines.next_line().await?.ok_or("no salt")?;
    let salt = parse_salt(&salt_line)?;

    let key = derive_key_with_salt(pass.as_bytes(), &salt).expect("Deriving key with salt faliure");
    let key_rx = key.clone();

    pass.zeroize();

    let input_buf = Arc::new(Mutex::new(String::new()));
    let input_buf_rx = Arc::clone(&input_buf);

    let stdout = Arc::new(Mutex::new(io::stdout()));
    let stdout_rx = Arc::clone(&stdout);

    tokio::spawn(async move {
        while let Ok(Some(line)) = server_lines.next_line().await {
            let line = line.trim();
            if line.is_empty() || line.starts_with("SALT,") {
                continue;
            }

            let (ct_b64, nonce_b64) = match line.split_once(',') {
                Some((a, b)) => (a.trim(), b.trim()),
                None => continue,
            };

            let ciphertext = match B64.decode(ct_b64) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let nonce = match B64.decode(nonce_b64) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if nonce.len() != 12 {
                redraw_with_message(&stdout_rx, &input_buf_rx, "<< [bad nonce]").ok();
                continue;
            }

            match decrypt(&key_rx, &nonce, &ciphertext) {
                Ok(plaintext) => {
                    if let Ok(text) = String::from_utf8(plaintext) {
                        redraw_with_message(&stdout_rx, &input_buf_rx, &format!("<< {}", text)).ok();
                    } else {
                        redraw_with_message(&stdout_rx, &input_buf_rx, "<< [non-utf8]").ok();
                    }
                }
                Err(_) => {
                    redraw_with_message(
                        &stdout_rx,
                        &input_buf_rx,
                        "<< Could not decrypt (wrong key/salt?)",
                    )
                    .ok();
                }
            }
        }
        redraw_with_message(&stdout_rx, &input_buf_rx, "[-] server closed connection").ok();
    });

    enable_raw_mode()?;
    {
        let mut out = stdout.lock().unwrap();
        execute!(out, MoveToColumn(1), Clear(ClearType::CurrentLine))?;
        print!("you> ");
        out.flush()?;
    }

    loop {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                match key_event.code {
                    KeyCode::Char(c) => {
                        let mut buf = input_buf.lock().unwrap();
                        buf.push(c);
                        let mut out = stdout.lock().unwrap();
                        print!("{c}");
                        out.flush()?;
                    }
                    KeyCode::Backspace => {
                        let mut buf = input_buf.lock().unwrap();
                        if !buf.is_empty() {
                            buf.pop();
                            let mut out = stdout.lock().unwrap();
                            print!("\u{8} \u{8}");
                            out.flush()?;
                        }
                    }
                    KeyCode::Enter => {
                        let line = {
                            let mut buf = input_buf.lock().unwrap();
                            let s = std::mem::take(&mut *buf);
                            s
                        };

                        {
                            let mut out = stdout.lock().unwrap();
                            println!();
                            out.flush()?;
                        }

                        if line == "!exit" {
                            break;
                        }

                        let msg = format!("{nick}: {line}");
                        let (nonce, ciphertext) = match encrypt(&key, msg.as_bytes()) {
                            Ok(v) => v,
                            Err(e) => {
                                redraw_with_message(
                                    &stdout,
                                    &input_buf,
                                    &format!("<< encrypt failed: {e}"),
                                )
                                .ok();
                                continue;
                            }
                        };

                        let data =
                            format!("{},{}\n", B64.encode(&ciphertext), B64.encode(&nonce));
                        if let Err(e) = write_half.write_all(data.as_bytes()).await {
                            redraw_with_message(
                                &stdout,
                                &input_buf,
                                &format!("<< send failed: {e}"),
                            )
                            .ok();
                            break;
                        }
                        let _ = write_half.flush().await;

                        {
                            let mut out = stdout.lock().unwrap();
                            execute!(out, MoveToColumn(1), Clear(ClearType::CurrentLine))?;
                            print!("you> ");
                            out.flush()?;
                        }
                    }
                    KeyCode::Esc => {
                        let mut buf = input_buf.lock().unwrap();
                        buf.clear();
                        let mut out = stdout.lock().unwrap();
                        execute!(out, MoveToColumn(1), Clear(ClearType::CurrentLine))?;
                        print!("you> ");
                        out.flush()?;
                    }
                    KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Home
                    | KeyCode::End
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Tab
                    | KeyCode::Delete
                    | KeyCode::Insert
                    | KeyCode::F(_) => {
                        // ignore
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    println!();
    Ok(())
}

fn redraw_with_message(
    stdout: &Arc<Mutex<io::Stdout>>,
    input_buf: &Arc<Mutex<String>>,
    msg: &str,
) -> std::io::Result<()> {
    let current = {
        let buf = input_buf.lock().unwrap();
        buf.clone()
    };
    let mut out = stdout.lock().unwrap();
    execute!(out, MoveToColumn(1), Clear(ClearType::CurrentLine))?;
    println!("{msg}");
    execute!(out, MoveToColumn(1), Clear(ClearType::CurrentLine))?;
    print!("you> {current}");
    out.flush()?;
    Ok(())
}

fn parse_salt(line: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let b64 = line.trim().strip_prefix("SALT,").ok_or("missing SALT prefix")?;
    let salt = B64.decode(b64.trim())?;
    if salt.len() < 16 {
        return Err("salt too short".into());
    }
    Ok(salt)
}

fn prompt(msg: &str) -> String {
    print!("{msg}");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim_end().to_string()
}