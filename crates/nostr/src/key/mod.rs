// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Keys
//!
//! This module defines the [`Keys`] structure.

use core::fmt;
#[cfg(feature = "nip19")]
use core::str::FromStr;

use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::Rng;
use secp256k1::schnorr::Signature;
use secp256k1::Message;
pub use secp256k1::{KeyPair, PublicKey, SecretKey, XOnlyPublicKey};

use crate::SECP256K1;

#[cfg(feature = "vanity")]
pub mod vanity;

#[cfg(feature = "nip19")]
use crate::nips::nip19::FromBech32;

/// [`Keys`] error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Invalid secret key
    InvalidSecretKey,
    /// Invalid public key
    InvalidPublicKey,
    /// Secret key missing
    SkMissing,
    /// Unsupported char
    InvalidChar(char),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSecretKey => write!(f, "Invalid secret key"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::SkMissing => write!(f, "Secret key missing"),
            Self::InvalidChar(c) => write!(f, "Unsupported char: {c}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// Trait for [`Keys`]
pub trait FromSkStr: Sized {
    /// Error
    type Err;
    /// Init [`Keys`] from `hex` or `bech32` secret key string
    fn from_sk_str(secret_key: &str) -> Result<Self, Self::Err>;
}

/// Trait for [`Keys`]
pub trait FromPkStr: Sized {
    /// Error
    type Err;
    /// Init [`Keys`] from `hex` or `bech32` public key string
    fn from_pk_str(public_key: &str) -> Result<Self, Self::Err>;
}

/// Keys
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    public_key: XOnlyPublicKey,
    key_pair: Option<KeyPair>,
    secret_key: Option<SecretKey>,
}

impl Keys {
    /// Initialize from secret key.
    pub fn new(secret_key: SecretKey) -> Self {
        let key_pair = KeyPair::from_secret_key(SECP256K1, &secret_key);
        let public_key = XOnlyPublicKey::from_keypair(&key_pair).0;

        Self {
            public_key,
            key_pair: Some(key_pair),
            secret_key: Some(secret_key),
        }
    }

    /// Initialize with public key only (no secret key).
    pub fn from_public_key(public_key: XOnlyPublicKey) -> Self {
        Self {
            public_key,
            key_pair: None,
            secret_key: None,
        }
    }

    /// Generate new random [`Keys`]
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let (secret_key, _) = SECP256K1.generate_keypair(&mut rng);
        Self::new(secret_key)
    }

    /// Generate random [`Keys`] with custom [`Rng`]
    pub fn generate_with_rng<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        let (secret_key, _) = SECP256K1.generate_keypair(rng);
        Self::new(secret_key)
    }

    /// Generate random [`Keys`] with custom [`Rng`] and without [`KeyPair`]
    /// Useful for faster [`Keys`] generation (ex. vanity pubkey mining)
    pub fn generate_without_keypair<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        let (secret_key, public_key) = SECP256K1.generate_keypair(rng);
        let (public_key, _) = public_key.x_only_public_key();
        Self {
            public_key,
            key_pair: None,
            secret_key: Some(secret_key),
        }
    }

    /// Get [`XOnlyPublicKey`]
    pub fn public_key(&self) -> XOnlyPublicKey {
        self.public_key
    }

    /// Get secret key
    pub fn secret_key(&self) -> Result<SecretKey, Error> {
        if let Some(secret_key) = self.secret_key {
            Ok(secret_key)
        } else {
            Err(Error::SkMissing)
        }
    }

    /// Get [`PublicKey`]
    pub fn normalized_public_key(&self) -> Result<PublicKey, Error> {
        Ok(self.secret_key()?.public_key(SECP256K1))
    }

    /// Get keypair
    ///
    /// If not exists, will be created
    pub fn key_pair(&self) -> Result<KeyPair, Error> {
        if let Some(key_pair) = self.key_pair {
            Ok(key_pair)
        } else {
            let sk = self.secret_key()?;
            Ok(KeyPair::from_secret_key(SECP256K1, &sk))
        }
    }

    /// Sign schnorr [`Message`]
    pub fn sign_schnorr(&self, message: &Message) -> Result<Signature, Error> {
        let keypair: &KeyPair = &self.key_pair()?;
        Ok(SECP256K1.sign_schnorr(message, keypair))
    }
}

#[cfg(feature = "nip19")]
impl FromSkStr for Keys {
    type Err = Error;

    /// Init [`Keys`] from `hex` or `bech32` secret key
    fn from_sk_str(secret_key: &str) -> Result<Self, Self::Err> {
        match SecretKey::from_str(secret_key) {
            Ok(secret_key) => Ok(Self::new(secret_key)),
            Err(_) => match SecretKey::from_bech32(secret_key) {
                Ok(secret_key) => Ok(Self::new(secret_key)),
                Err(_) => Err(Error::InvalidSecretKey),
            },
        }
    }
}

#[cfg(feature = "nip19")]
impl FromPkStr for Keys {
    type Err = Error;

    /// Init [`Keys`] from `hex` or `bech32` public key
    fn from_pk_str(public_key: &str) -> Result<Self, Self::Err> {
        match XOnlyPublicKey::from_str(public_key) {
            Ok(public_key) => Ok(Self::from_public_key(public_key)),
            Err(_) => match XOnlyPublicKey::from_bech32(public_key) {
                Ok(public_key) => Ok(Self::from_public_key(public_key)),
                Err(_) => Err(Error::InvalidPublicKey),
            },
        }
    }
}
