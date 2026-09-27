//! SHA-256 dan hex (SPEC §3).

use sha2::{Digest, Sha256};

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(DIGITS[(b >> 4) as usize] as char);
        s.push(DIGITS[(b & 0xf) as usize] as char);
    }
    s
}

/// Hex 64 karakter menjadi 32 byte.
pub fn unhex32(text: &str) -> Result<[u8; 32], String> {
    let text = text.trim();
    if text.len() != 64 {
        return Err(format!("hex harus 64 karakter, dapat {}", text.len()));
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)
            .map_err(|_| format!("bukan hex: `{text}`"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hex_round_trip() {
        let b: [u8; 32] = core::array::from_fn(|i| i as u8 * 7);
        assert_eq!(unhex32(&hex(&b)).unwrap(), b);
        assert!(unhex32("zz").is_err());
    }
}
