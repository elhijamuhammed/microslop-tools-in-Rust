pub fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

pub fn bps_to_mbps(bps: f64) -> f64 {
    bps / (1024.0 * 1024.0)
}
