pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let units = ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut value = bytes as f64;
    let mut unit = "B";

    for u in units {
        value /= 1024.0;
        unit = u;
        if value < 1024.0 {
            break;
        }
    }

    format!("{value:.2} {unit}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_small_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(12), "12 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn formats_kib_mib() {
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(1536), "1.50 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MiB");
    }
}
