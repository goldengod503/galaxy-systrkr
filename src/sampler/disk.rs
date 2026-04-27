use std::fs;
use std::time::Instant;

const SECTOR_SIZE: u64 = 512;

#[derive(Debug, Clone, Default)]
pub struct DiskSample {
    pub read_bps: u64,
    pub write_bps: u64,
}

pub struct DiskSampler {
    last: Option<(Instant, u64, u64)>,
}

impl DiskSampler {
    pub fn new() -> Self {
        Self { last: None }
    }

    pub fn tick(&mut self) -> DiskSample {
        let now = Instant::now();
        let (r, w) = read_diskstats_totals().unwrap_or((0, 0));
        match self.last.replace((now, r, w)) {
            Some((prev_t, prev_r, prev_w)) => {
                let dt = now.saturating_duration_since(prev_t).as_secs_f64().max(0.001);
                DiskSample {
                    read_bps: ((r.saturating_sub(prev_r) as f64) / dt) as u64,
                    write_bps: ((w.saturating_sub(prev_w) as f64) / dt) as u64,
                }
            }
            None => DiskSample::default(),
        }
    }
}

impl Default for DiskSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Read total bytes read/written across physical block devices from /proc/diskstats.
///
/// /proc/diskstats columns (fields after major/minor/name): 1 reads completed,
/// 2 reads merged, 3 sectors read, 4 ms reading, 5 writes completed, 6 writes
/// merged, 7 sectors written, ... Sectors are 512B by Linux convention.
///
/// Filters out partitions (e.g. sda1), loop, ram, and dm devices to avoid
/// double-counting and to ignore non-physical devices.
fn read_diskstats_totals() -> Option<(u64, u64)> {
    let contents = fs::read_to_string("/proc/diskstats").ok()?;
    let mut total_read_sectors: u64 = 0;
    let mut total_written_sectors: u64 = 0;

    for line in contents.lines() {
        let mut it = line.split_whitespace();
        let _major = it.next()?;
        let _minor = it.next()?;
        let name = it.next()?;
        if !is_physical_whole_device(name) {
            continue;
        }
        let fields: Vec<&str> = it.collect();
        if fields.len() < 7 {
            continue;
        }
        let sectors_read: u64 = fields[2].parse().unwrap_or(0);
        let sectors_written: u64 = fields[6].parse().unwrap_or(0);
        total_read_sectors = total_read_sectors.saturating_add(sectors_read);
        total_written_sectors = total_written_sectors.saturating_add(sectors_written);
    }

    Some((
        total_read_sectors.saturating_mul(SECTOR_SIZE),
        total_written_sectors.saturating_mul(SECTOR_SIZE),
    ))
}

fn is_physical_whole_device(name: &str) -> bool {
    if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("dm-") {
        return false;
    }
    // NVMe whole device: nvme0n1 (yes), nvme0n1p1 (no — partition).
    if let Some(rest) = name.strip_prefix("nvme") {
        return !rest.contains('p');
    }
    // sd*, hd*, vd*, xvd*: whole device has no trailing digits (sda yes, sda1 no).
    if name.starts_with("sd")
        || name.starts_with("hd")
        || name.starts_with("vd")
        || name.starts_with("xvd")
    {
        return !name.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false);
    }
    // mmcblk0 is whole; mmcblk0p1 is a partition.
    if name.starts_with("mmcblk") {
        return !name.contains('p');
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_tick_is_zero() {
        let mut s = DiskSampler::new();

        let first = s.tick();

        assert_eq!(first.read_bps, 0);
        assert_eq!(first.write_bps, 0);
    }

    #[test]
    fn classifies_whole_devices_only() {
        assert!(is_physical_whole_device("sda"));
        assert!(!is_physical_whole_device("sda1"));
        assert!(is_physical_whole_device("nvme0n1"));
        assert!(!is_physical_whole_device("nvme0n1p1"));
        assert!(!is_physical_whole_device("loop0"));
        assert!(!is_physical_whole_device("dm-0"));
        assert!(is_physical_whole_device("mmcblk0"));
        assert!(!is_physical_whole_device("mmcblk0p1"));
    }
}
