//! Finding serial ports, so the user does not have to know port names.

use cwld_types::RadioKind;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PortInfo {
    pub name: String,
    pub description: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    /// A guess at the radio behind the port. Only a hint, the user decides.
    pub hint: Option<RadioKind>,
}

/// USB vendor id of Icom Inc. **[VERIFIEER]** on the real radio.
pub const ICOM_VID: u16 = 0x0C26;

pub fn hint_for(vid: Option<u16>) -> Option<RadioKind> {
    match vid {
        Some(ICOM_VID) => Some(RadioKind::Icom),
        _ => None,
    }
}

pub fn list_ports() -> Vec<PortInfo> {
    let Ok(ports) = serialport::available_ports() else { return Vec::new() };
    let mut out: Vec<PortInfo> = ports
        .into_iter()
        .map(|p| {
            let (description, vid, pid) = match &p.port_type {
                serialport::SerialPortType::UsbPort(u) => (
                    u.product.clone().unwrap_or_else(|| "USB serial port".to_string()),
                    Some(u.vid),
                    Some(u.pid),
                ),
                serialport::SerialPortType::BluetoothPort => ("Bluetooth".to_string(), None, None),
                serialport::SerialPortType::PciPort => ("PCI".to_string(), None, None),
                serialport::SerialPortType::Unknown => (String::new(), None, None),
            };
            PortInfo { name: p.port_name, description, vid, pid, hint: hint_for(vid) }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icom_vendor_is_hinted_others_are_not() {
        assert_eq!(hint_for(Some(ICOM_VID)), Some(RadioKind::Icom));
        assert_eq!(hint_for(Some(0x1234)), None);
        assert_eq!(hint_for(None), None);
    }
}
