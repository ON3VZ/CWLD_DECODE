//! The byte pipe to a radio. Drivers only see [`Transport`], so they can be tested with
//! [`MockTransport`] and the exact bytes on the wire can be checked.

use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub trait Transport: Send {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()>;
    /// Returns whatever has arrived, without waiting long. An empty vector means nothing yet.
    fn read_available(&mut self) -> io::Result<Vec<u8>>;
}

/// A real serial port (a USB virtual COM port counts as one).
pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl SerialTransport {
    pub fn open(path: &str, baud: u32) -> io::Result<Self> {
        let port = serialport::new(path, baud)
            .timeout(Duration::from_millis(30))
            .open()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(SerialTransport { port })
    }
}

impl Transport for SerialTransport {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        use std::io::Write;
        self.port.write_all(bytes)?;
        self.port.flush()
    }

    fn read_available(&mut self) -> io::Result<Vec<u8>> {
        use std::io::Read;
        let n = self.port.bytes_to_read().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))? as usize;
        if n == 0 {
            return Ok(Vec::new());
        }
        let mut buf = vec![0u8; n];
        let got = self.port.read(&mut buf)?;
        buf.truncate(got);
        Ok(buf)
    }
}

/// Shared view on a [`MockTransport`], kept by the test.
#[derive(Clone, Default)]
pub struct MockHandle {
    written: Arc<Mutex<Vec<u8>>>,
    incoming: Arc<Mutex<VecDeque<u8>>>,
    broken: Arc<Mutex<bool>>,
}

impl MockHandle {
    /// Everything the driver wrote since the last call.
    pub fn take_written(&self) -> Vec<u8> {
        std::mem::take(&mut *self.written.lock().unwrap())
    }

    /// Everything written so far, as text (for line protocols).
    pub fn take_written_text(&self) -> String {
        String::from_utf8_lossy(&self.take_written()).to_string()
    }

    /// Bytes the "radio" sends to the driver.
    pub fn feed(&self, bytes: &[u8]) {
        self.incoming.lock().unwrap().extend(bytes.iter().copied());
    }

    pub fn feed_text(&self, text: &str) {
        self.feed(text.as_bytes());
    }

    /// Makes every following read and write fail, like a pulled USB cable.
    pub fn unplug(&self) {
        *self.broken.lock().unwrap() = true;
    }
}

pub struct MockTransport(MockHandle);

impl MockTransport {
    pub fn new() -> (MockTransport, MockHandle) {
        let h = MockHandle::default();
        (MockTransport(h.clone()), h)
    }
}

impl Transport for MockTransport {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        if *self.0.broken.lock().unwrap() {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "unplugged"));
        }
        self.0.written.lock().unwrap().extend_from_slice(bytes);
        Ok(())
    }

    fn read_available(&mut self) -> io::Result<Vec<u8>> {
        if *self.0.broken.lock().unwrap() {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "unplugged"));
        }
        Ok(self.0.incoming.lock().unwrap().drain(..).collect())
    }
}
