use std::path::PathBuf;
use std::process::Command;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures")
}

fn run(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_veritool"))
        .args(args)
        .output()
        .expect("failed to run veritool");
    assert!(
        out.status.success(),
        "veritool exited with error\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

// ── ports ──────────────────────────────────────────────────────────────────

#[test]
fn snap_counter_ports_text() {
    let f = fixtures_dir().join("counter.sv");
    insta::assert_snapshot!(run(&["ports", f.to_str().unwrap()]));
}

#[test]
fn snap_counter_ports_json() {
    let f = fixtures_dir().join("counter.sv");
    insta::assert_snapshot!(run(&["ports", "--format", "json", f.to_str().unwrap()]));
}

#[test]
fn snap_counter_ports_markdown() {
    let f = fixtures_dir().join("counter.sv");
    insta::assert_snapshot!(run(&["ports", "--format", "markdown", f.to_str().unwrap()]));
}

#[test]
fn snap_counter_ports_csv() {
    let f = fixtures_dir().join("counter.sv");
    insta::assert_snapshot!(run(&["ports", "--format", "csv", f.to_str().unwrap()]));
}

// ── signals ────────────────────────────────────────────────────────────────

#[test]
fn snap_fifo_signals_text() {
    let f = fixtures_dir().join("fifo_sync.sv");
    insta::assert_snapshot!(run(&["signals", "-m", "fifo_sync", f.to_str().unwrap()]));
}

#[test]
fn snap_fifo_signals_json() {
    let f = fixtures_dir().join("fifo_sync.sv");
    insta::assert_snapshot!(run(&[
        "signals",
        "-m",
        "fifo_sync",
        "--format",
        "json",
        f.to_str().unwrap(),
    ]));
}

// ── ff ─────────────────────────────────────────────────────────────────────

#[test]
fn snap_counter_ff_module_json() {
    let f = fixtures_dir().join("counter.sv");
    insta::assert_snapshot!(run(&[
        "ff",
        "-m",
        "counter",
        "--format",
        "json",
        f.to_str().unwrap(),
    ]));
}

#[test]
fn snap_fifo_ff_hierarchy_text() {
    let f = fixtures_dir().join("fifo_sync.sv");
    insta::assert_snapshot!(run(&["ff", f.to_str().unwrap()]));
}

#[test]
fn snap_fifo_ff_hierarchy_json() {
    let f = fixtures_dir().join("fifo_sync.sv");
    insta::assert_snapshot!(run(&["ff", "--format", "json", f.to_str().unwrap()]));
}

// ── hier ───────────────────────────────────────────────────────────────────

#[test]
fn snap_top_with_subs_hier_text() {
    let f = fixtures_dir().join("top_with_subs.sv");
    insta::assert_snapshot!(run(&["hier", "--top", "top1", f.to_str().unwrap()]));
}

#[test]
fn snap_top_with_subs_hier_json() {
    let f = fixtures_dir().join("top_with_subs.sv");
    insta::assert_snapshot!(run(&[
        "hier",
        "--top",
        "top1",
        "--format",
        "json",
        f.to_str().unwrap(),
    ]));
}

#[test]
fn snap_top_with_subs_hier_markdown() {
    let f = fixtures_dir().join("top_with_subs.sv");
    insta::assert_snapshot!(run(&[
        "hier",
        "--top",
        "top1",
        "--format",
        "markdown",
        f.to_str().unwrap(),
    ]));
}

// ── top ────────────────────────────────────────────────────────────────────

#[test]
fn snap_top_with_subs_top_text() {
    let f = fixtures_dir().join("top_with_subs.sv");
    insta::assert_snapshot!(run(&["top", f.to_str().unwrap()]));
}

#[test]
fn snap_top_with_subs_top_json() {
    let f = fixtures_dir().join("top_with_subs.sv");
    insta::assert_snapshot!(run(&["top", "--format", "json", f.to_str().unwrap()]));
}
