//! The logbook of CWLD.
//!
//! One SQLite file for the whole app. Normal QSOs, contest QSOs and test QSOs (the separate
//! test log used to try out the serial number) live in the same table; the `test` flag keeps
//! them apart. A deleted QSO is only marked, so nothing is lost by accident.
//! ADIF is an export format, not the storage.

mod adif;
mod band;
mod check;

pub use adif::{adif_record, adif_text, wwff_file_name, AdifOptions};
pub use adif::date_time as adif_date_time;
pub use band::band_of;
pub use check::{check_export, valid_grid, valid_wwff_ref, CheckIssue, Severity};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("logbook: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("logbook file: {0}")]
    Io(#[from] std::io::Error),
}

impl LogError {
    pub fn code(&self) -> &'static str {
        match self {
            LogError::Db(_) => "log_db",
            LogError::Io(_) => "log_io",
        }
    }
}

/// One contact. `id` is 0 for a QSO that has not been stored yet.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Qso {
    pub id: i64,
    /// Seconds since 1970-01-01 00:00:00 UTC.
    pub ts: i64,
    pub call: String,
    pub band: String,
    /// The frequency this station transmitted on (ADIF `FREQ`).
    pub freq_hz: u64,
    /// The frequency this station listened on when it was another one (split, ADIF `FREQ_RX`).
    /// 0: the same as `freq_hz`.
    pub freq_rx_hz: u64,
    pub mode: String,
    pub rst_sent: String,
    pub rst_rcvd: String,
    /// Serial number sent (0: none).
    pub nr_sent: u32,
    /// Serial number received, as typed (may be empty).
    pub nr_rcvd: String,
    /// Extra field sent (for example the section) and received.
    pub extra_sent: String,
    pub extra_rcvd: String,
    pub name: String,
    pub qth: String,
    pub comment: String,
    /// Id of the contest profile, empty for a normal QSO.
    pub contest: String,
    /// Which run of that contest (a new contest raises it, dupes only count inside one).
    pub session: u32,
    pub my_call: String,
    pub test: bool,
    pub deleted: bool,
    /// The station the QSO was made from (copied from the profile at log time).
    pub station_call: String,
    pub operator: String,
    pub my_grid: String,
    pub tx_pwr: String,
    pub my_rig: String,
    pub my_antenna: String,
    /// The activation this QSO belongs to (0: none) and the own references of it.
    pub activation_id: i64,
    pub my_wwff: String,
    pub my_pota: String,
    pub my_sota: String,
    /// Park to park: the program (`WWFF`, `POTA`, `SOTA`) and reference of the other station.
    pub their_prog: String,
    pub their_ref: String,
    /// Where this QSO has been sent already, as words: `adif`.
    pub exported_to: String,
}

/// A stay at a location for fauna and flora (WWFF, ONFF), POTA or SOTA. At most one is open.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Activation {
    pub id: i64,
    pub profile: String,
    pub grid: String,
    pub power: String,
    pub wwff: String,
    pub pota: String,
    pub sota: String,
    pub note: String,
    pub started_ts: i64,
    /// 0 while the activation is open.
    pub ended_ts: i64,
}

/// What to select. Empty or zero means: no restriction on that point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LogFilter {
    pub from_ts: Option<i64>,
    pub to_ts: Option<i64>,
    pub band: String,
    pub contest: String,
    pub session: Option<u32>,
    /// None: both. Some(false): the real log only. Some(true): the test log only.
    pub test: Option<bool>,
    pub include_deleted: bool,
    /// Only the QSOs of this activation.
    pub activation: Option<i64>,
    /// Only QSOs that have not been exported yet.
    pub not_exported: bool,
    /// Part of the callsign, name, QTH, comment or the other station's reference (any case).
    pub search: String,
    /// Newest first, at most this many (0: all).
    pub limit: u32,
}

fn act_row(r: &rusqlite::Row) -> rusqlite::Result<Activation> {
    Ok(Activation {
        id: r.get(0)?,
        profile: r.get(1)?,
        grid: r.get(2)?,
        power: r.get(3)?,
        wwff: r.get(4)?,
        pota: r.get(5)?,
        sota: r.get(6)?,
        note: r.get(7)?,
        started_ts: r.get(8)?,
        ended_ts: r.get(9)?,
    })
}

pub struct Logbook {
    conn: Connection,
}

const COLS: &str = "id, ts, call, band, freq_hz, mode, rst_sent, rst_rcvd, nr_sent, nr_rcvd, extra_sent, extra_rcvd, name, qth, comment, contest, session, my_call, test, deleted, station_call, operator, my_grid, tx_pwr, my_rig, my_antenna, activation_id, my_wwff, my_pota, my_sota, their_prog, their_ref, exported_to, freq_rx_hz";

fn row(r: &rusqlite::Row) -> rusqlite::Result<Qso> {
    Ok(Qso {
        id: r.get(0)?,
        ts: r.get(1)?,
        call: r.get(2)?,
        band: r.get(3)?,
        freq_hz: r.get::<_, i64>(4)? as u64,
        mode: r.get(5)?,
        rst_sent: r.get(6)?,
        rst_rcvd: r.get(7)?,
        nr_sent: r.get::<_, i64>(8)? as u32,
        nr_rcvd: r.get(9)?,
        extra_sent: r.get(10)?,
        extra_rcvd: r.get(11)?,
        name: r.get(12)?,
        qth: r.get(13)?,
        comment: r.get(14)?,
        contest: r.get(15)?,
        session: r.get::<_, i64>(16)? as u32,
        my_call: r.get(17)?,
        test: r.get::<_, i64>(18)? != 0,
        deleted: r.get::<_, i64>(19)? != 0,
        station_call: r.get(20)?,
        operator: r.get(21)?,
        my_grid: r.get(22)?,
        tx_pwr: r.get(23)?,
        my_rig: r.get(24)?,
        my_antenna: r.get(25)?,
        activation_id: r.get(26)?,
        my_wwff: r.get(27)?,
        my_pota: r.get(28)?,
        my_sota: r.get(29)?,
        their_prog: r.get(30)?,
        their_ref: r.get(31)?,
        exported_to: r.get(32)?,
        freq_rx_hz: r.get::<_, i64>(33)? as u64,
    })
}

impl Logbook {
    /// Opens (or creates) the logbook file. A copy of the previous state is made first.
    pub fn open(path: &Path) -> Result<Self, LogError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        if path.exists() {
            // One backup per start, overwritten by the next start.
            let _ = std::fs::copy(path, path.with_extension("db.bak"));
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_memory() -> Result<Self, LogError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, LogError> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             CREATE TABLE IF NOT EXISTS qso (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               ts INTEGER NOT NULL,
               call TEXT NOT NULL,
               band TEXT NOT NULL DEFAULT '',
               freq_hz INTEGER NOT NULL DEFAULT 0,
               mode TEXT NOT NULL DEFAULT 'CW',
               rst_sent TEXT NOT NULL DEFAULT '',
               rst_rcvd TEXT NOT NULL DEFAULT '',
               nr_sent INTEGER NOT NULL DEFAULT 0,
               nr_rcvd TEXT NOT NULL DEFAULT '',
               extra_sent TEXT NOT NULL DEFAULT '',
               extra_rcvd TEXT NOT NULL DEFAULT '',
               name TEXT NOT NULL DEFAULT '',
               qth TEXT NOT NULL DEFAULT '',
               comment TEXT NOT NULL DEFAULT '',
               contest TEXT NOT NULL DEFAULT '',
               session INTEGER NOT NULL DEFAULT 0,
               my_call TEXT NOT NULL DEFAULT '',
               test INTEGER NOT NULL DEFAULT 0,
               deleted INTEGER NOT NULL DEFAULT 0
             );
             CREATE INDEX IF NOT EXISTS qso_call ON qso(call);
             CREATE INDEX IF NOT EXISTS qso_ts ON qso(ts);
             CREATE TABLE IF NOT EXISTS activation (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               profile TEXT NOT NULL DEFAULT '',
               grid TEXT NOT NULL DEFAULT '',
               power TEXT NOT NULL DEFAULT '',
               wwff TEXT NOT NULL DEFAULT '',
               pota TEXT NOT NULL DEFAULT '',
               sota TEXT NOT NULL DEFAULT '',
               note TEXT NOT NULL DEFAULT '',
               started_ts INTEGER NOT NULL,
               ended_ts INTEGER NOT NULL DEFAULT 0
             );",
        )?;
        let lb = Self { conn };
        lb.migrate()?;
        Ok(lb)
    }

    /// Adds the columns that older files do not have yet.
    fn migrate(&self) -> Result<(), LogError> {
        let mut have: Vec<String> = Vec::new();
        {
            let mut st = self.conn.prepare("PRAGMA table_info(qso)")?;
            let rows = st.query_map([], |r| r.get::<_, String>(1))?;
            for r in rows {
                have.push(r?);
            }
        }
        for (name, kind) in [
            ("station_call", "TEXT NOT NULL DEFAULT ''"),
            ("operator", "TEXT NOT NULL DEFAULT ''"),
            ("my_grid", "TEXT NOT NULL DEFAULT ''"),
            ("tx_pwr", "TEXT NOT NULL DEFAULT ''"),
            ("my_rig", "TEXT NOT NULL DEFAULT ''"),
            ("my_antenna", "TEXT NOT NULL DEFAULT ''"),
            ("activation_id", "INTEGER NOT NULL DEFAULT 0"),
            ("my_wwff", "TEXT NOT NULL DEFAULT ''"),
            ("my_pota", "TEXT NOT NULL DEFAULT ''"),
            ("my_sota", "TEXT NOT NULL DEFAULT ''"),
            ("their_prog", "TEXT NOT NULL DEFAULT ''"),
            ("their_ref", "TEXT NOT NULL DEFAULT ''"),
            ("exported_to", "TEXT NOT NULL DEFAULT ''"),
            ("freq_rx_hz", "INTEGER NOT NULL DEFAULT 0"),
        ] {
            if !have.iter().any(|h| h == name) {
                self.conn.execute(&format!("ALTER TABLE qso ADD COLUMN {name} {kind}"), [])?;
            }
        }
        Ok(())
    }

    /// Stores a new QSO. The band is filled in from the frequency when it is empty.
    pub fn add(&self, q: &Qso) -> Result<Qso, LogError> {
        let mut q = q.clone();
        q.call = q.call.trim().to_uppercase();
        if q.band.is_empty() {
            q.band = band_of(q.freq_hz).to_string();
        }
        if q.mode.is_empty() {
            q.mode = "CW".into();
        }
        if q.freq_rx_hz == q.freq_hz {
            q.freq_rx_hz = 0;
        }
        self.conn.execute(
            "INSERT INTO qso (ts, call, band, freq_hz, mode, rst_sent, rst_rcvd, nr_sent, nr_rcvd, extra_sent, extra_rcvd, name, qth, comment, contest, session, my_call, test, deleted,
                              station_call, operator, my_grid, tx_pwr, my_rig, my_antenna, activation_id, my_wwff, my_pota, my_sota, their_prog, their_ref, exported_to, freq_rx_hz)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,0,
                     ?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32)",
            params![q.ts, q.call, q.band, q.freq_hz as i64, q.mode, q.rst_sent, q.rst_rcvd, q.nr_sent as i64, q.nr_rcvd, q.extra_sent, q.extra_rcvd, q.name, q.qth, q.comment, q.contest, q.session as i64, q.my_call, q.test as i64,
                    q.station_call, q.operator, q.my_grid, q.tx_pwr, q.my_rig, q.my_antenna, q.activation_id, q.my_wwff, q.my_pota, q.my_sota, q.their_prog, q.their_ref, q.exported_to, q.freq_rx_hz as i64],
        )?;
        q.id = self.conn.last_insert_rowid();
        q.deleted = false;
        Ok(q)
    }

    /// Replaces the fields of a stored QSO.
    pub fn update(&self, q: &Qso) -> Result<(), LogError> {
        let call = q.call.trim().to_uppercase();
        // The band follows the frequency whenever the frequency lies in an amateur band.
        let band = match band_of(q.freq_hz) {
            "" => q.band.clone(),
            b => b.to_string(),
        };
        let mode = if q.mode.trim().is_empty() { "CW".to_string() } else { q.mode.trim().to_uppercase() };
        self.conn.execute(
            "UPDATE qso SET ts=?2, call=?3, band=?4, freq_hz=?5, mode=?6, rst_sent=?7, rst_rcvd=?8, nr_sent=?9, nr_rcvd=?10, extra_sent=?11, extra_rcvd=?12, name=?13, qth=?14, comment=?15, their_prog=?16, their_ref=?17,
                station_call=?18, operator=?19, my_grid=?20, tx_pwr=?21, my_rig=?22, my_antenna=?23, my_wwff=?24, my_pota=?25, my_sota=?26, freq_rx_hz=?27, exported_to='' WHERE id=?1",
            params![q.id, q.ts, call, band, q.freq_hz as i64, mode, q.rst_sent, q.rst_rcvd, q.nr_sent as i64, q.nr_rcvd, q.extra_sent, q.extra_rcvd, q.name, q.qth, q.comment, q.their_prog, q.their_ref,
                    q.station_call, q.operator, q.my_grid, q.tx_pwr, q.my_rig, q.my_antenna, q.my_wwff, q.my_pota, q.my_sota, if q.freq_rx_hz == q.freq_hz { 0 } else { q.freq_rx_hz as i64 }],
        )?;
        Ok(())
    }

    /// Marks a QSO as deleted (it stays in the file).
    pub fn delete(&self, id: i64) -> Result<(), LogError> {
        self.conn.execute("UPDATE qso SET deleted=1 WHERE id=?1", params![id])?;
        Ok(())
    }

    /// Removes the whole test log for good.
    pub fn clear_test(&self) -> Result<u64, LogError> {
        Ok(self.conn.execute("DELETE FROM qso WHERE test=1", [])? as u64)
    }

    pub fn list(&self, f: &LogFilter) -> Result<Vec<Qso>, LogError> {
        let mut sql = format!("SELECT {COLS} FROM qso WHERE 1=1");
        let mut args: Vec<rusqlite::types::Value> = Vec::new();
        let mut push = |clause: &str, v: rusqlite::types::Value, sql: &mut String| {
            args.push(v);
            sql.push_str(&format!(" AND {clause}?{}", args.len()));
        };
        if !f.include_deleted {
            sql.push_str(" AND deleted=0");
        }
        if let Some(v) = f.from_ts {
            push("ts>=", v.into(), &mut sql);
        }
        if let Some(v) = f.to_ts {
            push("ts<=", v.into(), &mut sql);
        }
        if !f.band.is_empty() {
            push("band=", f.band.clone().into(), &mut sql);
        }
        if !f.contest.is_empty() {
            push("contest=", f.contest.clone().into(), &mut sql);
        }
        if let Some(v) = f.session {
            push("session=", (v as i64).into(), &mut sql);
        }
        if let Some(v) = f.test {
            push("test=", (v as i64).into(), &mut sql);
        }
        if let Some(v) = f.activation {
            push("activation_id=", v.into(), &mut sql);
        }
        if f.not_exported {
            sql.push_str(" AND exported_to NOT LIKE '%adif%'");
        }
        let needle = f.search.trim().to_uppercase();
        if !needle.is_empty() {
            // LIKE with the wildcards of the text itself escaped, so % and _ are plain characters.
            let esc = needle.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            args.push(format!("%{esc}%").into());
            let n = args.len();
            sql.push_str(&format!(" AND (UPPER(call) LIKE ?{n} ESCAPE '\\' OR UPPER(name) LIKE ?{n} ESCAPE '\\' OR UPPER(qth) LIKE ?{n} ESCAPE '\\' OR UPPER(comment) LIKE ?{n} ESCAPE '\\' OR UPPER(their_ref) LIKE ?{n} ESCAPE '\\')"));
        }
        sql.push_str(" ORDER BY ts DESC, id DESC");
        if f.limit > 0 {
            sql.push_str(&format!(" LIMIT {}", f.limit));
        }
        let mut st = self.conn.prepare(&sql)?;
        let rows = st.query_map(rusqlite::params_from_iter(args.iter()), row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// The earlier QSO with the same call on the same band and mode in the same run of the
    /// same contest (test and real never mix). A dupe is only a warning.
    pub fn dupe(&self, call: &str, band: &str, mode: &str, contest: &str, session: u32, test: bool) -> Result<Option<Qso>, LogError> {
        let sql = format!(
            "SELECT {COLS} FROM qso WHERE deleted=0 AND call=?1 AND band=?2 AND mode=?3 AND contest=?4 AND session=?5 AND test=?6 ORDER BY ts DESC LIMIT 1"
        );
        let mut st = self.conn.prepare(&sql)?;
        let mut it = st.query_map(params![call.trim().to_uppercase(), band, mode, contest, session as i64, test as i64], row)?;
        Ok(it.next().transpose()?)
    }

    /// The same call on any other band of this contest run, for the grey "worked on 40 m" hint.
    pub fn worked_bands(&self, call: &str, contest: &str, session: u32, test: bool) -> Result<Vec<String>, LogError> {
        let mut st = self.conn.prepare("SELECT DISTINCT band FROM qso WHERE deleted=0 AND call=?1 AND contest=?2 AND session=?3 AND test=?4 ORDER BY band")?;
        let rows = st.query_map(params![call.trim().to_uppercase(), contest, session as i64, test as i64], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Every (call, band) pair of the real log, for coloring spots. With `run` only the QSOs of
    /// that contest and run count (a dupe only counts inside one run); without it the whole log.
    pub fn worked_pairs(&self, run: Option<(&str, u32)>) -> Result<std::collections::HashSet<(String, String)>, LogError> {
        let (sql, args): (&str, Vec<rusqlite::types::Value>) = match run {
            Some((contest, session)) => ("SELECT DISTINCT call, band FROM qso WHERE deleted=0 AND test=0 AND contest=?1 AND session=?2", vec![contest.to_string().into(), (session as i64).into()]),
            None => ("SELECT DISTINCT call, band FROM qso WHERE deleted=0 AND test=0", vec![]),
        };
        let mut st = self.conn.prepare(sql)?;
        let rows = st.query_map(rusqlite::params_from_iter(args.iter()), |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// The next serial number of the test log: one more than the highest used so far.
    pub fn next_test_serial(&self) -> Result<u32, LogError> {
        let n: i64 = self.conn.query_row("SELECT COALESCE(MAX(nr_sent),0) FROM qso WHERE test=1 AND deleted=0", [], |r| r.get(0))?;
        Ok(n as u32 + 1)
    }

    /// Remembers that these QSOs went into an ADIF file. They are left out of "since last export".
    pub fn mark_exported(&self, ids: &[i64]) -> Result<(), LogError> {
        for id in ids {
            self.conn.execute("UPDATE qso SET exported_to = CASE WHEN exported_to='' THEN 'adif' ELSE exported_to || ' adif' END WHERE id=?1 AND exported_to NOT LIKE '%adif%'", params![id])?;
        }
        Ok(())
    }

    /// Opens an activation. An activation that is still open is ended first.
    pub fn start_activation(&self, a: &Activation, now: i64) -> Result<Activation, LogError> {
        self.end_activation(now)?;
        self.conn.execute(
            "INSERT INTO activation (profile, grid, power, wwff, pota, sota, note, started_ts, ended_ts) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,0)",
            params![a.profile, a.grid.trim().to_uppercase(), a.power, a.wwff.trim().to_uppercase(), a.pota.trim().to_uppercase(), a.sota.trim().to_uppercase(), a.note, now],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(self.activation(id)?.expect("just inserted"))
    }

    pub fn end_activation(&self, now: i64) -> Result<(), LogError> {
        self.conn.execute("UPDATE activation SET ended_ts=?1 WHERE ended_ts=0", params![now])?;
        Ok(())
    }

    pub fn activation(&self, id: i64) -> Result<Option<Activation>, LogError> {
        let mut st = self.conn.prepare("SELECT id, profile, grid, power, wwff, pota, sota, note, started_ts, ended_ts FROM activation WHERE id=?1")?;
        let mut it = st.query_map(params![id], act_row)?;
        Ok(it.next().transpose()?)
    }

    pub fn current_activation(&self) -> Result<Option<Activation>, LogError> {
        let mut st = self.conn.prepare("SELECT id, profile, grid, power, wwff, pota, sota, note, started_ts, ended_ts FROM activation WHERE ended_ts=0 ORDER BY id DESC LIMIT 1")?;
        let mut it = st.query_map([], act_row)?;
        Ok(it.next().transpose()?)
    }

    /// Latest activations first.
    pub fn activations(&self, limit: u32) -> Result<Vec<Activation>, LogError> {
        let mut st = self.conn.prepare("SELECT id, profile, grid, power, wwff, pota, sota, note, started_ts, ended_ts FROM activation ORDER BY id DESC LIMIT ?1")?;
        let rows = st.query_map(params![limit as i64], act_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn count(&self, f: &LogFilter) -> Result<usize, LogError> {
        Ok(self.list(&LogFilter { limit: 0, ..f.clone() })?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(call: &str, ts: i64, band: &str) -> Qso {
        Qso { ts, call: call.into(), band: band.into(), mode: "CW".into(), contest: "uba-dx-cw".into(), session: 1, my_call: "ON3VZ".into(), ..Default::default() }
    }

    #[test]
    fn add_list_update_delete() {
        let lb = Logbook::open_memory().unwrap();
        let a = lb.add(&q("pa3abc", 1000, "40m")).unwrap();
        assert_eq!(a.call, "PA3ABC");
        assert!(a.id > 0);
        let mut b = lb.add(&q("DL1XYZ", 2000, "40m")).unwrap();
        let all = lb.list(&LogFilter::default()).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].call, "DL1XYZ", "newest first");
        b.rst_rcvd = "579".into();
        lb.update(&b).unwrap();
        assert_eq!(lb.list(&LogFilter { limit: 1, ..Default::default() }).unwrap()[0].rst_rcvd, "579");
        lb.delete(a.id).unwrap();
        assert_eq!(lb.list(&LogFilter::default()).unwrap().len(), 1);
        assert_eq!(lb.list(&LogFilter { include_deleted: true, ..Default::default() }).unwrap().len(), 2, "a deleted QSO stays in the file");
    }

    #[test]
    fn update_changes_every_field_and_the_band_follows_the_frequency() {
        let lb = Logbook::open_memory().unwrap();
        let mut a = lb.add(&Qso { ts: 1, call: "PA3ABC".into(), freq_hz: 7_030_000, ..Default::default() }).unwrap();
        a.freq_hz = 14_025_000;
        a.ts = 5000;
        a.mode = "ssb".into();
        a.comment = "fixed".into();
        a.station_call = "ON3VZ/P".into();
        a.my_grid = "JO21".into();
        a.tx_pwr = "5".into();
        a.their_ref = "ONFF-0001".into();
        a.exported_to = "adif".into();
        lb.update(&a).unwrap();
        let r = &lb.list(&LogFilter::default()).unwrap()[0];
        assert_eq!((r.band.as_str(), r.freq_hz, r.ts, r.mode.as_str()), ("20m", 14_025_000, 5000, "SSB"));
        assert_eq!((r.comment.as_str(), r.station_call.as_str(), r.my_grid.as_str(), r.tx_pwr.as_str(), r.their_ref.as_str()), ("fixed", "ON3VZ/P", "JO21", "5", "ONFF-0001"));
        assert_eq!(r.exported_to, "", "an edited QSO is exported again");
        let mut a = lb.list(&LogFilter::default()).unwrap().remove(0);
        a.freq_hz = 9_000_000;
        lb.update(&a).unwrap();
        assert_eq!(lb.list(&LogFilter::default()).unwrap()[0].band, "20m", "outside the bands the band stays as it was");
    }

    #[test]
    fn band_is_filled_from_the_frequency() {
        let lb = Logbook::open_memory().unwrap();
        let a = lb.add(&Qso { ts: 1, call: "PA3ABC".into(), freq_hz: 7_030_000, ..Default::default() }).unwrap();
        assert_eq!(a.band, "40m");
        assert_eq!(a.mode, "CW");
    }

    #[test]
    fn dupe_needs_same_band_mode_contest_run_and_log() {
        let lb = Logbook::open_memory().unwrap();
        lb.add(&q("PA3ABC", 1000, "40m")).unwrap();
        assert!(lb.dupe("pa3abc", "40m", "CW", "uba-dx-cw", 1, false).unwrap().is_some());
        assert!(lb.dupe("PA3ABC", "20m", "CW", "uba-dx-cw", 1, false).unwrap().is_none(), "other band");
        assert!(lb.dupe("PA3ABC", "40m", "CW", "uba-dx-cw", 2, false).unwrap().is_none(), "new contest run");
        assert!(lb.dupe("PA3ABC", "40m", "CW", "uba-dx-cw", 1, true).unwrap().is_none(), "test log is separate");
        assert_eq!(lb.worked_bands("PA3ABC", "uba-dx-cw", 1, false).unwrap(), vec!["40m".to_string()]);
    }

    #[test]
    fn worked_pairs_follow_the_run_and_skip_test_and_deleted() {
        let lb = Logbook::open_memory().unwrap();
        lb.add(&q("PA3ABC", 1000, "40m")).unwrap();
        let gone = lb.add(&q("DL1XYZ", 1001, "20m")).unwrap();
        lb.delete(gone.id).unwrap();
        lb.add(&Qso { test: true, ..q("TEST1", 1002, "40m") }).unwrap();
        lb.add(&Qso { contest: String::new(), session: 0, ..q("ON4AA", 1003, "80m") }).unwrap();
        let all = lb.worked_pairs(None).unwrap();
        assert!(all.contains(&("PA3ABC".to_string(), "40m".to_string())));
        assert!(all.contains(&("ON4AA".to_string(), "80m".to_string())));
        assert_eq!(all.len(), 2, "no test QSO and no deleted one: {all:?}");
        let run = lb.worked_pairs(Some(("uba-dx-cw", 1))).unwrap();
        assert_eq!(run.len(), 1);
        assert!(lb.worked_pairs(Some(("uba-dx-cw", 2))).unwrap().is_empty(), "a new run starts empty");
    }

    #[test]
    fn test_log_is_separate_and_can_be_cleared() {
        let lb = Logbook::open_memory().unwrap();
        lb.add(&q("PA3ABC", 1000, "40m")).unwrap();
        for n in 1..=3 {
            lb.add(&Qso { test: true, nr_sent: n, ..q("TEST1", 2000 + n as i64, "40m") }).unwrap();
        }
        assert_eq!(lb.next_test_serial().unwrap(), 4);
        assert_eq!(lb.list(&LogFilter { test: Some(false), ..Default::default() }).unwrap().len(), 1);
        assert_eq!(lb.clear_test().unwrap(), 3);
        assert_eq!(lb.next_test_serial().unwrap(), 1);
        assert_eq!(lb.list(&LogFilter::default()).unwrap().len(), 1, "the real log is untouched");
    }

    #[test]
    fn search_finds_part_of_a_call_name_qth_or_reference() {
        let lb = Logbook::open_memory().unwrap();
        lb.add(&Qso { name: "Hans".into(), qth: "Berlin".into(), ..q("DL1ABC", 100, "40m") }).unwrap();
        lb.add(&Qso { their_ref: "ONFF-0599".into(), ..q("ON4XYZ", 200, "40m") }).unwrap();
        lb.add(&Qso { comment: "100%_good".into(), ..q("PA3ABC", 300, "20m") }).unwrap();
        let find = |s: &str| lb.list(&LogFilter { search: s.into(), ..Default::default() }).unwrap().into_iter().map(|q| q.call).collect::<Vec<_>>();
        assert_eq!(find("abc"), vec!["PA3ABC", "DL1ABC"]);
        assert_eq!(find("berl"), vec!["DL1ABC"]);
        assert_eq!(find("onff-0599"), vec!["ON4XYZ"]);
        assert_eq!(find("%_"), vec!["PA3ABC"], "wildcards are plain characters");
        assert_eq!(find("_"), vec!["PA3ABC"]);
        assert_eq!(find("  "), vec!["PA3ABC", "ON4XYZ", "DL1ABC"], "an empty search filters nothing");
        let both = lb.list(&LogFilter { search: "abc".into(), band: "40m".into(), ..Default::default() }).unwrap();
        assert_eq!(both.len(), 1);
    }

    #[test]
    fn filter_by_time_and_band() {
        let lb = Logbook::open_memory().unwrap();
        lb.add(&q("A1AA", 100, "40m")).unwrap();
        lb.add(&q("B1BB", 200, "20m")).unwrap();
        lb.add(&q("C1CC", 300, "40m")).unwrap();
        let f = LogFilter { from_ts: Some(150), to_ts: Some(300), band: "40m".into(), ..Default::default() };
        let r = lb.list(&f).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].call, "C1CC");
    }

    #[test]
    fn one_open_activation_at_a_time_and_qsos_can_be_filtered_by_it() {
        let lb = Logbook::open_memory().unwrap();
        let a = lb.start_activation(&Activation { profile: "portable".into(), grid: "jo20qe".into(), wwff: "onff-0599".into(), ..Default::default() }, 1000).unwrap();
        assert_eq!((a.grid.as_str(), a.wwff.as_str(), a.ended_ts), ("JO20QE", "ONFF-0599", 0));
        lb.add(&Qso { activation_id: a.id, ..q("A1AA", 1100, "40m") }).unwrap();
        lb.add(&q("B1BB", 1200, "40m")).unwrap();
        let b = lb.start_activation(&Activation { grid: "JO20AA".into(), ..Default::default() }, 2000).unwrap();
        assert_eq!(lb.current_activation().unwrap().unwrap().id, b.id);
        assert_eq!(lb.activation(a.id).unwrap().unwrap().ended_ts, 2000, "the first one was closed");
        let r = lb.list(&LogFilter { activation: Some(a.id), ..Default::default() }).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].call, "A1AA");
        lb.end_activation(3000).unwrap();
        assert!(lb.current_activation().unwrap().is_none());
        assert_eq!(lb.activations(10).unwrap().len(), 2);
    }

    #[test]
    fn exported_qsos_are_left_out_of_since_the_last_export_and_an_edit_brings_them_back() {
        let lb = Logbook::open_memory().unwrap();
        let a = lb.add(&q("A1AA", 100, "40m")).unwrap();
        let b = lb.add(&q("B1BB", 200, "40m")).unwrap();
        let new = LogFilter { not_exported: true, ..Default::default() };
        assert_eq!(lb.list(&new).unwrap().len(), 2);
        lb.mark_exported(&[a.id]).unwrap();
        lb.mark_exported(&[a.id]).unwrap();
        let r = lb.list(&new).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, b.id);
        assert_eq!(lb.list(&LogFilter::default()).unwrap().iter().find(|x| x.id == a.id).unwrap().exported_to, "adif");
        let mut a2 = lb.list(&LogFilter { limit: 0, ..Default::default() }).unwrap().into_iter().find(|x| x.id == a.id).unwrap();
        a2.name = "FIXED".into();
        lb.update(&a2).unwrap();
        assert_eq!(lb.list(&new).unwrap().len(), 2, "a changed QSO must be sent again");
    }

    #[test]
    fn a_split_qso_keeps_both_frequencies() {
        let lb = Logbook::open_memory().unwrap();
        let a = lb.add(&Qso { ts: 1, call: "3B8XX".into(), freq_hz: 7_026_000, freq_rx_hz: 7_025_000, ..Default::default() }).unwrap();
        let r = lb.list(&LogFilter::default()).unwrap();
        assert_eq!((r[0].freq_hz, r[0].freq_rx_hz, r[0].band.as_str()), (7_026_000, 7_025_000, "40m"));
        // The same frequency twice is simply not split.
        let b = lb.add(&Qso { ts: 2, call: "PA3ABC".into(), freq_hz: 7_030_000, freq_rx_hz: 7_030_000, ..Default::default() }).unwrap();
        assert_eq!(b.freq_rx_hz, 0);
        // Editing changes it, and making both equal clears it.
        let mut e = a.clone();
        e.freq_rx_hz = 7_024_500;
        lb.update(&e).unwrap();
        assert_eq!(lb.list(&LogFilter::default()).unwrap().iter().find(|q| q.id == a.id).unwrap().freq_rx_hz, 7_024_500);
        e.freq_rx_hz = e.freq_hz;
        lb.update(&e).unwrap();
        assert_eq!(lb.list(&LogFilter::default()).unwrap().iter().find(|q| q.id == a.id).unwrap().freq_rx_hz, 0);
    }

    #[test]
    fn an_older_file_without_the_new_columns_is_upgraded() {
        let dir = std::env::temp_dir().join(format!("cwld-log-old-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("logbook.db");
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch("CREATE TABLE qso (id INTEGER PRIMARY KEY AUTOINCREMENT, ts INTEGER NOT NULL, call TEXT NOT NULL, band TEXT NOT NULL DEFAULT '', freq_hz INTEGER NOT NULL DEFAULT 0, mode TEXT NOT NULL DEFAULT 'CW', rst_sent TEXT NOT NULL DEFAULT '', rst_rcvd TEXT NOT NULL DEFAULT '', nr_sent INTEGER NOT NULL DEFAULT 0, nr_rcvd TEXT NOT NULL DEFAULT '', extra_sent TEXT NOT NULL DEFAULT '', extra_rcvd TEXT NOT NULL DEFAULT '', name TEXT NOT NULL DEFAULT '', qth TEXT NOT NULL DEFAULT '', comment TEXT NOT NULL DEFAULT '', contest TEXT NOT NULL DEFAULT '', session INTEGER NOT NULL DEFAULT 0, my_call TEXT NOT NULL DEFAULT '', test INTEGER NOT NULL DEFAULT 0, deleted INTEGER NOT NULL DEFAULT 0);
                             INSERT INTO qso (ts, call) VALUES (5, 'OLD1AA');").unwrap();
        }
        let lb = Logbook::open(&path).unwrap();
        let r = lb.list(&LogFilter::default()).unwrap();
        assert_eq!(r[0].call, "OLD1AA");
        assert_eq!(r[0].activation_id, 0);
        assert_eq!(r[0].freq_rx_hz, 0);
        lb.add(&q("NEW1BB", 6, "20m")).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_survives_reopen_and_keeps_a_backup() {
        let dir = std::env::temp_dir().join(format!("cwld-log-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("logbook.db");
        {
            let lb = Logbook::open(&path).unwrap();
            lb.add(&q("PA3ABC", 1000, "40m")).unwrap();
        }
        let lb = Logbook::open(&path).unwrap();
        assert_eq!(lb.list(&LogFilter::default()).unwrap().len(), 1);
        assert!(path.with_extension("db.bak").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
