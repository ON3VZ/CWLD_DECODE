/// ADIF band name for a frequency in Hz. Empty when it is outside the amateur bands.
pub fn band_of(hz: u64) -> &'static str {
    let khz = hz / 1000;
    match khz {
        1800..=2000 => "160m",
        3500..=4000 => "80m",
        5250..=5450 => "60m",
        7000..=7300 => "40m",
        10100..=10150 => "30m",
        14000..=14350 => "20m",
        18068..=18168 => "17m",
        21000..=21450 => "15m",
        24890..=24990 => "12m",
        28000..=29700 => "10m",
        50000..=54000 => "6m",
        144000..=148000 => "2m",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bands() {
        assert_eq!(band_of(7_030_000), "40m");
        assert_eq!(band_of(14_025_000), "20m");
        assert_eq!(band_of(3_550_000), "80m");
        assert_eq!(band_of(1_830_000), "160m");
        assert_eq!(band_of(28_020_000), "10m");
        assert_eq!(band_of(9_000_000), "");
    }
}
