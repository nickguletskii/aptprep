use debian_packaging::checksum::AnyContentDigest;
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Verification failed: expected {}, got {}",
        hex::encode(.expected),
        hex::encode(.actual)
    )]
    VerificationFailed { expected: Vec<u8>, actual: Vec<u8> },
}

enum ContentDigestHasher {
    Sha1(Sha1),
    Sha256(Sha256),
    Sha384(Sha384),
    Sha512(Sha512),
    Md5(Md5),
}
pub struct ContentDigestVerifier {
    hasher: ContentDigestHasher,
    expected_digest: Vec<u8>,
}

impl ContentDigestVerifier {
    #[inline]
    pub fn new(content_digest: AnyContentDigest) -> Self {
        match content_digest {
            AnyContentDigest::Md5(expected_digest) => Self {
                hasher: ContentDigestHasher::Md5(Md5::new()),
                expected_digest,
            },
            AnyContentDigest::Sha1(expected_digest) => Self {
                hasher: ContentDigestHasher::Sha1(Sha1::new()),
                expected_digest,
            },
            AnyContentDigest::Sha256(expected_digest) => Self {
                hasher: ContentDigestHasher::Sha256(Sha256::new()),
                expected_digest,
            },
            AnyContentDigest::Sha384(expected_digest) => Self {
                hasher: ContentDigestHasher::Sha384(Sha384::new()),
                expected_digest,
            },
            AnyContentDigest::Sha512(expected_digest) => Self {
                hasher: ContentDigestHasher::Sha512(Sha512::new()),
                expected_digest,
            },
        }
    }

    #[inline]
    pub fn update(&mut self, data: impl AsRef<[u8]>) {
        match &mut self.hasher {
            ContentDigestHasher::Sha1(digest) => Digest::update(digest, data.as_ref()),
            ContentDigestHasher::Sha256(digest) => Digest::update(digest, data.as_ref()),
            ContentDigestHasher::Sha384(digest) => Digest::update(digest, data.as_ref()),
            ContentDigestHasher::Sha512(digest) => Digest::update(digest, data.as_ref()),
            ContentDigestHasher::Md5(digest) => Digest::update(digest, data.as_ref()),
        };
    }

    pub fn verify(self) -> Result<(), VerificationError> {
        let actual_digest = match self.hasher {
            ContentDigestHasher::Sha1(digest) => digest.finalize().to_vec(),
            ContentDigestHasher::Sha256(digest) => digest.finalize().to_vec(),
            ContentDigestHasher::Sha384(digest) => digest.finalize().to_vec(),
            ContentDigestHasher::Sha512(digest) => digest.finalize().to_vec(),
            ContentDigestHasher::Md5(digest) => digest.finalize().to_vec(),
        };

        if actual_digest == self.expected_digest {
            Ok(())
        } else {
            Err(VerificationError::VerificationFailed {
                expected: self.expected_digest.clone(),
                actual: actual_digest,
            })
        }
    }
}
