//! Callsign lookup. First source: the QRZ.com XML service, for operators who have an account.
//!
//! Rules this crate keeps:
//! - The password is only in the OS credential store and in memory for the length of one login call.
//! - Error texts never contain the request address, because that holds the password.
//! - Results are cached, so one callsign is asked at most once a day and the service is not hammered.
//! - The lookup only returns data. Whether it is used is decided by the caller, which never overwrites
//!   what the operator typed.

mod secret;
mod xml;

pub use secret::{OsStore, SecretStore};

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LookupError {
    #[error("no password stored")]
    NoPassword,
    #[error("QRZ.com refused the login: {0}")]
    Login(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("credential store: {0}")]
    Store(String),
    #[error("unexpected reply from QRZ.com")]
    BadReply,
    #[error("QRZ.com: {0}")]
    Service(String),
}

impl LookupError {
    /// Stable code the interface turns into a translated message.
    pub fn code(&self) -> &'static str {
        match self {
            LookupError::NoPassword => "lookup_no_password",
            LookupError::Login(_) => "lookup_login",
            LookupError::Network(_) => "lookup_network",
            LookupError::Store(_) => "lookup_store",
            LookupError::BadReply => "lookup_bad_reply",
            LookupError::Service(_) => "lookup_service",
        }
    }
}

/// What QRZ.com knows about a station, reduced to what the QSO panel can use.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct Info {
    pub call: String,
    /// First name, as it is said on the air.
    pub first_name: String,
    pub last_name: String,
    /// Town (QRZ field `addr2`).
    pub qth: String,
    pub country: String,
    pub grid: String,
    /// The account sees limited data (no XML subscription).
    pub limited: bool,
}

/// Result of a connection test.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestReport {
    /// The account has the XML Logbook Data subscription, so names and addresses are complete.
    pub subscriber: bool,
}

/// One HTTPS GET. A trait so tests can answer without a network.
pub trait Fetcher: Send + Sync {
    fn get(&self, url: &str) -> Result<String, LookupError>;
}

pub struct HttpFetcher;

impl Fetcher for HttpFetcher {
    fn get(&self, url: &str) -> Result<String, LookupError> {
        let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(8))).build().into();
        match agent.get(url).call() {
            Ok(mut r) => r.body_mut().read_to_string().map_err(|_| LookupError::BadReply),
            // Deliberately terse: the full error text can contain the address with the password.
            Err(ureq::Error::StatusCode(c)) => Err(LookupError::Network(format!("http {c}"))),
            Err(ureq::Error::Timeout(_)) => Err(LookupError::Network("timeout".into())),
            Err(_) => Err(LookupError::Network("connection failed".into())),
        }
    }
}

const HIT_TTL: Duration = Duration::from_secs(24 * 3600);
const MISS_TTL: Duration = Duration::from_secs(10 * 60);
const CACHE_MAX: usize = 500;
pub const QRZ_URL: &str = "https://xmldata.qrz.com/xml/current/";

pub struct Lookup {
    fetcher: Box<dyn Fetcher>,
    store: Box<dyn SecretStore>,
    base: String,
    agent: String,
    session: Mutex<Option<String>>,
    cache: Mutex<HashMap<String, (Instant, Option<Info>)>>,
}

impl Lookup {
    pub fn new(version: &str) -> Self {
        Self::with(Box::new(HttpFetcher), Box::new(OsStore), QRZ_URL.into(), version)
    }

    pub fn with(fetcher: Box<dyn Fetcher>, store: Box<dyn SecretStore>, base: String, version: &str) -> Self {
        Lookup { fetcher, store, base, agent: format!("CWLD{version}"), session: Mutex::new(None), cache: Mutex::new(HashMap::new()) }
    }

    pub fn has_password(&self, user: &str) -> bool {
        !user.trim().is_empty() && matches!(self.store.get(user.trim()), Ok(Some(_)))
    }

    /// Stores (encrypted by the OS) a new password and forgets the old session and cached answers.
    pub fn set_password(&self, user: &str, password: &str) -> Result<(), LookupError> {
        self.store.set(user.trim(), password)?;
        self.forget();
        Ok(())
    }

    pub fn clear_password(&self, user: &str) -> Result<(), LookupError> {
        self.store.clear(user.trim())?;
        self.forget();
        Ok(())
    }

    fn forget(&self) {
        *self.session.lock().unwrap() = None;
        self.cache.lock().unwrap().clear();
    }

    fn login(&self, user: &str) -> Result<(String, bool), LookupError> {
        let password = self.store.get(user)?.ok_or(LookupError::NoPassword)?;
        let url = format!("{}?username={};password={};agent={}", self.base, xml::encode(user), xml::encode(&password), xml::encode(&self.agent));
        let reply = self.fetcher.get(&url)?;
        if let Some(e) = xml::tag(&reply, "Error") {
            return Err(LookupError::Login(e));
        }
        let key = xml::tag(&reply, "Key").filter(|k| !k.is_empty()).ok_or(LookupError::BadReply)?;
        let subscriber = !xml::tag(&reply, "SubExp").unwrap_or_default().to_ascii_lowercase().contains("non-subscriber");
        Ok((key, subscriber))
    }

    /// Checks the account without looking anything up.
    pub fn test(&self, user: &str) -> Result<TestReport, LookupError> {
        let (key, subscriber) = self.login(user.trim())?;
        *self.session.lock().unwrap() = Some(key);
        Ok(TestReport { subscriber })
    }

    /// Looks a callsign up. `Ok(None)` means QRZ.com does not know it.
    pub fn lookup(&self, user: &str, call: &str) -> Result<Option<Info>, LookupError> {
        let user = user.trim();
        let call = call.trim().to_ascii_uppercase();
        if call.len() < 3 || user.is_empty() {
            return Ok(None);
        }
        if let Some((at, v)) = self.cache.lock().unwrap().get(&call) {
            if at.elapsed() < if v.is_some() { HIT_TTL } else { MISS_TTL } {
                return Ok(v.clone());
            }
        }
        let mut found = self.ask(user, &call)?;
        if found.is_none() && call.contains('/') {
            // ON3VZ/P and F/ON3VZ: try the part that holds the station's own call.
            if let Some(core) = call.split('/').filter(|p| p.chars().any(|c| c.is_ascii_digit())).max_by_key(|p| p.len()) {
                if core != call {
                    found = self.ask(user, core)?;
                }
            }
        }
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= CACHE_MAX {
            cache.clear();
        }
        cache.insert(call, (Instant::now(), found.clone()));
        Ok(found)
    }

    fn ask(&self, user: &str, call: &str) -> Result<Option<Info>, LookupError> {
        for attempt in 0..2 {
            let existing = self.session.lock().unwrap().clone();
            let (key, subscriber_hint) = match existing {
                Some(k) => (k, None),
                None => {
                    let (k, sub) = self.login(user)?;
                    *self.session.lock().unwrap() = Some(k.clone());
                    (k, Some(sub))
                }
            };
            let url = format!("{}?s={};callsign={}", self.base, xml::encode(&key), xml::encode(call));
            let reply = self.fetcher.get(&url)?;
            if let Some(e) = xml::tag(&reply, "Error") {
                let low = e.to_ascii_lowercase();
                if low.contains("session timeout") || low.contains("invalid session") {
                    *self.session.lock().unwrap() = None;
                    if attempt == 0 {
                        continue;
                    }
                }
                if low.starts_with("not found") {
                    return Ok(None);
                }
                return Err(LookupError::Service(e));
            }
            if !reply.contains("<Callsign>") {
                return Err(LookupError::BadReply);
            }
            let limited = xml::tag(&reply, "SubExp").map(|s| s.to_ascii_lowercase().contains("non-subscriber")).or(subscriber_hint.map(|s| !s)).unwrap_or(false);
            let t = |n: &str| xml::tag(&reply, n).unwrap_or_default();
            return Ok(Some(Info {
                call: t("call"),
                first_name: first_word(&t("fname")),
                last_name: t("name"),
                qth: t("addr2"),
                country: t("country"),
                grid: t("grid").to_ascii_uppercase(),
                limited,
            }));
        }
        Err(LookupError::BadReply)
    }
}

/// "Jan Piet" becomes "Jan": the name that is sent on the air.
fn first_word(s: &str) -> String {
    s.split_whitespace().next().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex as M};

    #[derive(Default)]
    struct Mem(M<HashMap<String, String>>);
    impl SecretStore for Arc<Mem> {
        fn get(&self, u: &str) -> Result<Option<String>, LookupError> {
            Ok(self.0.lock().unwrap().get(u).cloned())
        }
        fn set(&self, u: &str, p: &str) -> Result<(), LookupError> {
            self.0.lock().unwrap().insert(u.into(), p.into());
            Ok(())
        }
        fn clear(&self, u: &str) -> Result<(), LookupError> {
            self.0.lock().unwrap().remove(u);
            Ok(())
        }
    }

    /// Answers like QRZ.com and records the addresses it was asked.
    struct Fake {
        log: Arc<M<Vec<String>>>,
        timeouts: Arc<M<u32>>,
    }
    impl Fetcher for Fake {
        fn get(&self, url: &str) -> Result<String, LookupError> {
            self.log.lock().unwrap().push(url.to_string());
            if url.contains("username=") {
                return Ok(if url.contains("password=goed") {
                    "<QRZDatabase><Session><Key>K1</Key><SubExp>Mon Dec 1 2030</SubExp></Session></QRZDatabase>".into()
                } else {
                    "<QRZDatabase><Session><Error>Username/password incorrect</Error></Session></QRZDatabase>".into()
                });
            }
            let mut t = self.timeouts.lock().unwrap();
            if *t > 0 {
                *t -= 1;
                return Ok("<QRZDatabase><Session><Error>Session Timeout</Error></Session></QRZDatabase>".into());
            }
            if url.contains("callsign=ON3VZ") {
                Ok("<QRZDatabase><Callsign><call>ON3VZ</call><fname>Kristof Jan</fname><name>Cornelis</name><addr2>Hoboken</addr2><country>Belgium</country><grid>jo21</grid></Callsign></QRZDatabase>".into())
            } else {
                Ok("<QRZDatabase><Session><Error>Not found: X</Error></Session></QRZDatabase>".into())
            }
        }
    }

    fn make(pw: &str) -> (Lookup, Arc<M<Vec<String>>>, Arc<M<u32>>) {
        let log = Arc::new(M::new(vec![]));
        let timeouts = Arc::new(M::new(0));
        let store = Arc::new(Mem::default());
        store.set("ON3VZ", pw).unwrap();
        let l = Lookup::with(Box::new(Fake { log: log.clone(), timeouts: timeouts.clone() }), Box::new(store), "http://x/".into(), "0.13");
        (l, log, timeouts)
    }

    #[test]
    fn finds_a_station_and_uses_the_first_name_only() {
        let (l, _, _) = make("goed");
        let i = l.lookup("ON3VZ", "on3vz").unwrap().unwrap();
        assert_eq!((i.first_name.as_str(), i.qth.as_str(), i.grid.as_str()), ("Kristof", "Hoboken", "JO21"));
        assert!(!i.limited);
    }

    #[test]
    fn unknown_call_is_none_and_cached_without_new_requests() {
        let (l, log, _) = make("goed");
        assert_eq!(l.lookup("ON3VZ", "ZZ9ZZ").unwrap(), None);
        let n = log.lock().unwrap().len();
        assert_eq!(l.lookup("ON3VZ", "ZZ9ZZ").unwrap(), None);
        assert_eq!(log.lock().unwrap().len(), n);
    }

    #[test]
    fn wrong_password_is_a_login_error_and_never_in_the_message() {
        let (l, _, _) = make("fout&geheim");
        let e = l.lookup("ON3VZ", "ON3VZ").unwrap_err();
        assert_eq!(e.code(), "lookup_login");
        assert!(!e.to_string().contains("geheim"));
    }

    #[test]
    fn an_expired_session_is_renewed_once() {
        let (l, log, timeouts) = make("goed");
        l.test("ON3VZ").unwrap();
        *timeouts.lock().unwrap() = 1;
        assert!(l.lookup("ON3VZ", "ON3VZ").unwrap().is_some());
        assert_eq!(log.lock().unwrap().iter().filter(|u| u.contains("username=")).count(), 2);
    }

    #[test]
    fn password_characters_are_encoded_and_missing_password_is_reported() {
        let (l, log, _) = make("goed");
        l.set_password("ON3VZ", "a;b&c").unwrap();
        let _ = l.test("ON3VZ");
        assert!(log.lock().unwrap().last().unwrap().contains("password=a%3Bb%26c"));
        l.clear_password("ON3VZ").unwrap();
        assert!(!l.has_password("ON3VZ"));
        assert_eq!(l.lookup("ON3VZ", "ON3VZ").unwrap_err(), LookupError::NoPassword);
    }

    #[test]
    fn portable_suffix_falls_back_to_the_base_call() {
        let (l, _, _) = make("goed");
        assert_eq!(l.lookup("ON3VZ", "ON3VZ/P").unwrap().unwrap().call, "ON3VZ");
    }
}
