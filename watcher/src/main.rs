//! synapse-watcher: per-device Rust client of the synapse "brain".
//!
//! Watches the given directories recursively and reports every filesystem
//! event as a JSON line — POSTed to the hub (`SYNAPSE_HUB_URL`) when set,
//! printed to stdout otherwise. The hub aggregates these into device state
//! and decides what reaches Feishu.

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct DeviceEvent {
    device: String,
    kind: String,
    paths: Vec<String>,
    ts: u64,
}

fn device_name() -> String {
    std::env::var("SYNAPSE_DEVICE")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".into())
}

fn report(ev: &DeviceEvent, hub: Option<&str>) {
    let body = serde_json::to_string(ev).expect("DeviceEvent is serializable");
    match hub {
        Some(url) => {
            // Fire-and-forget: the hub may be down; the watcher must not die with it.
            if let Err(e) = ureq::post(url)
                .set("content-type", "application/json")
                .send_string(&body)
            {
                eprintln!("hub post failed ({e}); event: {body}");
            }
        }
        None => println!("{body}"),
    }
}

fn main() -> notify::Result<()> {
    let roots: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    let roots = if roots.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        roots
    };
    let hub = std::env::var("SYNAPSE_HUB_URL").ok().filter(|u| !u.is_empty());
    let device = device_name();

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default(),
    )?;
    for root in &roots {
        watcher.watch(root, RecursiveMode::Recursive)?;
        eprintln!("watching {}", root.display());
    }

    for res in rx {
        match res {
            Ok(Event { kind, paths, .. }) => report(
                &DeviceEvent {
                    device: device.clone(),
                    kind: format!("{kind:?}"),
                    paths: paths.iter().map(|p| p.display().to_string()).collect(),
                    ts: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                },
                hub.as_deref(),
            ),
            Err(e) => eprintln!("watch error: {e}"),
        }
    }
    Ok(())
}
