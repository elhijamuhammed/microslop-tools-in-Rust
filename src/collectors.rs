use std::cmp::Ordering;
use std::time::{Duration, Instant};
use sysinfo::ProcessesToUpdate;
use crate::models::{App, ProcRow, Tool};

pub fn init_app() -> App {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all(); // first sample

    let mut nets = sysinfo::Networks::new();
    nets.refresh(true); // initial list + counters

    let mut app = App {
        selected_tool: Tool::Processes,
        processes: Vec::new(),
        error: None,
        sys,
        nets,
        prev_proc_disk: std::collections::HashMap::new(),
        prev_net: (0, 0),
        net_rx_bps: 0.0,
        net_tx_bps: 0.0,
        last_refresh: Instant::now(),
        auto_refresh: true,
        refresh_ms: 1500,
    };

    refresh_everything(&mut app);
    app
}

pub fn refresh_everything(app: &mut App) {
    let now = Instant::now();
    let dt = now
        .checked_duration_since(app.last_refresh)
        .unwrap_or(Duration::from_millis(app.refresh_ms.max(250)))
        .as_secs_f64()
        .max(0.25);

    refresh_network(app, dt);
    refresh_processes(app, dt);

    app.last_refresh = now;
}

pub fn refresh_network(app: &mut App, dt: f64) {
    app.nets.refresh(false);

    let mut rx_total: u64 = 0;
    let mut tx_total: u64 = 0;

    for (_name, data) in app.nets.iter() {
        rx_total = rx_total.saturating_add(data.received());
        tx_total = tx_total.saturating_add(data.transmitted());
    }

    let (prev_rx, prev_tx) = app.prev_net;
    let rx_delta = rx_total.saturating_sub(prev_rx) as f64;
    let tx_delta = tx_total.saturating_sub(prev_tx) as f64;

    app.net_rx_bps = rx_delta / dt;
    app.net_tx_bps = tx_delta / dt;

    app.prev_net = (rx_total, tx_total);
}

pub fn refresh_processes(app: &mut App, dt: f64) {
    app.error = None;

    app.sys.refresh_processes(ProcessesToUpdate::All, true);

    // NEW: collect PIDs that own a real top-level window (Windows), or empty set otherwise
    let window_pids = pids_with_top_level_windows();

    let mut rows: Vec<ProcRow> = Vec::with_capacity(app.sys.processes().len());

    for (pid, p) in app.sys.processes().iter() {
        let pid_i32 = pid.as_u32() as i32;

        let cpu = p.cpu_usage();
        let memory_bytes = p.memory();

        let du = p.disk_usage();
        let read_bytes = du.read_bytes;
        let written_bytes = du.written_bytes;

        let (prev_r, prev_w) = app
            .prev_proc_disk
            .get(&pid_i32)
            .copied()
            .unwrap_or((read_bytes, written_bytes));

        let r_delta = read_bytes.saturating_sub(prev_r) as f64;
        let w_delta = written_bytes.saturating_sub(prev_w) as f64;

        let read_bps = r_delta / dt;
        let write_bps = w_delta / dt;

        app.prev_proc_disk.insert(pid_i32, (read_bytes, written_bytes));

        rows.push(ProcRow {
            pid: pid_i32,
            name: p.name().to_string_lossy().to_string(),
            cpu,
            memory_bytes,
            read_bps,
            write_bps,
            has_window: window_pids.contains(&pid_i32), // <-- NEW
        });
    }

    rows.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(Ordering::Equal));
    app.processes = rows;
}

#[cfg(windows)]
fn pids_with_top_level_windows() -> std::collections::HashSet<i32> {
    use std::collections::HashSet;

    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible,
        GW_OWNER,
    };

    unsafe extern "system" fn enum_windows_proc( hwnd: HWND, lparam: LPARAM, ) -> windows::core::BOOL {
        let set = &mut *(lparam.0 as *mut std::collections::HashSet<i32>);

        if !IsWindowVisible(hwnd).as_bool() {
            return windows::core::BOOL(1);
        }

        let owner = GetWindow(hwnd, GW_OWNER).unwrap_or(HWND(std::ptr::null_mut()));
        if owner != HWND(std::ptr::null_mut()) {
            return windows::core::BOOL(1);
        }

        if GetWindowTextLengthW(hwnd) == 0 {
            return windows::core::BOOL(1);
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid != 0 {
            set.insert(pid as i32);
        }

        windows::core::BOOL(1)
    }

    let mut set: HashSet<i32> = HashSet::new();
    unsafe {
        let lparam = LPARAM((&mut set as *mut HashSet<i32>) as isize);
        let _ = EnumWindows(Some(enum_windows_proc), lparam);
    }
    set
}

#[cfg(not(windows))]
fn pids_with_top_level_windows() -> std::collections::HashSet<i32> {
    std::collections::HashSet::new()
}

#[cfg(not(windows))]
fn pids_with_top_level_windows() -> std::collections::HashSet<i32> {
    std::collections::HashSet::new()
}

#[cfg(not(windows))]
fn pids_with_top_level_windows() -> std::collections::HashSet<i32> {
    // Non-Windows stub (you can implement X11/macOS later)
    std::collections::HashSet::new()
}
