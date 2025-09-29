use debian_packaging::io::ContentDigest;
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Verification failed: expected {expected:?}, got {actual:?}")]
    VerificationFailed { expected: Vec<u8>, actual: Vec<u8> },
}

enum ContentDigestHasher {
    Sha1(Sha1),
    Sha256(Sha256),
    Md5(Md5),
}
pub struct ContentDigestVerifier {
    hasher: ContentDigestHasher,
    expected_digest: Vec<u8>,
}

impl ContentDigestVerifier {
    #[inline]
    pub fn new(content_digest: ContentDigest) -> Self {
        match content_digest {
            ContentDigest::Md5(expected_digest) => Self {
                hasher: ContentDigestHasher::Md5(Md5::new()),
                expected_digest,
            },
            ContentDigest::Sha1(expected_digest) => Self {
                hasher: ContentDigestHasher::Sha1(Sha1::new()),
                expected_digest,
            },
            ContentDigest::Sha256(expected_digest) => Self {
                hasher: ContentDigestHasher::Sha256(Sha256::new()),
                expected_digest,
            },
        }
    }

    #[inline]
    pub fn update(&mut self, data: impl AsRef<[u8]>) {
        match &mut self.hasher {
            ContentDigestHasher::Sha1(digest) => Digest::update(digest, data.as_ref()),
            ContentDigestHasher::Sha256(digest) => Digest::update(digest, data.as_ref()),
            ContentDigestHasher::Md5(digest) => Digest::update(digest, data.as_ref()),
        };
    }

    pub fn verify(self) -> Result<(), VerificationError> {
        let actual_digest = match self.hasher {
            ContentDigestHasher::Sha1(digest) => digest.finalize().to_vec(),
            ContentDigestHasher::Sha256(digest) => digest.finalize().to_vec(),
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
