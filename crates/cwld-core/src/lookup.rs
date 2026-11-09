//! Callsign lookup (QRZ.com XML). The core holds no password: it is stored by the operating system.

use crate::{AppCore, CoreError};
use cwld_lookup::{Info, LookupError};
use serde::Serialize;

/// What the settings screen needs to know.
#[derive(Debug, Clone, Serialize)]
pub struct LookupStatus {
    pub enabled: bool,
    pub user: String,
    /// A password is stored in the credential store for this user.
    pub has_password: bool,
}

/// Result of a connection test.
#[derive(Debug, Clone, Serialize)]
pub struct LookupTest {
    pub subscriber: bool,
}

impl From<LookupError> for CoreError {
    fn from(e: LookupError) -> Self {
        CoreError::Lookup(e)
    }
}

impl AppCore {
    fn qrz_user(&self) -> String {
        self.settings.lock().unwrap().lookup.qrz_user.trim().to_string()
    }

    pub fn lookup_status(&self) -> LookupStatus {
        let l = self.settings.lock().unwrap().lookup.clone();
        let user = l.qrz_user.trim().to_string();
        LookupStatus { enabled: l.qrz_enabled, has_password: self.lookup.has_password(&user), user }
    }

    /// Stores the password for the current QRZ user in the credential store.
    pub fn lookup_set_password(&self, password: &str) -> Result<(), CoreError> {
        let user = self.qrz_user();
        if user.is_empty() {
            return Err(CoreError::Invalid("lookup_no_user"));
        }
        if password.is_empty() {
            return Err(CoreError::Invalid("lookup_no_password"));
        }
        Ok(self.lookup.set_password(&user, password)?)
    }

    pub fn lookup_clear_password(&self) -> Result<(), CoreError> {
        Ok(self.lookup.clear_password(&self.qrz_user())?)
    }

    /// Logs in once, to tell the operator whether name and password work.
    pub fn lookup_test(&self) -> Result<LookupTest, CoreError> {
        let r = self.lookup.test(&self.qrz_user())?;
        Ok(LookupTest { subscriber: r.subscriber })
    }

    /// Asks QRZ.com for a callsign. `None` when lookup is off or the call is unknown.
    /// Blocks for at most a few seconds: call it off the interface thread.
    pub fn lookup_call(&self, call: &str) -> Result<Option<Info>, CoreError> {
        let (enabled, user) = {
            let s = self.settings.lock().unwrap();
            (s.lookup.qrz_enabled, s.lookup.qrz_user.trim().to_string())
        };
        if !enabled || user.is_empty() {
            return Ok(None);
        }
        Ok(self.lookup.lookup(&user, call)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_is_off_by_default_and_never_asks_the_network() {
        let core = AppCore::new(None);
        assert!(!core.lookup_status().enabled);
        assert_eq!(core.lookup_call("ON3VZ").unwrap(), None);
    }

    #[test]
    fn a_password_needs_a_user_name_first() {
        let core = AppCore::new(None);
        assert_eq!(core.lookup_set_password("x").unwrap_err().code(), "lookup_no_user");
    }
}
