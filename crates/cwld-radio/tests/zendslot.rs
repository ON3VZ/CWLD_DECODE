//! The transmit guard must keep every transmit command away from the radio outside Live.

use cwld_radio::{GuardedRig, Rig, RigError, SimRig};
use cwld_types::{CommandClass, PolicyHandle, RadioCommand, RadioMode, TxPolicy};

fn guarded(policy: TxPolicy) -> (GuardedRig<SimRig>, PolicyHandle) {
    let handle = PolicyHandle::new(policy);
    (GuardedRig::new(SimRig::new(), handle.clone()), handle)
}

#[test]
fn no_transmit_command_reaches_the_radio_outside_live() {
    for policy in [TxPolicy::ListenOnly, TxPolicy::Practice] {
        let (mut rig, _) = guarded(policy);
        for cmd in RadioCommand::examples() {
            let result = rig.execute(&cmd);
            if cmd.class() == CommandClass::Transmit {
                assert!(
                    matches!(result, Err(RigError::Blocked { .. })),
                    "{} must be blocked in {policy:?}",
                    cmd.name()
                );
            } else {
                assert!(result.is_ok(), "{} must pass in {policy:?}", cmd.name());
            }
        }
        // The decisive check: the radio itself never saw a transmit command.
        assert_eq!(rig.inner().transmit_command_count(), 0, "policy {policy:?}");
        assert!(rig.inner().received().iter().all(|c| c.class() != CommandClass::Transmit));
        assert!(!rig.status().transmitting);
        assert_eq!(rig.blocked_count(), 4, "send_cw, set_ptt, key_down and raw");
    }
}

#[test]
fn live_passes_transmit_commands() {
    let (mut rig, _) = guarded(TxPolicy::Live);
    rig.execute(&RadioCommand::SendCw("CQ CQ DE TEST".into())).unwrap();
    assert_eq!(rig.inner().transmit_command_count(), 1);
    assert!(rig.status().transmitting);
    assert_eq!(rig.blocked_count(), 0);
}

#[test]
fn stop_is_allowed_in_every_policy() {
    for policy in [TxPolicy::Live, TxPolicy::ListenOnly, TxPolicy::Practice] {
        let (mut rig, _) = guarded(policy);
        assert!(rig.execute(&RadioCommand::StopCw).is_ok(), "{policy:?}");
    }
}

#[test]
fn tuning_works_while_listening() {
    let (mut rig, _) = guarded(TxPolicy::ListenOnly);
    rig.execute(&RadioCommand::SetFrequency(14_030_000)).unwrap();
    rig.execute(&RadioCommand::SetMode(RadioMode::Usb)).unwrap();
    let s = rig.status();
    assert_eq!(s.frequency_hz, 14_030_000);
    assert_eq!(s.mode, RadioMode::Usb);
}

#[test]
fn switching_the_policy_takes_effect_immediately() {
    let (mut rig, handle) = guarded(TxPolicy::Live);
    rig.execute(&RadioCommand::SendCw("E".into())).unwrap();
    handle.set(TxPolicy::ListenOnly);
    let r = rig.execute(&RadioCommand::SendCw("E".into()));
    assert!(matches!(r, Err(RigError::Blocked { policy: TxPolicy::ListenOnly, .. })));
    assert_eq!(rig.inner().transmit_command_count(), 1);
    handle.set(TxPolicy::Live);
    rig.execute(&RadioCommand::SendCw("E".into())).unwrap();
    assert_eq!(rig.inner().transmit_command_count(), 2);
}

#[test]
fn keyer_speed_zero_is_refused_even_in_live() {
    // On a QRP Labs QMX, KS00 would switch the radio to straight-key mode.
    let (mut rig, _) = guarded(TxPolicy::Live);
    for bad in [0u8, 1, 4, 61, 255] {
        let r = rig.execute(&RadioCommand::SetCwSpeed(bad));
        assert!(matches!(r, Err(RigError::InvalidArgument(_))), "{bad}");
    }
    assert!(rig.execute(&RadioCommand::SetCwSpeed(5)).is_ok());
    assert!(rig.execute(&RadioCommand::SetCwSpeed(60)).is_ok());
    assert_eq!(rig.status().cw_wpm, 60);
}

#[test]
fn empty_text_is_not_sent() {
    let (mut rig, _) = guarded(TxPolicy::Live);
    assert!(matches!(rig.execute(&RadioCommand::SendCw("   ".into())), Err(RigError::InvalidArgument(_))));
    assert_eq!(rig.inner().transmit_command_count(), 0);
}

#[test]
fn simulated_transmission_ends_after_the_morse_duration() {
    let (mut rig, _) = guarded(TxPolicy::Live);
    rig.execute(&RadioCommand::SetCwSpeed(20)).unwrap();
    rig.execute(&RadioCommand::SendCw("PARIS".into())).unwrap(); // 2.58 s at 20 WPM
    let mut ticks = 0;
    while rig.status().transmitting && ticks < 50 {
        rig.poll();
        ticks += 1;
    }
    assert_eq!(ticks, 6, "2.58 s is 6 polls of 0.5 s");
    assert!(!rig.status().transmitting);
}

#[test]
fn knob_turns_are_reported() {
    let (mut rig, _) = guarded(TxPolicy::ListenOnly);
    rig.poll();
    rig.inner_mut().turn_knob(7_031_000);
    let events = rig.poll();
    assert!(events.contains(&cwld_types::RigEvent::Frequency(7_031_000)));
}
