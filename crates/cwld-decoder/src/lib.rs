//! A probabilistic CW decoder that runs on the CPU and needs no training.
//!
//! It works on soft evidence per frame (see `cwld_dsp::Frontend`) and finds the most likely
//! sequence of key-down and key-up segments with a semi-Markov Viterbi search. Every segment
//! gets a label (dot, dash, gap inside a character, gap between characters, gap between
//! words) and a log-normal duration likelihood around the expected length. The speed is
//! found by trying candidate unit lengths and keeping the one that explains the data best.
//! Characters get a confidence from the other readings the same timing could have had.

pub mod bench;
pub mod decoder;
pub mod gate;
pub mod lang;
pub mod pipeline;
pub mod voice;

pub use decoder::{DecodeUpdate, Decoder, DecoderConfig, TentativeChar};
pub use pipeline::{Pipeline, PipelineConfig};

#[cfg(test)]
mod tests {
    use crate::bench::{decode_samples, levenshtein};
    use crate::PipelineConfig;
    use cwld_dsp::{generate, CwSpec};

    fn decode(spec: &CwSpec) -> (String, String) {
        let g = generate(spec);
        (g.truth.clone(), decode_samples(&g.samples, PipelineConfig::new(spec.sample_rate, spec.tone_hz)))
    }

    #[test]
    fn clean_machine_cw_is_decoded_exactly() {
        for wpm in [12.0, 20.0, 30.0] {
            let (truth, got) = decode(&CwSpec { wpm, text: "CQ CQ DE ON3VZ K".into(), ..Default::default() });
            assert_eq!(got, truth, "{wpm} WPM");
        }
    }

    #[test]
    fn moderate_noise_is_fine() {
        let (truth, got) = decode(&CwSpec { wpm: 20.0, snr_db: Some(12.0), jitter: 0.05, text: "UR RST 599 IN HOBOKEN".into(), ..Default::default() });
        assert!(levenshtein(&truth, &got) <= 2, "{truth:?} vs {got:?}");
    }

    #[test]
    fn a_steady_carrier_next_to_the_signal_is_notched_out() {
        let spec = CwSpec { wpm: 20.0, snr_db: Some(10.0), carrier: Some((50.0, 15.0)), lead_s: 3.0, text: "CQ CQ DE ON3VZ ON3VZ K".into(), ..Default::default() };
        let (truth, got) = decode(&spec);
        assert!(levenshtein(&truth, &got) <= 2, "{truth:?} vs {got:?}");
    }

    #[test]
    fn strong_clicks_are_blanked() {
        let spec = CwSpec { wpm: 20.0, snr_db: Some(10.0), impulses_per_s: 15.0, impulse_db: 46.0, text: "UR RST 599 IN HOBOKEN".into(), ..Default::default() };
        let (truth, got) = decode(&spec);
        assert!(levenshtein(&truth, &got) <= 1, "{truth:?} vs {got:?}");
    }

    #[test]
    fn silence_and_pure_noise_decode_to_nothing() {
        let g = generate(&CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 20.0, trail_s: 0.0, ..Default::default() });
        let got = decode_samples(&g.samples, PipelineConfig::new(8000, 700.0));
        assert!(got.chars().count() <= 3, "{got:?}");
    }
}
