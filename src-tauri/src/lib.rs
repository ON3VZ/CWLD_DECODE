//! Thin Tauri shell around `cwld-core`. All rules live in the core, so they are tested there.

use cwld_core::{AppCore, AudioStatus, CoreError, CoreEvent, InputDevice, SendOutcome, Snapshot};
use cwld_radio::PortInfo;
use cwld_types::{RadioSettings, Settings, TxPolicy};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

/// Error as the interface receives it: a stable code, translated on the other side.
#[derive(Serialize)]
pub struct Failure {
    code: &'static str,
    message: String,
}

impl From<CoreError> for Failure {
    fn from(e: CoreError) -> Self {
        Failure { code: e.code(), message: e.to_string() }
    }
}

type Core<'a> = State<'a, Arc<AppCore>>;

#[tauri::command]
fn get_snapshot(core: Core) -> Snapshot {
    core.snapshot()
}

#[tauri::command]
fn set_policy(core: Core, policy: TxPolicy, confirmed: bool) -> Result<(), Failure> {
    core.set_policy(policy, confirmed)?;
    let _ = core.save_settings();
    Ok(())
}

#[tauri::command]
fn send_text(core: Core, text: String) -> Result<SendOutcome, Failure> {
    Ok(core.send_text(&text)?)
}

#[tauri::command]
fn stop_tx(core: Core) -> Result<(), Failure> {
    Ok(core.stop_tx()?)
}

#[tauri::command]
fn set_speed(core: Core, wpm: u8) -> Result<(), Failure> {
    Ok(core.set_speed(wpm)?)
}

#[tauri::command]
fn set_frequency(core: Core, hz: u64) -> Result<(), Failure> {
    Ok(core.set_frequency(hz)?)
}

#[tauri::command]
fn save_settings(core: Core, settings: Settings) -> Result<(), Failure> {
    Ok(core.update_settings(settings)?)
}

#[tauri::command]
fn list_ports(core: Core) -> Vec<PortInfo> {
    core.list_ports()
}

#[tauri::command]
fn connect(core: Core, radio: RadioSettings) -> Result<Snapshot, Failure> {
    core.connect(&radio)?;
    Ok(core.snapshot())
}

#[tauri::command]
fn disconnect(core: Core) -> Snapshot {
    core.disconnect();
    core.snapshot()
}

#[tauri::command]
fn ack_swr(core: Core) -> Vec<CoreEvent> {
    core.ack_swr()
}

#[tauri::command]
fn list_audio_inputs(core: Core) -> Vec<InputDevice> {
    core.list_audio_inputs()
}

#[tauri::command]
fn start_audio(core: Core) -> Result<AudioStatus, Failure> {
    Ok(core.start_audio()?)
}

#[tauri::command]
fn stop_audio(core: Core) -> AudioStatus {
    core.stop_audio()
}

#[tauri::command]
fn decode_file(core: Core, path: String) -> Result<AudioStatus, Failure> {
    Ok(core.start_audio_file(path.into())?)
}

#[tauri::command]
fn set_squelch(core: Core, value: f32) -> Result<(), Failure> {
    core.set_squelch(value)?;
    let _ = core.save_settings();
    Ok(())
}

#[tauri::command]
fn set_tuning(core: Core, gain_db: f32, wide_filter: bool, min_wpm: u8, max_wpm: u8, lock_wpm: u8) -> Result<(), Failure> {
    core.set_tuning(gain_db, wide_filter, min_wpm, max_wpm, lock_wpm)?;
    let _ = core.save_settings();
    Ok(())
}

#[tauri::command]
fn set_afc(core: Core, on: bool) -> Result<(), Failure> {
    core.set_afc(on)?;
    let _ = core.save_settings();
    Ok(())
}

#[tauri::command]
fn go_strongest(core: Core) -> Result<(), Failure> {
    core.go_strongest()?;
    Ok(())
}

#[tauri::command]
fn set_auto_tone(core: Core, on: bool) -> Result<(), Failure> {
    core.set_auto_tone(on)?;
    let _ = core.save_settings();
    Ok(())
}

#[tauri::command]
fn set_tone(core: Core, hz: f64) -> Result<(), Failure> {
    core.set_tone(hz)?;
    let _ = core.save_settings();
    Ok(())
}

#[tauri::command]
fn start_recording(core: Core) -> Result<String, Failure> {
    Ok(core.start_recording()?)
}

#[tauri::command]
fn stop_recording(core: Core) {
    core.stop_recording()
}

/// Before an update is installed: let go of the radio port and the sound card so that the
/// installer finds nothing left to wait for.
#[tauri::command]
fn prepare_update(core: Core) {
    release_within(core.inner().clone(), RELEASE_WAIT);
}

/// How long CWLD waits for the radio port and the sound card to close before it goes on.
/// A USB driver that does not answer must not keep CWLD (or an update) waiting for ever.
const RELEASE_WAIT: Duration = Duration::from_millis(1500);

/// Lets go of the radio and the sound card, but waits at most `wait`.
fn release_within(core: Arc<AppCore>, wait: Duration) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        core.prepare_exit();
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(wait);
}

/// Ends the process at once. The normal end of a Windows program runs the clean-up of every
/// library, and a sound or USB library that hangs there leaves cwld.exe behind, which blocks
/// the installer of the next version.
#[cfg(windows)]
fn end_process_now() -> ! {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn TerminateProcess(process: *mut std::ffi::c_void, exit_code: u32) -> i32;
    }
    // SAFETY: both are plain Win32 calls on the pseudo handle of this process.
    unsafe {
        TerminateProcess(GetCurrentProcess(), 0);
    }
    std::process::exit(0)
}

#[cfg(not(windows))]
fn end_process_now() -> ! {
    std::process::exit(0)
}

/// Every 100 ms the safety readings (SWR), every fifth round the full status.
fn start_ticker(app: AppHandle, core: Arc<AppCore>) {
    std::thread::spawn(move || {
        let mut round = 0u32;
        loop {
            std::thread::sleep(Duration::from_millis(100));
            round = round.wrapping_add(1);
            let mut events: Vec<CoreEvent> = core.fast_tick();
            events.extend(core.audio_tick());
            if round % 5 == 0 {
                events.extend(core.tick());
            }
            if !events.is_empty() {
                let _ = app.emit("core-events", events);
            }
        }
    });
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let core = Arc::new(AppCore::new(Some(dir.join("settings.json"))));
            if let Ok(docs) = app.path().document_dir() {
                core.set_default_recordings_dir(docs.join("CWLD"));
            }
            if core.settings().audio.auto_start {
                // A missing sound card is not fatal: the status shows the error.
                let _ = core.start_audio();
            }
            // Connect to the saved radio. A radio that is off is not fatal: the status shows it.
            let _ = core.auto_connect();
            app.manage(core.clone());
            start_ticker(app.handle().clone(), core);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            set_policy,
            send_text,
            stop_tx,
            set_speed,
            set_frequency,
            save_settings,
            list_ports,
            connect,
            disconnect,
            ack_swr,
            list_audio_inputs,
            start_audio,
            stop_audio,
            decode_file,
            set_tone,
            set_squelch,
            set_tuning,
            set_afc,
            go_strongest,
            set_auto_tone,
            start_recording,
            stop_recording,
            prepare_update
        ])
        .build(tauri::generate_context!())
        .expect("error while building CWLD")
        .run(|app, event| {
            // When CWLD ends, free the radio and the sound card and make sure the process is
            // really gone (a lingering process stops the installer of an update).
            if let tauri::RunEvent::Exit = event {
                if let Some(core) = app.try_state::<Arc<AppCore>>() {
                    release_within(core.inner().clone(), RELEASE_WAIT);
                }
                end_process_now();
            }
        });
}
