use openssl::pkey::Public;
use openssl::rand::rand_bytes;
use openssl::rsa::Padding;
use openssl::rsa::Rsa;
use openssl::symm::{decrypt, encrypt, Cipher};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct RouteNode {
    pub addr: SocketAddr,
    pub next: SocketAddr,
    pub rsa: Rsa<Public>,
    pub aes: (Vec<u8>, Vec<u8>),
}

impl RouteNode {
    pub fn new(addr: SocketAddr, next: SocketAddr, rsa: Rsa<Public>) -> RouteNode {
        let aes = create_aes_key();

        RouteNode {
            addr,
            next,
            rsa,
            aes,
        }
    }

    pub fn rsa_encrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut buf = vec![0; self.rsa.size() as usize];

        self.rsa
            .public_encrypt(data, &mut buf, Padding::PKCS1)
            .unwrap();

        buf
    }

    pub fn aes_encrypt(&self, data: &[u8]) -> Vec<u8> {
        encrypt(Cipher::aes_256_cbc(), &self.aes.0, Some(&self.aes.1), data).unwrap()
    }

    pub fn aes_decrypt(&self, data: &[u8]) -> Vec<u8> {
        decrypt(Cipher::aes_256_cbc(), &self.aes.0, Some(&self.aes.1), data).unwrap()
    }

    pub fn aes_key_iv(&self) -> Vec<u8> {
        let mut k = self.aes.0.clone();
        k.extend_from_slice(&self.aes.1);
        k
    }
}

fn create_aes_key() -> (Vec<u8>, Vec<u8>) {
    let mut key = [0; 32];
    let mut iv = [0; 16];

    rand_bytes(&mut key).unwrap();
    rand_bytes(&mut iv).unwrap();

    (key.to_vec(), iv.to_vec())
}
