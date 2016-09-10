use ring::digest;
use ring::hmac;

use std::io::Write;

fn concat_sign(key: &hmac::SigningKey, a: &[u8], b: &[u8]) -> digest::Digest {
  let mut ctx = hmac::SigningContext::with_key(key);
  ctx.update(a);
  ctx.update(b);
  ctx.sign()
}

fn p(out: &mut [u8],
     hashalg: &'static digest::Algorithm,
     secret: &[u8],
     seed: &[u8]) {
  let hmac_key = hmac::SigningKey::new(hashalg, secret);

  /* A(1) */
  let mut current_a = hmac::sign(&hmac_key, seed);

  let mut offs = 0;

  while offs < out.len() {
    /* P_hash[i] = HMAC_hash(secret, A(i) + seed) */
    let p_term = concat_sign(&hmac_key, current_a.as_ref(), seed);
    offs += out[offs..].as_mut().write(p_term.as_ref()).unwrap();

    /* A(i+1) = HMAC_hash(secret, A(i)) */
    current_a = hmac::sign(&hmac_key, current_a.as_ref());
  }
}

fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
  let mut ret = Vec::new();
  ret.extend_from_slice(a);
  ret.extend_from_slice(b);
  ret
}

pub fn prf(out: &mut [u8],
           hashalg: &'static digest::Algorithm,
           secret: &[u8],
           label: &[u8],
           seed: &[u8]) {
  let joined_seed = concat(label, seed);
  p(out, hashalg, secret, &joined_seed);
}

#[cfg(test)]
mod tests {
  use ring::digest::{SHA256, SHA512};

  #[test]
  fn check_sha256() {
    let secret = b"\x9b\xbe\x43\x6b\xa9\x40\xf0\x17\xb1\x76\x52\x84\x9a\x71\xdb\x35";
    let seed = b"\xa0\xba\x9f\x93\x6c\xda\x31\x18\x27\xa6\xf7\x96\xff\xd5\x19\x8c";
    let label = b"test label";
    let expect = b"\xe3\xf2\x29\xba\x72\x7b\xe1\x7b\x8d\x12\x26\x20\x55\x7c\xd4\x53\xc2\xaa\xb2\x1d\x07\xc3\xd4\x95\x32\x9b\x52\xd4\xe6\x1e\xdb\x5a\x6b\x30\x17\x91\xe9\x0d\x35\xc9\xc9\xa4\x6b\x4e\x14\xba\xf9\xaf\x0f\xa0\x22\xf7\x07\x7d\xef\x17\xab\xfd\x37\x97\xc0\x56\x4b\xab\x4f\xbc\x91\x66\x6e\x9d\xef\x9b\x97\xfc\xe3\x4f\x79\x67\x89\xba\xa4\x80\x82\xd1\x22\xee\x42\xc5\xa7\x2e\x5a\x51\x10\xff\xf7\x01\x87\x34\x7b\x66";
    let mut output = [0u8; 100];

    super::prf(&mut output, &SHA256, secret, label, seed);
    assert_eq!(expect.len(), output.len());
    assert_eq!(expect.to_vec(), output.to_vec());
  }

  #[test]
  fn check_sha512() {
    let secret = b"\xb0\x32\x35\x23\xc1\x85\x35\x99\x58\x4d\x88\x56\x8b\xbb\x05\xeb";
    let seed = b"\xd4\x64\x0e\x12\xe4\xbc\xdb\xfb\x43\x7f\x03\xe6\xae\x41\x8e\xe5";
    let label = b"test label";
    let expect = b"\x12\x61\xf5\x88\xc7\x98\xc5\xc2\x01\xff\x03\x6e\x7a\x9c\xb5\xed\xcd\x7f\xe3\xf9\x4c\x66\x9a\x12\x2a\x46\x38\xd7\xd5\x08\xb2\x83\x04\x2d\xf6\x78\x98\x75\xc7\x14\x7e\x90\x6d\x86\x8b\xc7\x5c\x45\xe2\x0e\xb4\x0c\x1c\xf4\xa1\x71\x3b\x27\x37\x1f\x68\x43\x25\x92\xf7\xdc\x8e\xa8\xef\x22\x3e\x12\xea\x85\x07\x84\x13\x11\xbf\x68\x65\x3d\x0c\xfc\x40\x56\xd8\x11\xf0\x25\xc4\x5d\xdf\xa6\xe6\xfe\xc7\x02\xf0\x54\xb4\x09\xd6\xf2\x8d\xd0\xa3\x23\x3e\x49\x8d\xa4\x1a\x3e\x75\xc5\x63\x0e\xed\xbe\x22\xfe\x25\x4e\x33\xa1\xb0\xe9\xf6\xb9\x82\x66\x75\xbe\xc7\xd0\x1a\x84\x56\x58\xdc\x9c\x39\x75\x45\x40\x1d\x40\xb9\xf4\x6c\x7a\x40\x0e\xe1\xb8\xf8\x1c\xa0\xa6\x0d\x1a\x39\x7a\x10\x28\xbf\xf5\xd2\xef\x50\x66\x12\x68\x42\xfb\x8d\xa4\x19\x76\x32\xbd\xb5\x4f\xf6\x63\x3f\x86\xbb\xc8\x36\xe6\x40\xd4\xd8\x98";
    let mut output = [0u8; 196];

    super::prf(&mut output, &SHA512, secret, label, seed);
    assert_eq!(expect.len(), output.len());
    assert_eq!(expect.to_vec(), output.to_vec());
  }
}
