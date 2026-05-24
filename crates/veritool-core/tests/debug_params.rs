use std::path::PathBuf;
use veritool_core::{loader, ParamEnv};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("tests/fixtures")
}

#[test]
fn debug_counter_params() {
    let file = fixtures_dir().join("counter.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let counter = design.modules.get("counter").unwrap();
    println!("params: {:?}", counter.params);
    println!("ff_decls: {:?}", counter.ff_decls);
    let env = ParamEnv::from_module(counter);
    println!("env: {:?}", env);
    let q_port = counter.ports.iter().find(|p| p.name == "q").unwrap();
    println!("q packed_width: {:?}", q_port.packed_width);
}

#[test]
fn debug_fifo_params() {
    let file = fixtures_dir().join("fifo_sync.sv");
    let design = loader::parse_sv_files(&[file], &[], &[]).unwrap();
    let fifo = design.modules.get("fifo_sync").unwrap();
    println!("params: {:?}", fifo.params);
    let env = ParamEnv::from_module(fifo);
    println!("env: {:?}", env);
    let wr = fifo.signals.iter().find(|s| s.name == "wr_ptr").unwrap();
    println!("wr_ptr packed_width: {:?}", wr.packed_width);
}
