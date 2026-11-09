//! The QRZ.com password lives in the operating system's credential store (Windows Credential Manager,
//! macOS Keychain, Secret Service on Linux), which encrypts it for the logged-in user. It is never written
//! to the settings file, the log or the interface.

use crate::LookupError;

/// Where the password is kept. A trait so tests do not touch the real store.
pub trait SecretStore: Send + Sync {
    fn get(&self, user: &str) -> Result<Option<String>, LookupError>;
    fn set(&self, user: &str, password: &str) -> Result<(), LookupError>;
    fn clear(&self, user: &str) -> Result<(), LookupError>;
}

const SERVICE: &str = "CWLD QRZ.com";

/// The real store.
pub struct OsStore;

fn map(e: keyring::Error) -> LookupError {
    LookupError::Store(e.to_string())
}

impl SecretStore for OsStore {
    fn get(&self, user: &str) -> Result<Option<String>, LookupError> {
        let entry = keyring::Entry::new(SERVICE, &user.to_ascii_uppercase()).map_err(map)?;
        match entry.get_password() {
            Ok(p) => Ok(Some(p)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(map(e)),
        }
    }
    fn set(&self, user: &str, password: &str) -> Result<(), LookupError> {
        keyring::Entry::new(SERVICE, &user.to_ascii_uppercase()).map_err(map)?.set_password(password).map_err(map)
    }
    fn clear(&self, user: &str) -> Result<(), LookupError> {
        match keyring::Entry::new(SERVICE, &user.to_ascii_uppercase()).map_err(map)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(map(e)),
        }
    }
}
