use std::vec;

use argon2::Argon2;
use argon2::password_hash::SaltString;
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit, Error as AeadError};
use rand::{rng, Rng};
use argon2::password_hash::rand_core::OsRng;

pub fn create_key(password: &[u8]) -> (Vec<u8>, String) {
    let argon2 = Argon2::default();

    let salt = SaltString::generate(&mut OsRng);

    let mut key = vec![0u8; 32];
    argon2
        .hash_password_into(password, salt.as_salt().as_str().as_bytes(), &mut key)
        .expect("argon2 hashing failed");

    (key, salt.to_string())
}

pub fn derive_key_with_salt(password: &[u8], salt: &[u8]) -> Vec<u8> {
    let argon2 = Argon2::default();
    let mut key = vec![0u8; 32];

    argon2
        .hash_password_into(password, salt, &mut key)
        .expect("argon2 hashing failed");

    key
}

pub fn encrypt(key: &[u8], plaintext: &[u8])
    -> Result<(Vec<u8>, Vec<u8>), AeadError>
{
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AeadError)?;

    
    let mut nonce_bytes = [0u8; 12];
    rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| AeadError)?;

    Ok((nonce_bytes.to_vec(), ct))
}

pub fn decrypt(
    key: &[u8],
    nonce_bytes: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, AeadError>
{
    let cipher = Aes256Gcm::new_from_slice(key)
        .expect("AES256 key must be 32 bytes");

    let pt = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| AeadError)?;

    Ok(pt)
}
