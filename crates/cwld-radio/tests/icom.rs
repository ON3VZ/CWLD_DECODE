use cwld_radio::civ::{self, encode, CONTROLLER};
use cwld_radio::{GuardedRig, IcomRig, ManualClock, MockHandle, MockTransport, Rig, RigError};
use cwld_types::{PolicyHandle, RadioCommand, RadioMode, RigEvent, TxPolicy};

const RADIO: u8 = 0xB6;

fn rig(pipeline: bool) -> (IcomRig<MockTransport>, MockHandle, ManualClock) {
    let (t, h) = MockTransport::new();
    let clock = ManualClock::default();
    (IcomRig::new(t, Box::new(clock.clone()), RADIO, pipeline), h, clock)
}

fn from_radio(body: &[u8]) -> Vec<u8> {
    encode(CONTROLLER, RADIO, body)
}

/// Splits what the driver wrote into frames.
fn written_frames(h: &MockHandle) -> Vec<civ::Frame> {
    civ::Parser::default().push(&h.take_written())
}

fn cw_blocks(frames: &[civ::Frame]) -> Vec<String> {
    frames
        .iter()
        .filter(|f| f.body.first() == Some(&0x17) && f.body.get(1) != Some(&0xFF))
        .map(|f| String::from_utf8(f.body[1..].to_vec()).unwrap())
        .collect()
}

#[test]
fn poll_asks_for_the_five_status_values() {
    let (mut r, h, _c) = rig(false);
    r.poll();
    let frames = written_frames(&h);
    let bodies: Vec<Vec<u8>> = frames.iter().map(|f| f.body.clone()).collect();
    assert_eq!(bodies, vec![vec![0x03], vec![0x04], vec![0x1C, 0x00], vec![0x14, 0x0C], vec![0x15, 0x02]]);
    assert!(frames.iter().all(|f| f.to == RADIO && f.from == CONTROLLER));
}

#[test]
fn frequency_push_and_answer_become_events() {
    let (mut r, h, _c) = rig(false);
    h.feed(&from_radio(&[0x00, 0x00, 0x40, 0x07, 0x14, 0x00])); // transceive push: 14.074 MHz
    let ev = r.poll();
    assert!(ev.contains(&RigEvent::Connected(true)));
    assert!(ev.contains(&RigEvent::Frequency(14_074_000)));
    assert_eq!(r.status().frequency_hz, 14_074_000);
    h.feed(&from_radio(&[0x03, 0x00, 0x00, 0x03, 0x07, 0x00])); // answer to a read: 7.030 MHz
    assert!(r.poll().contains(&RigEvent::Frequency(7_030_000)));
}

#[test]
fn echo_of_our_own_commands_is_ignored() {
    let (mut r, h, _c) = rig(false);
    h.feed(&encode(RADIO, CONTROLLER, &[0x05, 0x00, 0x40, 0x07, 0x14, 0x00]));
    let ev = r.poll();
    assert!(!ev.iter().any(|e| matches!(e, RigEvent::Frequency(_))));
    assert!(!r.status().connected);
}

#[test]
fn mode_speed_smeter_and_tx_are_read() {
    let (mut r, h, _c) = rig(false);
    h.feed(&from_radio(&[0x04, 0x01, 0x02]));
    h.feed(&from_radio(&[0x14, 0x0C, 0x01, 0x46]));
    h.feed(&from_radio(&[0x15, 0x02, 0x01, 0x20]));
    h.feed(&from_radio(&[0x1C, 0x00, 0x01]));
    let ev = r.poll();
    assert!(ev.contains(&RigEvent::Mode(RadioMode::Usb)));
    assert!(ev.contains(&RigEvent::CwSpeed(30)));
    assert!(ev.contains(&RigEvent::SMeter(47)));
    assert!(ev.contains(&RigEvent::Tx(true)));
    assert_eq!(r.status().filter, Some(2));
}

#[test]
fn set_frequency_writes_exact_bytes() {
    let (mut r, h, _c) = rig(false);
    r.execute(&RadioCommand::SetFrequency(14_074_000)).unwrap();
    assert_eq!(h.take_written(), vec![0xFE, 0xFE, 0xB6, 0xE0, 0x05, 0x00, 0x40, 0x07, 0x14, 0x00, 0xFD]);
}

#[test]
fn set_speed_writes_exact_bytes_and_checks_the_range() {
    let (mut r, h, _c) = rig(false);
    r.execute(&RadioCommand::SetCwSpeed(20)).unwrap();
    assert_eq!(h.take_written(), vec![0xFE, 0xFE, 0xB6, 0xE0, 0x14, 0x0C, 0x00, 0x85, 0xFD]);
    assert!(matches!(r.execute(&RadioCommand::SetCwSpeed(5)), Err(RigError::InvalidArgument(_))));
    assert!(matches!(r.execute(&RadioCommand::SetCwSpeed(49)), Err(RigError::InvalidArgument(_))));
    assert!(h.take_written().is_empty());
}

#[test]
fn short_text_is_one_block() {
    let (mut r, h, _c) = rig(false);
    r.execute(&RadioCommand::SendCw("cq de on3vz".into())).unwrap();
    assert_eq!(cw_blocks(&written_frames(&h)), vec!["CQ DE ON3VZ".to_string()]);
}

#[test]
fn long_text_goes_out_block_by_block_never_over_30_characters() {
    let (mut r, h, clock) = rig(false);
    let text = "CQ CQ CQ DE ON3VZ ON3VZ ON3VZ PSE K CQ CQ CQ DE ON3VZ ON3VZ ON3VZ PSE K";
    r.execute(&RadioCommand::SendCw(text.into())).unwrap();
    let mut sent = cw_blocks(&written_frames(&h));
    assert_eq!(sent.len(), 1, "only one block at a time");
    h.feed(&from_radio(&[civ::OK]));
    // While the first block is being sent nothing else goes out.
    clock.advance(1_000);
    r.poll();
    assert!(cw_blocks(&written_frames(&h)).is_empty());
    // After the estimated time the next block follows.
    for _ in 0..200 {
        clock.advance(500);
        r.poll();
        let b = cw_blocks(&written_frames(&h));
        sent.extend(b);
        h.feed(&from_radio(&[civ::OK]));
    }
    assert!(sent.len() >= 2);
    assert!(sent.iter().all(|b| b.len() <= 30), "{sent:?}");
    assert_eq!(sent.concat(), text);
}

#[test]
fn next_block_waits_while_the_radio_reports_tx() {
    let (mut r, h, clock) = rig(false);
    r.execute(&RadioCommand::SendCw("CQ CQ CQ DE ON3VZ ON3VZ ON3VZ PSE K CQ CQ".into())).unwrap();
    written_frames(&h);
    clock.advance(60_000); // far beyond the estimate
    h.feed(&from_radio(&[0x1C, 0x00, 0x01])); // but the radio still transmits
    r.poll();
    assert!(cw_blocks(&written_frames(&h)).is_empty());
    h.feed(&from_radio(&[0x1C, 0x00, 0x00]));
    r.poll();
    assert_eq!(cw_blocks(&written_frames(&h)).len(), 1);
}

#[test]
fn stop_sends_ff_and_empties_the_queue() {
    let (mut r, h, clock) = rig(false);
    r.execute(&RadioCommand::SendCw("CQ CQ CQ DE ON3VZ ON3VZ ON3VZ PSE K CQ CQ CQ".into())).unwrap();
    written_frames(&h);
    r.execute(&RadioCommand::StopCw).unwrap();
    let frames = written_frames(&h);
    assert_eq!(frames.last().unwrap().body, vec![0x17, 0xFF]);
    clock.advance(60_000);
    r.poll();
    assert!(cw_blocks(&written_frames(&h)).is_empty(), "queue must be empty after stop");
}

#[test]
fn radio_refusing_a_block_without_pipeline_is_retried_then_given_up() {
    let (mut r, h, clock) = rig(false);
    r.execute(&RadioCommand::SendCw("CQ".into())).unwrap();
    let mut total = cw_blocks(&written_frames(&h)).len();
    let mut warned = false;
    for _ in 0..6 {
        h.feed(&from_radio(&[civ::NG]));
        clock.advance(1_000);
        let ev = r.poll();
        warned |= ev.contains(&RigEvent::Warning("civ_rejected".into()));
        total += cw_blocks(&written_frames(&h)).len();
    }
    assert!(warned);
    assert!((2..=4).contains(&total), "{total}");
}

#[test]
fn pipelining_switches_itself_off_when_the_radio_says_no() {
    let (mut r, h, _c) = rig(true);
    r.execute(&RadioCommand::SendCw("CQ".into())).unwrap();
    written_frames(&h);
    h.feed(&from_radio(&[civ::NG]));
    let ev = r.poll();
    assert!(ev.contains(&RigEvent::Warning("civ_pipeline_off".into())));
}

#[test]
fn warns_when_the_radio_does_not_start_transmitting() {
    let (mut r, h, clock) = rig(false);
    h.feed(&from_radio(&[0x1C, 0x00, 0x00]));
    r.poll();
    r.execute(&RadioCommand::SendCw("CQ".into())).unwrap();
    clock.advance(2_500);
    h.feed(&from_radio(&[0x1C, 0x00, 0x00]));
    assert!(r.poll().contains(&RigEvent::Warning("radio_not_transmitting".into())));
}

#[test]
fn silence_means_disconnected() {
    let (mut r, h, clock) = rig(false);
    h.feed(&from_radio(&[0x00, 0x00, 0x40, 0x07, 0x14, 0x00]));
    r.poll();
    assert!(r.status().connected);
    clock.advance(3_500);
    let ev = r.poll();
    assert!(ev.contains(&RigEvent::Connected(false)));
    assert!(ev.contains(&RigEvent::Warning("no_answer".into())));
}

#[test]
fn unplugged_cable_is_reported_not_a_panic() {
    let (mut r, h, _c) = rig(false);
    h.feed(&from_radio(&[0x00, 0x00, 0x40, 0x07, 0x14, 0x00]));
    r.poll();
    h.unplug();
    assert!(matches!(r.execute(&RadioCommand::SetFrequency(7_030_000)), Err(RigError::Io(_))));
    assert!(r.poll().contains(&RigEvent::Connected(false)));
}

#[test]
fn unsendable_text_is_refused_or_trimmed() {
    let (mut r, h, _c) = rig(false);
    assert!(r.execute(&RadioCommand::SendCw("!!!".into())).is_err());
    assert!(r.execute(&RadioCommand::SendCw("A".repeat(31))).is_err());
    assert!(r.execute(&RadioCommand::SendCw("CQ! DE".into())).is_ok());
    assert_eq!(cw_blocks(&written_frames(&h)), vec!["CQ DE".to_string()]);
}

#[test]
fn a_pasted_wall_of_text_is_refused() {
    let (mut r, h, _c) = rig(false);
    let wall = "CQ ".repeat(200);
    assert!(matches!(r.execute(&RadioCommand::SendCw(wall)), Err(RigError::InvalidArgument(m)) if m == "message_too_long"));
    assert!(h.take_written().is_empty());
}

#[test]
fn prosigns_are_translated_for_the_icom() {
    let (mut r, h, _c) = rig(false);
    r.execute(&RadioCommand::SendCw("73 <SK>".into())).unwrap();
    assert_eq!(cw_blocks(&written_frames(&h)), vec!["73 ^SK".to_string()]);
}

#[test]
fn guard_keeps_every_cw_byte_off_the_wire_outside_live() {
    for policy in [TxPolicy::ListenOnly, TxPolicy::Practice] {
        let (r, h, _c) = rig(false);
        let mut g = GuardedRig::new(r, PolicyHandle::new(policy));
        for cmd in RadioCommand::examples() {
            let _ = g.execute(&cmd);
        }
        let frames = written_frames(&h);
        assert!(cw_blocks(&frames).is_empty(), "{policy:?}");
        // No PTT frame (1C 00 01) either, and no raw bytes.
        assert!(!frames.iter().any(|f| f.body == vec![0x1C, 0x00, 0x01]), "{policy:?}");
        // The stop frame is allowed.
        assert!(frames.iter().any(|f| f.body == vec![0x17, 0xFF]), "stop must pass in {policy:?}");
    }
}

#[test]
fn live_lets_cw_through_the_guard() {
    let (r, h, _c) = rig(false);
    let mut g = GuardedRig::new(r, PolicyHandle::new(TxPolicy::Live));
    g.execute(&RadioCommand::SendCw("CQ".into())).unwrap();
    assert_eq!(cw_blocks(&written_frames(&h)), vec!["CQ".to_string()]);
}

#[test]
fn fast_poll_asks_for_swr_only_while_transmitting() {
    let (mut r, h, _c) = rig(false);
    r.fast_poll();
    let f = written_frames(&h);
    assert_eq!(f.iter().map(|x| x.body.clone()).collect::<Vec<_>>(), vec![vec![0x1C, 0x00]]);
    h.feed(&from_radio(&[0x1C, 0x00, 0x01]));
    r.fast_poll();
    written_frames(&h);
    r.fast_poll();
    let bodies: Vec<Vec<u8>> = written_frames(&h).iter().map(|x| x.body.clone()).collect();
    assert_eq!(bodies, vec![vec![0x1C, 0x00], vec![0x15, 0x12]]);
}

#[test]
fn swr_readings_are_reported_every_time() {
    let (mut r, h, _c) = rig(false);
    for _ in 0..3 {
        h.feed(&from_radio(&[0x15, 0x12, 0x00, 0x48])); // raw 48 is SWR 1.5
    }
    let ev = r.fast_poll();
    let n = ev.iter().filter(|e| matches!(e, RigEvent::Swr(s) if (*s - 1.5).abs() < 1e-3)).count();
    assert_eq!(n, 3);
}
