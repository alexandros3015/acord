use std::vec;

use argon2::{Argon2, Algorithm, Params, Version};
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit, Error as AeadError};
use rand_core::{TryRngCore, OsRng};
use argon2::password_hash::errors::InvalidValue;
use argon2::password_hash::Error;

fn argon2id() -> Argon2<'static> {
    let params = Params::new(64 * 1024, 3, 1, Some(32)).unwrap();
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
} 

pub fn create_key(password: &[u8]) -> Result<(Vec<u8>, Vec<u8>), argon2::password_hash::Error> {
    let argon2 = argon2id();

    let mut salt = [0u8; 16];

    match OsRng.try_fill_bytes(&mut salt) {
        Ok(()) => {}
        Err(_) => return Err(Error::Crypto),
    }


    let mut key = [0u8; 32];

    argon2
        .hash_password_into(password, &salt, &mut key)?;

    Ok((key.to_vec(), salt.to_vec()))
}

pub fn derive_key_with_salt(password: &[u8], salt: &[u8]) -> Result<Vec<u8>, argon2::password_hash::Error> {
    if salt.len() < 16 {
        return Err(Error::SaltInvalid(InvalidValue::TooShort));
    }
    else if salt.len() > 32 {
        return Err(Error::SaltInvalid(InvalidValue::TooLong));
    }

    let argon2 = argon2id();
    let mut key = vec![0u8; 32];

    if argon2
        .hash_password_into(password, salt, &mut key)
        .is_err()
        {
            return Err(Error::Password);
        }

    Ok(key)
}

pub fn encrypt(key: &[u8], plaintext: &[u8])
    -> Result<(Vec<u8>, Vec<u8>), AeadError>
{
    
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| AeadError)?;
    
    let mut nonce_bytes = [0u8; 12];
    OsRng.try_fill_bytes(&mut nonce_bytes).map_err(|_| AeadError)?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    
    let ct = cipher
        .encrypt(nonce, plaintext)?;
    

    Ok((nonce_bytes.to_vec(), ct))
}

pub fn decrypt(
    key: &[u8],
    nonce_bytes: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, AeadError>
{
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| AeadError)?;
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)?;

    Ok(pt)
}
