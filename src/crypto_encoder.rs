//
//  Crypto Encoding for DDCL
//

use std::num::NonZeroU32;
use aes::Aes128;
use anyhow::Result;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use ring::pbkdf2;
use base32::Alphabet::RFC4648;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

pub fn encrypt_and_encode(plain: String, password: String, salt: String, iv: String) -> Result<String> {
    let password = &password;
    let mut pbkdf2_hash = [0u8; 16]; // 16 bytes for Aes128
    let n_iter = NonZeroU32::new(65536).unwrap();
    let salt = salt.as_bytes();
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA1,
        n_iter,
        &salt,
        password.as_bytes(),
        &mut pbkdf2_hash,
    );
    let plain = plain.as_bytes();
    let cipher = Aes128Cbc::new_from_slices(&pbkdf2_hash, &iv.as_bytes())?;
    let mut buffer = [0_u8; 1000]; // big buffer
    let pos = plain.len();
    buffer[..pos].copy_from_slice(plain);
    let ciphertext = cipher.encrypt(&mut buffer, pos)?;
    Ok(base32::encode(RFC4648 { padding: true }, ciphertext))
}
