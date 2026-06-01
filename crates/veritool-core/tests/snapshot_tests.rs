use std::path::PathBuf;
use veritool_core::loader;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures")
}

/// Snapshot: counter module — params, ports, ff_decls (path/span redacted).
#[test]
fn snap_counter_module() {
    let design = loader::parse_sv_files(&[fixtures_dir().join("counter.sv")], &[], &[]).unwrap();
    let m = design.modules.get("counter").unwrap();
    insta::assert_json_snapshot!(m, {
        ".file" => "[file]",
        ".span" => "[span]",
    });
}

/// Snapshot: fifo_sync module — localparam + mem signal (path/span redacted).
#[test]
fn snap_fifo_module() {
    let design = loader::parse_sv_files(&[fixtures_dir().join("fifo_sync.sv")], &[], &[]).unwrap();
    let m = design.modules.get("fifo_sync").unwrap();
    insta::assert_json_snapshot!(m, {
        ".file" => "[file]",
        ".span" => "[span]",
    });
}

/// Snapshot: top_with_subs — hierarchy instances.
#[test]
fn snap_top_with_subs_instances() {
    let design =
        loader::parse_sv_files(&[fixtures_dir().join("top_with_subs.sv")], &[], &[]).unwrap();
    let top = design.modules.get("top1").unwrap();
    insta::assert_json_snapshot!(&top.instances);
}
