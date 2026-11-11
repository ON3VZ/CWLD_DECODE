//! Just enough of the Telnet protocol for a DX cluster: the option negotiation bytes are
//! removed from the text and every offer is declined, so the server stops asking.

const IAC: u8 = 255;
const DONT: u8 = 254;
const DO: u8 = 253;
const WONT: u8 = 252;
const WILL: u8 = 251;
const SB: u8 = 250;
const SE: u8 = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Text,
    Iac,
    Verb(u8),
    Sub,
    SubIac,
}

/// Filters a byte stream. Text goes to `text`, answers to the server go to `reply`.
#[derive(Debug)]
pub struct Telnet {
    state: State,
}

impl Default for Telnet {
    fn default() -> Self {
        Telnet { state: State::Text }
    }
}

impl Telnet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, input: &[u8], text: &mut Vec<u8>, reply: &mut Vec<u8>) {
        for &b in input {
            self.state = match self.state {
                State::Text => {
                    if b == IAC {
                        State::Iac
                    } else {
                        text.push(b);
                        State::Text
                    }
                }
                State::Iac => match b {
                    IAC => {
                        // An escaped 255 is a real data byte.
                        text.push(IAC);
                        State::Text
                    }
                    DO | DONT | WILL | WONT => State::Verb(b),
                    SB => State::Sub,
                    _ => State::Text,
                },
                State::Verb(v) => {
                    // Decline everything that is offered or asked. Answers to DONT and WONT would
                    // only start a loop, so those are left unanswered.
                    match v {
                        DO => reply.extend_from_slice(&[IAC, WONT, b]),
                        WILL => reply.extend_from_slice(&[IAC, DONT, b]),
                        _ => {}
                    }
                    State::Text
                }
                State::Sub => {
                    if b == IAC {
                        State::SubIac
                    } else {
                        State::Sub
                    }
                }
                State::SubIac => {
                    if b == SE {
                        State::Text
                    } else {
                        State::Sub
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiation_is_removed_and_declined() {
        let mut t = Telnet::new();
        let (mut text, mut reply) = (Vec::new(), Vec::new());
        t.feed(&[IAC, DO, 1, b'h', b'i', IAC, WILL, 3, b'!'], &mut text, &mut reply);
        assert_eq!(text, b"hi!");
        assert_eq!(reply, vec![IAC, WONT, 1, IAC, DONT, 3]);
    }

    #[test]
    fn dont_and_wont_get_no_answer() {
        let mut t = Telnet::new();
        let (mut text, mut reply) = (Vec::new(), Vec::new());
        t.feed(&[IAC, DONT, 1, IAC, WONT, 3, b'x'], &mut text, &mut reply);
        assert_eq!(text, b"x");
        assert!(reply.is_empty());
    }

    #[test]
    fn sequences_may_be_split_over_reads() {
        let mut t = Telnet::new();
        let (mut text, mut reply) = (Vec::new(), Vec::new());
        t.feed(&[b'a', IAC], &mut text, &mut reply);
        t.feed(&[DO], &mut text, &mut reply);
        t.feed(&[24, b'b'], &mut text, &mut reply);
        assert_eq!(text, b"ab");
        assert_eq!(reply, vec![IAC, WONT, 24]);
    }

    #[test]
    fn subnegotiation_and_escaped_255() {
        let mut t = Telnet::new();
        let (mut text, mut reply) = (Vec::new(), Vec::new());
        t.feed(&[IAC, SB, 24, 1, IAC, SE, b'o', b'k', IAC, IAC], &mut text, &mut reply);
        assert_eq!(text, vec![b'o', b'k', 255]);
        assert!(reply.is_empty());
    }
}
