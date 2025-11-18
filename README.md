# Acord

Welcome to Acord, an app that coincidentally resembles no other app in any way.

## What is Acord?
Acord is a secure messaging app that uses end-to-end encryption.  
I would recommend using it if you are sending sensitive information, or to test it out. I think there are better tools for secretive people and governments alike, but it's okay.  

## Why did I make it?
I wanted to learn Rust, and I wanted to make a secure messaging app.

## Why is it secure?
Acord uses a combination of AES-GCM and Argon2id to encrypt messages.  
The salt is generated randomly by the server, and the password is hashed with Argon2id.  
The server distributes the salt on client connection, and the client uses it to decrypt the messages.  
For AES-GCM, a fresh nonce is generated for each message sent alongside it.  
The client uses the starting salt, entered key, and nonce to decrypt the message.  
The server is not able to decrypt the message, because it never sees the key.  
If you are worried about someone cracking the password with the given salt, simply reset the server and generate a new salt.  
Attackers need to have your password, so avoid posting it on Twitter.

## What happens if the wrong password is entered?
It won't work. I don't know the specifics of how it won't work if the wrong password is entered, but it's not going to work.

## Features
- Easy to use
- No ads
- No tracking
- No censorship
- No data collection
- No annoying pop-ups
- No annoying notifications

## How to use
1. Enter the IP address of the server in the client
2. Enter your very own secret password (ensure everyone uses the same password)
3. Enter a message
4. Press enter
5. Use the `!exit` command to exit the program
6. Do note, port forwarding is required to host the server with a personal router.
7. If there are any other issues with firewalls, consult an internet guy/search engine.

## How to build
1. Install Rust
2. Clone the repository
3. To build the client, run `cargo build --release --bin client`
4. To build the server, run `cargo build --release --bin server`

## How to run
1. Run the server and enter the IP address (e.g. 0.0.0.0:8000)
2. Run the client and enter the IP address and port of the server adress (same as above)
3. There.

## How to contribute
I'm open to any contributions, whether it's a bug fix, a feature, or a translation.  
If you're interested in contributing, please open an issue or a pull request.

## Why is it called Acord?
It's Alex and Cord. NCORD REFERENCE (check out [NazarShuk](https://github.com/NazarShuk/))

## Why is there no GUI?
Because I haven't added one yet.

## Are there any servers?
Host your own server. I'm poor, and cannot afford to host a server myself.

## Why is there no documentation?
Laziness.

## Why is there no logo?
I don't have a logo.

## Why is there so much information in the README?
So you won't read it all.
