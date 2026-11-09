//! Logbook access of the core: thin wrappers so the interface never touches the database.

use crate::{AppCore, CoreError};
use cwld_radio::Rig;
use cwld_log::{adif_text, check_export, valid_grid, valid_wwff_ref, wwff_file_name, Activation, AdifOptions, CheckIssue, LogError, LogFilter, Qso, Severity};
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

impl From<LogError> for CoreError {
    fn from(e: LogError) -> Self {
        CoreError::Log(e.to_string())
    }
}

/// Result of a dupe check: the earlier QSO (if any) and the other bands this call was worked on.
#[derive(Debug, Clone, Serialize)]
pub struct DupeInfo {
    pub dupe: Option<Qso>,
    pub other_bands: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivationStats {
    pub total: usize,
    pub unique: usize,
    pub crosses_midnight: bool,
}

/// An exported file: where it was saved and how many QSOs it holds.
#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub path: String,
    pub count: usize,
    pub text: String,
}

fn now_ts() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

impl AppCore {
    /// Stores a QSO. A QSO without a time gets the current UTC time.
    pub fn log_add(&self, mut q: Qso) -> Result<Qso, CoreError> {
        if q.ts == 0 {
            q.ts = now_ts();
        }
        if q.freq_hz == 0 {
            q.freq_hz = self.rig.lock().unwrap().status().frequency_hz;
        }
        let lb = self.logbook.lock().unwrap();
        // The station the QSO is made from, copied now so a later change of profile cannot alter it.
        let s = self.settings.lock().unwrap();
        let base = s.station.calls.first().cloned().unwrap_or_else(|| q.my_call.clone());
        let profile = s.profiles.iter().find(|p| p.id == s.profile).cloned().unwrap_or_default();
        let fill = |field: &mut String, value: String| {
            if field.trim().is_empty() {
                *field = value;
            }
        };
        fill(&mut q.station_call, profile.on_air(&base));
        fill(&mut q.operator, base.to_uppercase());
        let act = if q.test { None } else { lb.current_activation()? };
        if let Some(a) = &act {
            q.activation_id = a.id;
            fill(&mut q.my_wwff, a.wwff.clone());
            fill(&mut q.my_pota, a.pota.clone());
            fill(&mut q.my_sota, a.sota.clone());
        }
        let grid = act.as_ref().map(|a| a.grid.clone()).filter(|g| !g.is_empty()).or_else(|| Some(profile.grid.clone()).filter(|g| !g.is_empty())).unwrap_or_else(|| s.station.locator.clone());
        let power = act.as_ref().map(|a| a.power.clone()).filter(|p| !p.is_empty()).unwrap_or_else(|| profile.power.clone());
        fill(&mut q.my_grid, grid.to_uppercase());
        fill(&mut q.tx_pwr, power);
        fill(&mut q.my_rig, profile.rig.clone());
        fill(&mut q.my_antenna, profile.antenna.clone());
        fill(&mut q.comment, profile.comment.clone());
        drop(s);
        let stored = lb.add(&q)?;
        drop(lb);
        // A failing send never blocks logging; the interface can resend the last QSO from Settings.
        if !stored.test {
            self.link_after_add(&stored);
        }
        Ok(stored)
    }

    /// Opens an activation with the checks the design asks for. An open one is closed first.
    pub fn activation_start(&self, mut a: Activation) -> Result<Activation, CoreError> {
        let s = self.settings.lock().unwrap();
        let profile = s.profiles.iter().find(|p| p.id == s.profile).cloned().unwrap_or_default();
        let home_grid = s.station.locator.clone();
        drop(s);
        a.profile = profile.id.clone();
        a.wwff = a.wwff.trim().to_uppercase();
        if !a.wwff.is_empty() && !valid_wwff_ref(&a.wwff) {
            return Err(CoreError::Invalid("activation_wwff"));
        }
        let grid = if a.grid.trim().is_empty() { profile.grid.clone() } else { a.grid.clone() };
        if grid.trim().is_empty() {
            // Portable stations must give the grid of this place. At home the station locator is used.
            if !profile.suffix.is_empty() || home_grid.is_empty() {
                return Err(CoreError::Invalid("activation_grid_missing"));
            }
        } else if !valid_grid(&grid) {
            return Err(CoreError::Invalid("activation_grid"));
        }
        a.grid = grid;
        if a.power.trim().is_empty() {
            a.power = profile.power.clone();
        }
        Ok(self.logbook.lock().unwrap().start_activation(&a, now_ts())?)
    }

    pub fn activation_end(&self) -> Result<(), CoreError> {
        Ok(self.logbook.lock().unwrap().end_activation(now_ts())?)
    }

    pub fn activation_current(&self) -> Result<Option<Activation>, CoreError> {
        Ok(self.logbook.lock().unwrap().current_activation()?)
    }

    /// Counts for the activation panel: all QSOs, the ones left after dupes, and whether the activation crosses UTC midnight.
    pub fn activation_stats(&self, id: i64) -> Result<ActivationStats, CoreError> {
        let lb = self.logbook.lock().unwrap();
        let qsos = lb.list(&LogFilter { activation: Some(id), test: Some(false), ..Default::default() })?;
        let mut seen = std::collections::BTreeSet::new();
        let unique = qsos.iter().filter(|q| seen.insert((q.call.clone(), q.band.clone(), q.mode.clone()))).count();
        let act = lb.activation(id)?;
        let crosses_midnight = act.map(|a| {
            let end = if a.ended_ts == 0 { now_ts() } else { a.ended_ts };
            a.started_ts.div_euclid(86_400) != end.div_euclid(86_400)
        });
        Ok(ActivationStats { total: qsos.len(), unique, crosses_midnight: crosses_midnight.unwrap_or(false) })
    }

    /// What an export of these QSOs would contain and what is wrong with it.
    pub fn log_check(&self, f: &LogFilter) -> Result<Vec<CheckIssue>, CoreError> {
        let lb = self.logbook.lock().unwrap();
        let qsos = lb.list(&LogFilter { limit: 0, ..f.clone() })?;
        let acts = lb.activations(200)?;
        Ok(check_export(&qsos, &acts, now_ts()))
    }

    pub fn log_list(&self, f: &LogFilter) -> Result<Vec<Qso>, CoreError> {
        Ok(self.logbook.lock().unwrap().list(f)?)
    }

    pub fn log_update(&self, q: &Qso) -> Result<(), CoreError> {
        Ok(self.logbook.lock().unwrap().update(q)?)
    }

    pub fn log_delete(&self, id: i64) -> Result<(), CoreError> {
        Ok(self.logbook.lock().unwrap().delete(id)?)
    }

    pub fn log_clear_test(&self) -> Result<u64, CoreError> {
        Ok(self.logbook.lock().unwrap().clear_test()?)
    }

    pub fn log_next_test_serial(&self) -> Result<u32, CoreError> {
        Ok(self.logbook.lock().unwrap().next_test_serial()?)
    }

    /// Dupe check for the call being worked, on the band the radio is on now.
    pub fn log_dupe(&self, call: &str, contest: &str, session: u32, test: bool) -> Result<DupeInfo, CoreError> {
        let hz = self.rig.lock().unwrap().status().frequency_hz;
        let band = cwld_log::band_of(hz);
        let lb = self.logbook.lock().unwrap();
        let dupe = lb.dupe(call, band, "CW", contest, session, test)?;
        let other_bands = lb.worked_bands(call, contest, session, test)?.into_iter().filter(|b| b != band).collect();
        Ok(DupeInfo { dupe, other_bands })
    }

    fn export_dir(&self) -> PathBuf {
        if let Some(d) = self.recordings_dir.lock().unwrap().clone() {
            return d;
        }
        match &self.settings_path {
            Some(p) => p.parent().map(|d| d.join("exports")).unwrap_or_else(|| PathBuf::from("exports")),
            None => std::env::temp_dir().join("cwld-exports"),
        }
    }

    /// Writes the selected QSOs as an ADIF file in the CWLD folder and returns the path.
    /// `contest_id` is the ADIF CONTEST_ID for contest QSOs (the operator can check it first).
    /// Errors found by the check block the export. QSOs of the real log are marked as exported.
    pub fn log_export(&self, f: &LogFilter, contest_id: &str, version: &str) -> Result<ExportResult, CoreError> {
        let (qsos, acts) = {
            let lb = self.logbook.lock().unwrap();
            (lb.list(&LogFilter { limit: 0, ..f.clone() })?, lb.activations(200)?)
        };
        let now = now_ts();
        if check_export(&qsos, &acts, now).iter().any(|i| i.severity == Severity::Error) {
            return Err(CoreError::Invalid("export_check_failed"));
        }
        let text = adif_text(&qsos, &AdifOptions { contest_id: contest_id.trim().to_string(), program_version: version.to_string(), now_ts: now });
        let dir = self.export_dir();
        std::fs::create_dir_all(&dir).map_err(|e| CoreError::Log(e.to_string()))?;
        // One activation with a WWFF reference gets the name the WWFF coordinator asks for.
        let wwff_name = f.activation.and_then(|id| acts.iter().find(|a| a.id == id)).filter(|a| !a.wwff.is_empty()).map(|a| {
            let call = qsos.first().map(|q| if q.station_call.is_empty() { q.my_call.clone() } else { q.station_call.clone() }).unwrap_or_default();
            wwff_file_name(&call, &a.wwff, a.started_ts)
        });
        let name = wwff_name.unwrap_or_else(|| {
            let (d, t) = cwld_log::adif_date_time(now);
            format!("cwld-log-{d}-{}.adi", &t[..4])
        });
        let path = dir.join(name);
        std::fs::write(&path, text.as_bytes()).map_err(|e| CoreError::Log(e.to_string()))?;
        if f.test != Some(true) {
            let ids: Vec<i64> = qsos.iter().filter(|q| !q.test).map(|q| q.id).collect();
            self.logbook.lock().unwrap().mark_exported(&ids)?;
        }
        Ok(ExportResult { path: path.display().to_string(), count: qsos.len(), text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> AppCore {
        AppCore::new(None)
    }

    fn portable_core() -> AppCore {
        let c = core();
        let mut s = c.settings();
        s.station.calls = vec!["ON3VZ".into()];
        s.station.locator = "JO20EE".into();
        s.profile = "portable".into();
        s.profiles[1].power = "5".into();
        s.profiles[1].rig = "QRP Labs QMX".into();
        s.profiles[1].comment = "5W QRP".into();
        c.update_settings(s).unwrap();
        c
    }

    fn rst() -> Qso {
        Qso { call: "DK1AB".into(), rst_sent: "599".into(), rst_rcvd: "599".into(), ..Default::default() }
    }

    #[test]
    fn a_qso_takes_the_station_from_the_profile_and_the_activation_from_the_open_one() {
        let c = portable_core();
        c.set_frequency(7_025_400).unwrap();
        let a = c.activation_start(Activation { grid: "jo20qe".into(), wwff: "onff-0599".into(), ..Default::default() }).unwrap();
        assert_eq!((a.grid.as_str(), a.wwff.as_str(), a.power.as_str()), ("JO20QE", "ONFF-0599", "5"));
        let q = c.log_add(rst()).unwrap();
        assert_eq!(q.station_call, "ON3VZ/P");
        assert_eq!(q.operator, "ON3VZ");
        assert_eq!((q.my_grid.as_str(), q.tx_pwr.as_str(), q.my_rig.as_str(), q.comment.as_str()), ("JO20QE", "5", "QRP Labs QMX", "5W QRP"));
        assert_eq!((q.activation_id, q.my_wwff.as_str()), (a.id, "ONFF-0599"));
        let t = c.log_add(Qso { test: true, ..rst() }).unwrap();
        assert_eq!(t.activation_id, 0, "the test log never joins an activation");
        let st = c.activation_stats(a.id).unwrap();
        assert_eq!((st.total, st.unique, st.crosses_midnight), (1, 1, false));
    }

    #[test]
    fn at_home_the_locator_of_the_station_is_the_grid_and_no_activation_is_needed() {
        let c = core();
        let mut s = c.settings();
        s.station.calls = vec!["ON3VZ".into()];
        s.station.locator = "jo20ee".into();
        c.update_settings(s).unwrap();
        let q = c.log_add(rst()).unwrap();
        assert_eq!((q.station_call.as_str(), q.my_grid.as_str(), q.activation_id), ("ON3VZ", "JO20EE", 0));
    }

    #[test]
    fn an_activation_needs_a_correct_reference_and_a_portable_one_needs_a_grid() {
        let c = portable_core();
        assert_eq!(c.activation_start(Activation { grid: "JO20QE".into(), wwff: "ONFF-599".into(), ..Default::default() }).unwrap_err().code(), "activation_wwff");
        assert_eq!(c.activation_start(Activation { wwff: "ONFF-0599".into(), ..Default::default() }).unwrap_err().code(), "activation_grid_missing");
        assert_eq!(c.activation_start(Activation { grid: "JO20".into(), ..Default::default() }).unwrap_err().code(), "activation_grid");
        assert!(c.activation_current().unwrap().is_none());
    }

    #[test]
    fn the_export_check_blocks_errors_names_the_wwff_file_and_marks_what_was_sent() {
        let c = portable_core();
        let a = c.activation_start(Activation { grid: "JO20QE".into(), wwff: "ONFF-0599".into(), ..Default::default() }).unwrap();
        let q = c.log_add(rst()).unwrap();
        c.log_add(Qso { call: "OK1XX".into(), rst_sent: String::new(), ..rst() }).unwrap();
        let f = LogFilter { activation: Some(a.id), ..Default::default() };
        assert!(c.log_check(&f).unwrap().iter().any(|i| i.code == "missing_rst" && i.severity == Severity::Error));
        assert_eq!(c.log_export(&f, "", "0.8.0").unwrap_err().code(), "export_check_failed");
        // Only the good one: export by call is not possible, so fix the bad one.
        let bad = c.log_list(&LogFilter { activation: Some(a.id), ..Default::default() }).unwrap().into_iter().find(|x| x.call == "OK1XX").unwrap();
        c.log_update(&Qso { rst_sent: "599".into(), ..bad }).unwrap();
        let r = c.log_export(&f, "", "0.8.0").unwrap();
        assert_eq!(r.count, 2);
        assert!(r.path.contains("ON3VZ-P @ ONFF-0599 "), "{}", r.path);
        assert!(r.text.contains("<MY_WWFF_REF:9>ONFF-0599") && r.text.contains("<STATION_CALLSIGN:7>ON3VZ/P"));
        assert!(c.log_list(&LogFilter { not_exported: true, ..Default::default() }).unwrap().is_empty());
        let _ = (q, std::fs::remove_file(&r.path));
    }

    #[test]
    fn a_logged_qso_gets_time_and_band_and_can_be_exported() {
        let c = core();
        c.set_frequency(7_030_000).unwrap();
        let q = c.log_add(Qso { call: "pa3abc".into(), contest: "uba-dx-cw".into(), session: 1, rst_sent: "599".into(), rst_rcvd: "599".into(), my_call: "ON3VZ".into(), ..Default::default() }).unwrap();
        assert!(q.ts > 1_700_000_000);
        assert_eq!(q.band, "40m");
        assert_eq!(q.freq_hz, 7_030_000);
        let info = c.log_dupe("PA3ABC", "uba-dx-cw", 1, false).unwrap();
        assert!(info.dupe.is_some());
        let r = c.log_export(&LogFilter::default(), "UBA-DX-CW", "0.7.0").unwrap();
        assert_eq!(r.count, 1);
        assert!(r.text.contains("<CALL:6>PA3ABC"));
        assert!(std::path::Path::new(&r.path).exists());
        let _ = std::fs::remove_file(&r.path);
    }
}
