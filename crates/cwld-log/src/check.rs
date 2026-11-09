//! Check before an export: what a log file would say, and what is wrong with it.
//!
//! Errors block the export. Warnings are shown but do not block.

use crate::{Activation, Qso};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

/// One finding. `code` is stable, the interface translates it. `id` is the QSO it is about (0: the whole file).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckIssue {
    pub severity: Severity,
    pub code: &'static str,
    pub id: i64,
    pub detail: String,
}

/// A Maidenhead locator of six characters, such as `JO20QE`.
pub fn valid_grid(g: &str) -> bool {
    let b = g.trim().as_bytes();
    b.len() == 6
        && b[0].to_ascii_uppercase().is_ascii_uppercase()
        && (b'A'..=b'R').contains(&b[0].to_ascii_uppercase())
        && (b'A'..=b'R').contains(&b[1].to_ascii_uppercase())
        && b[2].is_ascii_digit()
        && b[3].is_ascii_digit()
        && (b'A'..=b'X').contains(&b[4].to_ascii_uppercase())
        && (b'A'..=b'X').contains(&b[5].to_ascii_uppercase())
}

/// A WWFF reference: two letters, `FF`, a dash and four digits, such as `ONFF-0599`.
pub fn valid_wwff_ref(r: &str) -> bool {
    let b = r.trim().as_bytes();
    b.len() == 9 && b[0].is_ascii_uppercase() && b[1].is_ascii_uppercase() && b[2] == b'F' && b[3] == b'F' && b[4] == b'-' && b[5..].iter().all(|c| c.is_ascii_digit())
}

fn issue(severity: Severity, code: &'static str, id: i64, detail: impl Into<String>) -> CheckIssue {
    CheckIssue { severity, code, id, detail: detail.into() }
}

/// Checks the QSOs that would be exported. `activations` are the ones the QSOs refer to, `now` is the current time.
pub fn check_export(qsos: &[Qso], activations: &[Activation], now: i64) -> Vec<CheckIssue> {
    use Severity::*;
    let mut out = Vec::new();
    if qsos.is_empty() {
        out.push(issue(Error, "empty", 0, ""));
        return out;
    }
    let act = |id: i64| activations.iter().find(|a| a.id == id);

    for q in qsos {
        let on_air = if q.station_call.is_empty() { q.my_call.as_str() } else { q.station_call.as_str() };
        if on_air.is_empty() {
            out.push(issue(Error, "no_station_call", q.id, &q.call));
        }
        if q.call.is_empty() || q.band.is_empty() || q.mode.is_empty() || q.ts <= 0 {
            out.push(issue(Error, "missing_basic", q.id, &q.call));
        }
        if q.rst_sent.is_empty() || q.rst_rcvd.is_empty() {
            out.push(issue(Error, "missing_rst", q.id, &q.call));
        }
        // A portable call or an activation needs the grid of that place.
        let portable = on_air.contains('/') || q.activation_id != 0;
        if q.my_grid.is_empty() {
            out.push(issue(if portable { Error } else { Warning }, "no_grid", q.id, &q.call));
        } else if !valid_grid(&q.my_grid) {
            out.push(issue(Error, "bad_grid", q.id, &q.my_grid));
        }
        for (r, code) in [(&q.my_wwff, "bad_wwff_ref")] {
            if !r.is_empty() && !valid_wwff_ref(r) {
                out.push(issue(Error, code, q.id, r.as_str()));
            }
        }
        if q.their_prog == "WWFF" && !q.their_ref.is_empty() && !valid_wwff_ref(&q.their_ref) {
            out.push(issue(Error, "bad_wwff_ref", q.id, &q.their_ref));
        }
        if let Some(a) = act(q.activation_id) {
            let end = if a.ended_ts == 0 { now } else { a.ended_ts };
            if q.ts < a.started_ts - 60 || q.ts > end + 60 {
                out.push(issue(Error, "outside_activation", q.id, &q.call));
            }
        }
    }

    // One grid and one station call per file: show which ones are in it.
    let grids: BTreeSet<&str> = qsos.iter().map(|q| q.my_grid.as_str()).filter(|g| !g.is_empty()).collect();
    if grids.len() > 1 {
        out.push(issue(Warning, "several_grids", 0, grids.into_iter().collect::<Vec<_>>().join(" ")));
    }
    let calls: BTreeSet<&str> = qsos.iter().map(|q| if q.station_call.is_empty() { q.my_call.as_str() } else { q.station_call.as_str() }).filter(|c| !c.is_empty()).collect();
    if calls.len() > 1 {
        out.push(issue(Warning, "several_station_calls", 0, calls.into_iter().collect::<Vec<_>>().join(" ")));
    }

    // Dupes inside one activation (or inside the normal log): same call, band and mode.
    let mut seen: BTreeMap<(i64, &str, &str, &str), i64> = BTreeMap::new();
    let mut sorted: Vec<&Qso> = qsos.iter().collect();
    sorted.sort_by_key(|q| (q.ts, q.id));
    for q in sorted {
        if q.contest.is_empty() {
            let key = (q.activation_id, q.call.as_str(), q.band.as_str(), q.mode.as_str());
            if seen.insert(key, q.id).is_some() {
                out.push(issue(Warning, "dupe", q.id, &q.call));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> Qso {
        Qso { id: 1, ts: 1_000_000, call: "DK1AB".into(), band: "40m".into(), mode: "CW".into(), rst_sent: "599".into(), rst_rcvd: "599".into(), station_call: "ON3VZ/P".into(), my_grid: "JO20QE".into(), ..Default::default() }
    }
    fn codes(v: &[CheckIssue]) -> Vec<&'static str> {
        v.iter().map(|i| i.code).collect()
    }

    #[test]
    fn grid_and_reference_forms() {
        assert!(valid_grid("JO20QE") && valid_grid("jo20qe"));
        assert!(!valid_grid("JO20") && !valid_grid("ZZ20QE") && !valid_grid("JO20QZ"));
        assert!(valid_wwff_ref("ONFF-0599"));
        assert!(!valid_wwff_ref("ONFF-599") && !valid_wwff_ref("ONFF0599") && !valid_wwff_ref("onff-0599") && !valid_wwff_ref("ON-0599"));
    }

    #[test]
    fn a_good_qso_has_no_findings_and_an_empty_export_is_an_error() {
        assert!(check_export(&[good()], &[], 2_000_000).is_empty());
        assert_eq!(codes(&check_export(&[], &[], 0)), vec!["empty"]);
    }

    #[test]
    fn a_portable_qso_without_a_grid_is_an_error_but_a_home_qso_only_a_warning() {
        let mut p = good();
        p.my_grid.clear();
        let r = check_export(&[p], &[], 2_000_000);
        assert_eq!(r[0].code, "no_grid");
        assert_eq!(r[0].severity, Severity::Error);
        let mut h = good();
        h.my_grid.clear();
        h.station_call = "ON3VZ".into();
        let r = check_export(&[h], &[], 2_000_000);
        assert_eq!(r[0].severity, Severity::Warning);
    }

    #[test]
    fn bad_grid_bad_reference_and_missing_fields_are_errors() {
        let mut q = good();
        q.my_grid = "JO20".into();
        q.my_wwff = "ONFF-599".into();
        q.rst_rcvd.clear();
        let c = codes(&check_export(&[q], &[], 2_000_000));
        assert!(c.contains(&"bad_grid") && c.contains(&"bad_wwff_ref") && c.contains(&"missing_rst"));
    }

    #[test]
    fn a_qso_outside_its_activation_is_an_error() {
        let a = Activation { id: 7, started_ts: 500_000, ended_ts: 900_000, ..Default::default() };
        let mut q = good();
        q.activation_id = 7;
        q.ts = 1_000_000;
        assert!(codes(&check_export(&[q.clone()], &[a.clone()], 2_000_000)).contains(&"outside_activation"));
        q.ts = 600_000;
        assert!(!codes(&check_export(&[q], &[a], 2_000_000)).contains(&"outside_activation"));
    }

    #[test]
    fn dupes_and_mixed_grids_are_warnings() {
        let a = good();
        let mut b = good();
        b.id = 2;
        b.ts += 60;
        b.my_grid = "JO20AA".into();
        let r = check_export(&[a, b], &[], 2_000_000);
        let c = codes(&r);
        assert!(c.contains(&"dupe") && c.contains(&"several_grids"));
        assert!(r.iter().all(|i| i.severity == Severity::Warning));
    }
}
