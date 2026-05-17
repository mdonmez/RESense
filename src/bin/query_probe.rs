#![allow(dead_code)]

mod error {
    pub type Result<T> = anyhow::Result<T>;
}

#[path = "../platform/mod.rs"]
mod platform;

use anyhow::Context;
use serde::Serialize;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct Snapshot {
    label: String,
    bk_hotkey_number: Option<u32>,
    queries: Vec<QueryResult>,
}

#[derive(Debug, Serialize)]
struct QueryResult {
    cmd: u16,
    query: u32,
    raw_hex: String,
    value_hex: String,
    status_byte: u8,
    value_u64: u64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let label = args
        .next()
        .context("usage: cargo run --bin query_probe -- <label> [output.json]")?;
    let output_path = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("target").join(format!("query_probe_{label}.json"))
    });

    let bk_hotkey_number =
        platform::registry::read_hklm_dword(platform::registry::NITROSENSE, "BK_Hotkey_Number")
            .ok();

    let mut queries = Vec::new();

    for query in query_set_for_cmd_10() {
        if let Ok(result) = probe(10, query) {
            queries.push(result);
        }
    }
    for query in query_set_for_cmd_12() {
        if let Ok(result) = probe(12, query) {
            queries.push(result);
        }
    }
    for query in query_set_for_cmd_20(bk_hotkey_number) {
        if let Ok(result) = probe(20, query) {
            queries.push(result);
        }
    }
    for query in query_set_for_cmd_34() {
        if let Ok(result) = probe(34, query) {
            queries.push(result);
        }
    }

    let snapshot = Snapshot {
        label,
        bk_hotkey_number,
        queries,
    };

    fs::write(&output_path, serde_json::to_string_pretty(&snapshot)?)
        .with_context(|| format!("writing {}", output_path.display()))?;
    println!("{}", output_path.display());
    Ok(())
}

fn probe(cmd: u16, query: u32) -> anyhow::Result<QueryResult> {
    let (raw, value) = platform::pipe::service_get_u64(cmd, &[platform::pipe::u32_arg(query)])?;
    Ok(QueryResult {
        cmd,
        query,
        raw_hex: hex(&raw),
        value_hex: format!("{value:016x}"),
        status_byte: (value & 0xFF) as u8,
        value_u64: value,
    })
}

fn query_set_for_cmd_10() -> BTreeSet<u32> {
    (0..=64).collect()
}

fn query_set_for_cmd_12() -> BTreeSet<u32> {
    let mut queries: BTreeSet<u32> = (0..=32).collect();
    queries.extend([64, 128, 255, 256, 512, 1024, 2048]);
    queries
}

fn query_set_for_cmd_20(bk_hotkey_number: Option<u32>) -> BTreeSet<u32> {
    let mut queries: BTreeSet<u32> = (0..=1024).collect();
    queries.extend([264, 519]);
    if let Some(bk) = bk_hotkey_number {
        queries.insert(1 | (bk << 8) | 0x0008_0000);
        queries.insert(1 | (bk << 8) | 0xFFFF_0000);
    }
    queries
}

fn query_set_for_cmd_34() -> BTreeSet<u32> {
    (0..=32).collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
