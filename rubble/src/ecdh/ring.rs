use {
    super::*,
    ::ring::{
        agreement::{agree_ephemeral, EphemeralPrivateKey, UnparsedPublicKey, ECDH_P256},
        rand::SystemRandom,
    },
};

/// An ECDH provider that uses *ring* under the hood.
pub struct RingProvider {
    rng: SystemRandom,
}

impl RingProvider {
    /// Creates a new `RingProvider` that uses the system's RNG for key generation.
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
        }
    }
}

impl EcdhProvider for RingProvider {
    type SecretKey = RingSecretKey;

    fn generate_keypair<R>(&mut self, _: &mut R) -> (Self::SecretKey, PublicKey)
    where
        R: RngCore + CryptoRng,
    {
        let secret = EphemeralPrivateKey::generate(&ECDH_P256, &self.rng).unwrap();
        let public = secret.compute_public_key().unwrap();

        let mut pub_bytes = [0; 64];
        // Strip the first octet (indicates the key type; see RFC 5480)
        pub_bytes.copy_from_slice(&public.as_ref()[1..]);

        let secret = RingSecretKey(secret);
        let public = PublicKey(pub_bytes);

        (secret, public)
    }
}

/// A secret key generated by a [`RingProvider`].
///
/// [`RingProvider`]: struct.RingProvider.html
pub struct RingSecretKey(EphemeralPrivateKey);

impl SecretKey for RingSecretKey {
    fn agree(self, foreign_key: &PublicKey) -> Result<SharedSecret, InvalidPublicKey> {
        // Convert `foreign_key` to ring's format:
        let mut encoded = [0; 65];
        encoded[0] = 0x04; // indicates uncompressed format (see RFC 5480)
        encoded[1..].copy_from_slice(&foreign_key.0);
        let public = UnparsedPublicKey::new(&ECDH_P256, &encoded[..]);

        let mut shared_secret = [0; 32];
        agree_ephemeral(self.0, &public, InvalidPublicKey::new(), |b| {
            shared_secret.copy_from_slice(b);
            Ok(())
        })?;

        Ok(SharedSecret(shared_secret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testsuite() {
        run_tests(RingProvider::new());
    }
}
