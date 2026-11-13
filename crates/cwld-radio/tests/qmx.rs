use cwld_radio::kenwood::is_forbidden;
use cwld_radio::{GuardedRig, ManualClock, MockHandle, MockTransport, QmxRig, Rig, RigError};
use cwld_types::{PolicyHandle, RadioCommand, RadioMode, RigEvent, TxPolicy};

fn rig() -> (QmxRig<MockTransport>, MockHandle, ManualClock) {
    let (t, h) = MockTransport::new();
    let clock = ManualClock::default();
    (QmxRig::new(t, Box::new(clock.clone())), h, clock)
}

/// All `KY ...;` commands that carry text, in order.
fn ky_texts(written: &str) -> Vec<String> {
    written
        .split(';')
        .filter_map(|c| c.strip_prefix("KY "))
        .map(|t| t.to_string())
        .collect()
}

#[test]
fn first_poll_identifies_asks_for_pushes_and_reads_status() {
    let (mut r, h, _c) = rig();
    r.poll();
    let w = h.take_written_text();
    assert!(w.starts_with("ID;AI2;"), "{w}");
    assert!(w.contains("FA;MD;TQ;KS;KY;SP;"), "{w}");
    assert!(w.contains("TB;"), "{w}");
    // Identification is asked once.
    r.poll();
    assert!(!h.take_written_text().contains("ID;"));
}

#[test]
fn status_answers_become_events() {
    let (mut r, h, _c) = rig();
    h.feed_text("FA00007030000;MD7;TQ1;KS028;");
    let ev = r.poll();
    assert!(ev.contains(&RigEvent::Connected(true)));
    assert!(ev.contains(&RigEvent::Frequency(7_030_000)));
    assert!(ev.contains(&RigEvent::Mode(RadioMode::CwReverse)));
    assert!(ev.contains(&RigEvent::Tx(true)));
    assert!(ev.contains(&RigEvent::CwSpeed(28)));
}

#[test]
fn an_if_push_updates_the_frequency_only() {
    let (mut r, h, _c) = rig();
    h.feed_text("IF00014030000      +00000000002000000;");
    let ev = r.poll();
    assert!(ev.contains(&RigEvent::Frequency(14_030_000)));
    assert!(!ev.iter().any(|e| matches!(e, RigEvent::Tx(_) | RigEvent::Mode(_))));
}

#[test]
fn answers_split_over_two_reads_are_put_together() {
    let (mut r, h, _c) = rig();
    h.feed_text("FA000070");
    assert!(!r.poll().iter().any(|e| matches!(e, RigEvent::Frequency(_))));
    h.feed_text("30000;");
    assert!(r.poll().contains(&RigEvent::Frequency(7_030_000)));
}

#[test]
fn set_commands_have_the_documented_form() {
    let (mut r, h, _c) = rig();
    r.execute(&RadioCommand::SetFrequency(7_030_000)).unwrap();
    r.execute(&RadioCommand::SetMode(RadioMode::Cw)).unwrap();
    r.execute(&RadioCommand::SetCwSpeed(22)).unwrap();
    assert_eq!(h.take_written_text(), "FA00007030000;MD3;KS22;");
}

#[test]
fn speed_zero_and_forbidden_commands_never_reach_the_radio() {
    let (mut r, h, _c) = rig();
    assert!(matches!(r.execute(&RadioCommand::SetCwSpeed(0)), Err(RigError::InvalidArgument(_))));
    assert!(r.execute(&RadioCommand::Raw(b"PS0;".to_vec())).is_err());
    assert!(r.execute(&RadioCommand::Raw(b"KS00;".to_vec())).is_err());
    assert_eq!(h.take_written_text(), "");
    assert!(is_forbidden("PS0;") && is_forbidden("KS00;") && !is_forbidden("KS22;"));
}

#[test]
fn text_waits_for_the_buffer_state_and_then_goes_out_per_word() {
    let (mut r, h, _c) = rig();
    r.execute(&RadioCommand::SendCw("cq de on3vz".into())).unwrap();
    // Only the question about the buffer, no text yet.
    assert_eq!(h.take_written_text(), "KY;");
    h.feed_text("KY2;");
    r.poll();
    let texts = ky_texts(&h.take_written_text());
    assert_eq!(texts, vec!["CQ ", "DE ", "ON3VZ "]);
}

#[test]
fn nothing_is_added_while_the_buffer_is_over_75_percent_full() {
    let (mut r, h, _c) = rig();
    r.execute(&RadioCommand::SendCw("CQ DE ON3VZ".into())).unwrap();
    h.take_written_text();
    h.feed_text("KY1;");
    r.poll();
    assert!(ky_texts(&h.take_written_text()).is_empty());
}

#[test]
fn with_buffer_under_75_percent_only_small_words_are_added() {
    let (mut r, h, _c) = rig();
    r.execute(&RadioCommand::SendCw("CQ ".repeat(20))).unwrap();
    h.take_written_text();
    h.feed_text("KY0;");
    r.poll();
    let sent: usize = ky_texts(&h.take_written_text()).iter().map(|t| t.len()).sum();
    // At most 60 characters can already be in the buffer, so at most 20 more are added.
    assert!(sent <= 20, "{sent}");
    assert!(sent > 0);
}

#[test]
fn a_long_message_never_overflows_the_80_character_buffer() {
    let (mut r, h, c) = rig();
    let text = "CQ CQ CQ DE ON3VZ ON3VZ ON3VZ PSE K ".repeat(3);
    h.feed_text("KS040;"); // fast, so the message stays under the one minute limit
    r.poll();
    h.take_written_text();
    r.execute(&RadioCommand::SendCw(text.clone())).unwrap();
    let mut all = ky_texts(&h.take_written_text());
    let mut in_buffer = 0usize; // worst-case model of the radio buffer
    for _ in 0..120 {
        h.feed_text("KY2;");
        c.advance(500);
        r.poll();
        let batch = ky_texts(&h.take_written_text());
        in_buffer = batch.iter().map(|t| t.len()).sum::<usize>();
        assert!(in_buffer <= 80, "batch of {in_buffer}");
        all.extend(batch);
    }
    let _ = in_buffer;
    assert_eq!(all.concat().trim_end(), text.trim().replace("  ", " "));
}

#[test]
fn a_word_that_cannot_fit_is_refused() {
    let (mut r, _h, _c) = rig();
    assert!(r.execute(&RadioCommand::SendCw("A".repeat(80))).is_err());
}

#[test]
fn prosigns_use_the_qmx_characters() {
    let (mut r, h, _c) = rig();
    r.execute(&RadioCommand::SendCw("73 <SK>".into())).unwrap();
    h.take_written_text();
    h.feed_text("KY2;");
    r.poll();
    assert_eq!(ky_texts(&h.take_written_text()), vec!["73 ", "> "]);
}

#[test]
fn stop_empties_the_queue_and_sends_24_spaces() {
    let (mut r, h, _c) = rig();
    r.execute(&RadioCommand::SendCw("CQ DE ON3VZ".into())).unwrap();
    h.take_written_text();
    r.execute(&RadioCommand::StopCw).unwrap();
    let w = h.take_written_text();
    assert_eq!(w, format!("KY {};", " ".repeat(24)));
    h.feed_text("KY2;");
    r.poll();
    assert!(ky_texts(&h.take_written_text()).iter().all(|t| t.trim().is_empty()), "no text after stop");
}

#[test]
fn paddle_text_comes_from_the_radio_decoder() {
    let (mut r, h, _c) = rig();
    h.feed_text("TB005R R T;");
    assert!(r.poll().contains(&RigEvent::PaddleText("R R T".into())));
    // An empty buffer gives nothing.
    h.feed_text("TB000;");
    assert!(!r.poll().iter().any(|e| matches!(e, RigEvent::PaddleText(_))));
}

#[test]
fn the_decoder_echo_of_our_own_text_is_not_paddle() {
    let (mut r, h, c) = rig();
    r.execute(&RadioCommand::SendCw("CQ DE ON3VZ".into())).unwrap();
    h.feed_text("KY2;");
    r.poll();
    // The radio decodes what it is sending.
    h.feed_text("TB005CQ DE;");
    c.advance(500);
    assert!(!r.poll().iter().any(|e| matches!(e, RigEvent::PaddleText(_))));
    // Much later, with an empty queue, the paddle counts again.
    c.advance(60_000);
    h.feed_text("TB002AR;");
    assert!(r.poll().contains(&RigEvent::PaddleText("AR".into())));
}

#[test]
fn an_unknown_command_answer_is_a_warning() {
    let (mut r, h, _c) = rig();
    h.feed_text("?;");
    assert!(r.poll().contains(&RigEvent::Warning("qmx_rejected".into())));
}

#[test]
fn a_radio_that_is_not_a_qmx_is_noticed() {
    let (mut r, h, _c) = rig();
    h.feed_text("ID021;");
    assert!(r.poll().contains(&RigEvent::Warning("not_a_qmx".into())));
}

#[test]
fn unplugged_cable_is_reported() {
    let (mut r, h, _c) = rig();
    h.feed_text("FA00007030000;");
    r.poll();
    h.unplug();
    assert!(matches!(r.execute(&RadioCommand::SetFrequency(7_030_000)), Err(RigError::Io(_))));
    assert!(r.poll().contains(&RigEvent::Connected(false)));
}

#[test]
fn guard_keeps_all_text_and_transmit_commands_off_the_wire_outside_live() {
    for policy in [TxPolicy::ListenOnly, TxPolicy::Practice] {
        let (r, h, _c) = rig();
        let mut g = GuardedRig::new(r, PolicyHandle::new(policy));
        for cmd in RadioCommand::examples() {
            let _ = g.execute(&cmd);
        }
        let w = h.take_written_text();
        assert!(!w.contains("TQ1"), "{policy:?}: {w}");
        assert!(!w.contains("KD1"), "{policy:?}: {w}");
        assert!(ky_texts(&w).iter().all(|t| t.trim().is_empty()), "{policy:?}: {w}");
    }
}

#[test]
fn swr_is_asked_only_while_transmitting() {
    let (mut r, h, _c) = rig();
    r.fast_poll();
    let w = h.take_written_text();
    assert!(w.contains("TQ;") && !w.contains("SW;"), "{w}");
    h.feed_text("TQ1;");
    r.fast_poll();
    h.take_written_text();
    r.fast_poll();
    assert!(h.take_written_text().contains("SW;"));
}

#[test]
fn swr_answer_is_in_hundredths() {
    let (mut r, h, _c) = rig();
    h.feed_text("SW121;");
    assert!(r.fast_poll().contains(&RigEvent::Swr(1.21)));
    h.feed_text("SW300;");
    assert!(r.fast_poll().contains(&RigEvent::Swr(3.0)));
}

#[test]
fn empty_or_nonsense_swr_answer_is_no_reading() {
    let (mut r, h, _c) = rig();
    h.feed_text("SW;SW050;SWabc;");
    let ev = r.fast_poll();
    assert!(!ev.iter().any(|e| matches!(e, RigEvent::Swr(_))), "{ev:?}");
}

#[test]
fn qmx_reports_swr_capability() {
    let (r, _h, _c) = rig();
    assert!(r.capabilities().swr_meter);
}

#[test]
fn split_uses_vfo_b_and_sp() {
    let (mut r, h, _c) = rig();
    h.feed_text("FA00007025000;");
    r.poll();
    let _ = h.take_written_text();
    r.execute(&RadioCommand::SetSplit(7_026_000)).unwrap();
    assert_eq!(h.take_written_text(), "FB00007026000;SP1;");
    assert_eq!(r.status().split, cwld_types::SplitState { on: true, tx_hz: Some(7_026_000) });
    // An IF push with the transmit frequency while sending is not a retune.
    h.feed_text("IF00007026000     +00000000001000000 ;");
    assert!(!r.poll().iter().any(|e| matches!(e, RigEvent::Frequency(_))));
    let _ = h.take_written_text();
    // Split switched off on the radio is read back.
    h.feed_text("SP0;");
    assert!(r.poll().contains(&RigEvent::Split(cwld_types::SplitState::default())));
    let _ = h.take_written_text();
    r.execute(&RadioCommand::SetSplit(7_027_000)).unwrap();
    r.execute(&RadioCommand::SplitOff).unwrap();
    assert_eq!(h.take_written_text(), "FB00007027000;SP1;SP0;");
    assert!(!r.status().split.on);
}
