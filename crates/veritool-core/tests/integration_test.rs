use std::path::PathBuf;
use veritool_core::{loader, design::Direction, ParamEnv};
use veritool_core::width::calculate_width_with_params;
use veritool_core::design::Signal;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("tests/fixtures")
}

#[test]
fn test_counter_ports() {
    let file = fixtures_dir().join("counter.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let counter = design.modules.get("counter").expect("counter module not found");

    assert_eq!(counter.ports.len(), 4, "counter should have 4 ports");

    let port_names: Vec<&str> = counter.ports.iter().map(|p| p.name.as_str()).collect();
    assert!(port_names.contains(&"clk"), "should have clk port");
    assert!(port_names.contains(&"rst"), "should have rst port");
    assert!(port_names.contains(&"en"),  "should have en port");
    assert!(port_names.contains(&"q"),   "should have q port");

    let clk = counter.ports.iter().find(|p| p.name == "clk").unwrap();
    assert_eq!(clk.direction, Direction::Input);
    assert!(clk.packed_width.is_none(), "clk should be 1-bit (no packed dim)");

    let q = counter.ports.iter().find(|p| p.name == "q").unwrap();
    assert_eq!(q.direction, Direction::Output);
    assert!(q.packed_width.is_some(), "q should have packed dimension");
}

#[test]
fn test_counter_ff_decls() {
    let file = fixtures_dir().join("counter.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let counter = design.modules.get("counter").unwrap();

    assert!(!counter.ff_decls.is_empty(), "counter should have FF declarations");
    assert!(
        counter.ff_decls.iter().any(|ff| ff.signal_name == "q"),
        "q should be an FF"
    );
}

#[test]
fn test_top_module_detection() {
    let file = fixtures_dir().join("top_with_subs.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();

    let referenced: std::collections::HashSet<&str> = design
        .modules
        .values()
        .flat_map(|m| m.instances.iter().map(|i| i.module_ref.as_str()))
        .collect();

    let tops: Vec<&str> = design
        .modules
        .keys()
        .filter(|n| !referenced.contains(n.as_str()))
        .map(|n| n.as_str())
        .collect();

    assert_eq!(tops, vec!["top1"], "top1 should be the only top module");
}

#[test]
fn test_hierarchy_instances() {
    let file = fixtures_dir().join("top_with_subs.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let top = design.modules.get("top1").expect("top1 not found");

    assert_eq!(top.instances.len(), 2, "top1 should have 2 instances");

    let refs: Vec<&str> = top.instances.iter().map(|i| i.module_ref.as_str()).collect();
    assert!(refs.contains(&"alu"));
    assert!(refs.contains(&"reg_file"));
}

#[test]
fn test_fifo_signals() {
    let file = fixtures_dir().join("fifo_sync.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let fifo = design.modules.get("fifo_sync").expect("fifo_sync not found");

    let sig_names: Vec<&str> = fifo.signals.iter().map(|s| s.name.as_str()).collect();
    assert!(sig_names.contains(&"mem"),    "should have mem signal");
    assert!(sig_names.contains(&"wr_ptr"), "should have wr_ptr signal");
    assert!(sig_names.contains(&"rd_ptr"), "should have rd_ptr signal");
}

#[test]
fn test_fifo_ff_decls() {
    let file = fixtures_dir().join("fifo_sync.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let fifo = design.modules.get("fifo_sync").unwrap();

    let ff_names: Vec<&str> = fifo.ff_decls.iter().map(|f| f.signal_name.as_str()).collect();
    assert!(ff_names.contains(&"wr_ptr"), "wr_ptr should be an FF");
    assert!(ff_names.contains(&"rd_ptr"), "rd_ptr should be an FF");
}

#[test]
fn test_counter_param_evaluation() {
    let file = fixtures_dir().join("counter.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let counter = design.modules.get("counter").unwrap();

    // Default: WIDTH=8
    let env = ParamEnv::from_module(counter);
    assert_eq!(env.get("WIDTH"), Some(8), "WIDTH default should be 8");

    // q port: [WIDTH-1:0]
    let q = counter.ports.iter().find(|p| p.name == "q").unwrap();
    let sig = Signal {
        name: q.name.clone(),
        net_kind: q.net_kind.clone(),
        data_type: q.data_type.clone(),
        packed_width: q.packed_width.clone(),
        unpacked_dims: q.unpacked_dims.clone(),
    };
    let width = calculate_width_with_params(&sig, &env);
    assert_eq!(width, 8, "counter q should be 8 bits wide with WIDTH=8");

    // Override: WIDTH=16
    let env16 = ParamEnv::from_module(counter).with_overrides(&[("WIDTH".to_string(), 16)]);
    let width16 = calculate_width_with_params(&sig, &env16);
    assert_eq!(width16, 16, "counter q should be 16 bits wide with WIDTH=16");
}

#[test]
fn test_fifo_localparam_evaluation() {
    let file = fixtures_dir().join("fifo_sync.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let fifo = design.modules.get("fifo_sync").unwrap();

    let env = ParamEnv::from_module(fifo);
    // Default: WIDTH=8, DEPTH=16 → ADDR_W=$clog2(16)=4
    assert_eq!(env.get("WIDTH"), Some(8));
    assert_eq!(env.get("DEPTH"), Some(16));
    assert_eq!(env.get("ADDR_W"), Some(4), "ADDR_W should be $clog2(16)=4");

    // wr_ptr: [ADDR_W:0] = [4:0] = 5 bits
    let wr_ptr = fifo.signals.iter().find(|s| s.name == "wr_ptr").unwrap();
    let width = calculate_width_with_params(wr_ptr, &env);
    assert_eq!(width, 5, "wr_ptr should be 5 bits ([ADDR_W:0] with ADDR_W=4)");

    // mem: [WIDTH-1:0] [0:DEPTH-1] = 8 * 16 = 128 bits
    let mem = fifo.signals.iter().find(|s| s.name == "mem").unwrap();
    let mem_width = calculate_width_with_params(mem, &env);
    assert_eq!(mem_width, 128, "mem should be 128 bits (8 * 16)");
}

// ── generate if/else tests ────────────────────────────────────────────────────

#[test]
fn test_generate_if_default_params() {
    // gen_if has FAST=1 (default) → fast_core branch taken, slow_core skipped
    // and WIDE=0 (default) → narrow_bus branch taken, wide_bus skipped
    let file = fixtures_dir().join("gen_if.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let m = design.modules.get("gen_if").expect("gen_if module not found");

    let refs: Vec<&str> = m.instances.iter().map(|i| i.module_ref.as_str()).collect();

    assert!(refs.contains(&"fast_core"),   "fast_core should be instantiated (FAST=1)");
    assert!(!refs.contains(&"slow_core"),  "slow_core must NOT be instantiated (FAST=1)");
    assert!(refs.contains(&"narrow_bus"),  "narrow_bus should be instantiated (WIDE=0)");
    assert!(!refs.contains(&"wide_bus"),   "wide_bus must NOT be instantiated (WIDE=0)");
}

// ── generate case tests ───────────────────────────────────────────────────────

#[test]
fn test_generate_case_default_params() {
    // gen_case has MODE=1 (default) → medium_core selected, small_core and large_core skipped
    let file = fixtures_dir().join("gen_case.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let m = design.modules.get("gen_case").expect("gen_case module not found");

    let refs: Vec<&str> = m.instances.iter().map(|i| i.module_ref.as_str()).collect();

    assert!(refs.contains(&"medium_core"),  "medium_core should be instantiated (MODE=1)");
    assert!(!refs.contains(&"small_core"),  "small_core must NOT be instantiated (MODE=1)");
    assert!(!refs.contains(&"large_core"),  "large_core must NOT be instantiated (MODE=1, not default branch)");
}

// ── generate for loop tests ───────────────────────────────────────────────────

#[test]
fn test_generate_for_loop_expansion() {
    // gen_for has N=4 (default) → 4 instances of unit_cell
    let file = fixtures_dir().join("gen_for.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let m = design.modules.get("gen_for").expect("gen_for module not found");

    let unit_cell_count = m.instances.iter()
        .filter(|i| i.module_ref == "unit_cell")
        .count();

    assert_eq!(unit_cell_count, 4, "generate for N=4 should produce 4 unit_cell instances");
}
