use std::collections::HashSet;

use anyhow::Result;
use comfy_table::{Table, Cell, Attribute};
use veritool_core::design::{DataType, Design, Direction, Module, NetKind, Range, Signal};
use veritool_core::params::{evaluate_expr, ParamEnv};
use veritool_core::width::calculate_width_with_params;

use crate::args::OutputFormat;

// ─── ports ────────────────────────────────────────────────────────────────────

pub fn print_ports(module: &Module, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => print_ports_text(module),
        OutputFormat::Json => print_ports_json(module),
        OutputFormat::Markdown => print_ports_markdown(module),
        OutputFormat::Csv => print_ports_csv(module),
    }
}

fn print_ports_text(module: &Module) -> Result<()> {
    println!("Module: {}", module.name);
    if module.ports.is_empty() {
        println!("  (no ports)");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Direction").add_attribute(Attribute::Bold),
        Cell::new("Type").add_attribute(Attribute::Bold),
        Cell::new("Width").add_attribute(Attribute::Bold),
        Cell::new("Name").add_attribute(Attribute::Bold),
    ]);

    for port in &module.ports {
        let dir = match port.direction {
            Direction::Input => "input",
            Direction::Output => "output",
            Direction::Inout => "inout",
        };
        let type_str = format_type(&port.data_type, &port.net_kind);
        let width_str = format_range(&port.packed_width);
        table.add_row(vec![dir, &type_str, &width_str, &port.name]);
    }
    println!("{table}");
    Ok(())
}

fn print_ports_json(module: &Module) -> Result<()> {
    use serde_json::{json, Value};
    let ports: Vec<Value> = module
        .ports
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "direction": format!("{:?}", p.direction).to_lowercase(),
                "type": format_type(&p.data_type, &p.net_kind),
                "width": format_range(&p.packed_width),
            })
        })
        .collect();
    let obj = json!({ "module": module.name, "ports": ports });
    println!("{}", serde_json::to_string_pretty(&obj)?);
    Ok(())
}

fn print_ports_markdown(module: &Module) -> Result<()> {
    println!("## Module: `{}`\n", module.name);
    println!("| Direction | Type | Width | Name |");
    println!("|-----------|------|-------|------|");
    for port in &module.ports {
        let dir = match port.direction {
            Direction::Input => "input",
            Direction::Output => "output",
            Direction::Inout => "inout",
        };
        let type_str = format_type(&port.data_type, &port.net_kind);
        let width_str = format_range(&port.packed_width);
        println!("| {} | {} | {} | {} |", dir, type_str, width_str, port.name);
    }
    println!();
    Ok(())
}

fn print_ports_csv(module: &Module) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(["module", "direction", "type", "width", "name"])?;
    for port in &module.ports {
        let dir = match port.direction {
            Direction::Input => "input",
            Direction::Output => "output",
            Direction::Inout => "inout",
        };
        wtr.write_record([
            &module.name,
            dir,
            &format_type(&port.data_type, &port.net_kind),
            &format_range(&port.packed_width),
            &port.name,
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

// ─── signals ─────────────────────────────────────────────────────────────────

pub fn print_signals(module: &Module, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => print_signals_text(module),
        OutputFormat::Json => print_signals_json(module),
        OutputFormat::Markdown => print_signals_markdown(module),
        OutputFormat::Csv => print_signals_csv(module),
    }
}

fn print_signals_text(module: &Module) -> Result<()> {
    println!("Module: {}", module.name);
    if module.signals.is_empty() {
        println!("  (no signals)");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Type").add_attribute(Attribute::Bold),
        Cell::new("Packed").add_attribute(Attribute::Bold),
        Cell::new("Unpacked").add_attribute(Attribute::Bold),
        Cell::new("Name").add_attribute(Attribute::Bold),
    ]);

    for sig in &module.signals {
        let type_str = format_type(&sig.data_type, &sig.net_kind);
        let packed_str = format_range(&sig.packed_width);
        let unpacked_str = if sig.unpacked_dims.is_empty() {
            String::new()
        } else {
            sig.unpacked_dims.iter().map(|r| format_range(&Some(r.clone()))).collect::<Vec<_>>().join("")
        };
        table.add_row(vec![&type_str, &packed_str, &unpacked_str, &sig.name]);
    }
    println!("{table}");
    Ok(())
}

fn print_signals_json(module: &Module) -> Result<()> {
    use serde_json::{json, Value};
    let sigs: Vec<Value> = module
        .signals
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "type": format_type(&s.data_type, &s.net_kind),
                "width": format_range(&s.packed_width),
            })
        })
        .collect();
    let obj = json!({ "module": module.name, "signals": sigs });
    println!("{}", serde_json::to_string_pretty(&obj)?);
    Ok(())
}

fn print_signals_markdown(module: &Module) -> Result<()> {
    println!("## Module: `{}`\n", module.name);
    println!("| Type | Width | Name |");
    println!("|------|-------|------|");
    for sig in &module.signals {
        println!(
            "| {} | {} | {} |",
            format_type(&sig.data_type, &sig.net_kind),
            format_range(&sig.packed_width),
            sig.name
        );
    }
    println!();
    Ok(())
}

fn print_signals_csv(module: &Module) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(["module", "type", "width", "name"])?;
    for sig in &module.signals {
        wtr.write_record([
            &module.name,
            &format_type(&sig.data_type, &sig.net_kind),
            &format_range(&sig.packed_width),
            &sig.name,
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

// ─── FF count ────────────────────────────────────────────────────────────────

/// `overrides`: parsed `-P NAME=VAL` from CLI (applied to the specified module's params).
pub fn print_ff_module(
    _design: &Design,
    module: &Module,
    format: OutputFormat,
    overrides: &[(String, i64)],
) -> Result<()> {
    let env = ParamEnv::from_module(module).with_overrides(overrides);
    let ff_count = count_module_ffs_with_env(module, &env);
    match format {
        OutputFormat::Text => {
            println!("Module: {}  FF count: {}", module.name, ff_count);
            Ok(())
        }
        OutputFormat::Json => {
            use serde_json::json;
            let obj = json!({ "module": module.name, "ff_count": ff_count });
            println!("{}", serde_json::to_string_pretty(&obj)?);
            Ok(())
        }
        OutputFormat::Markdown => {
            println!("| Module | FF Count |");
            println!("|--------|----------|");
            println!("| {} | {} |", module.name, ff_count);
            Ok(())
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record(["module", "ff_count"])?;
            wtr.write_record([&module.name, &ff_count.to_string()])?;
            wtr.flush()?;
            Ok(())
        }
    }
}

/// `top_overrides`: parsed `-P NAME=VAL` from CLI (applied to the top module's params).
pub fn print_ff_hierarchy(
    design: &Design,
    top: &str,
    format: OutputFormat,
    top_overrides: &[(String, i64)],
) -> Result<()> {
    let rows = collect_ff_rows(design, top, top_overrides, &mut HashSet::new());
    match format {
        OutputFormat::Text => {
            let mut table = Table::new();
            table.set_header(vec![
                Cell::new("Module").add_attribute(Attribute::Bold),
                Cell::new("Own FFs").add_attribute(Attribute::Bold),
                Cell::new("Total FFs").add_attribute(Attribute::Bold),
            ]);
            for (name, own, total) in &rows {
                table.add_row(vec![name, &own.to_string(), &total.to_string()]);
            }
            println!("{table}");
            Ok(())
        }
        OutputFormat::Json => {
            use serde_json::{json, Value};
            let arr: Vec<Value> = rows
                .iter()
                .map(|(name, own, total)| json!({ "module": name, "own_ffs": own, "total_ffs": total }))
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
            Ok(())
        }
        OutputFormat::Markdown => {
            println!("| Module | Own FFs | Total FFs |");
            println!("|--------|---------|-----------|");
            for (name, own, total) in &rows {
                println!("| {} | {} | {} |", name, own, total);
            }
            Ok(())
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record(["module", "own_ffs", "total_ffs"])?;
            for (name, own, total) in &rows {
                wtr.write_record([name.as_str(), &own.to_string(), &total.to_string()])?;
            }
            wtr.flush()?;
            Ok(())
        }
    }
}

// ─── hierarchy ───────────────────────────────────────────────────────────────

pub fn print_hier(design: &Design, top: &str, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            print_hier_text(design, top, "", true, &mut HashSet::new());
            Ok(())
        }
        OutputFormat::Json => {
            let tree = build_hier_json(design, top, &mut HashSet::new());
            println!("{}", serde_json::to_string_pretty(&tree)?);
            Ok(())
        }
        OutputFormat::Markdown => {
            print_hier_markdown(design, top, 0, &mut HashSet::new());
            Ok(())
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record(["depth", "instance", "module"])?;
            print_hier_csv(&mut wtr, design, top, top, 0, &mut HashSet::new())?;
            wtr.flush()?;
            Ok(())
        }
    }
}

fn print_hier_text(
    design: &Design,
    mod_name: &str,
    prefix: &str,
    is_last: bool,
    visited: &mut HashSet<String>,
) {
    let connector = if is_last { "└─ " } else { "├─ " };
    println!("{}{}{}", prefix, connector, mod_name);

    if visited.contains(mod_name) {
        let child_prefix = if is_last { format!("{}   ", prefix) } else { format!("{}│  ", prefix) };
        println!("{}(circular reference)", child_prefix);
        return;
    }
    visited.insert(mod_name.to_string());

    let child_prefix = if is_last { format!("{}   ", prefix) } else { format!("{}│  ", prefix) };
    print_hier_instances(design, mod_name, &child_prefix, visited);
    visited.remove(mod_name);
}

fn print_hier_instances(
    design: &Design,
    mod_name: &str,
    prefix: &str,
    visited: &mut HashSet<String>,
) {
    if let Some(module) = design.modules.get(mod_name) {
        let n = module.instances.len();
        for (i, inst) in module.instances.iter().enumerate() {
            let child_last = i + 1 == n;
            let child_connector = if child_last { "└─ " } else { "├─ " };
            println!("{}{}{} ({})", prefix, child_connector, inst.inst_name, inst.module_ref);

            if visited.contains(&inst.module_ref) {
                let gc_prefix = if child_last { format!("{}   ", prefix) } else { format!("{}│  ", prefix) };
                println!("{}(circular reference)", gc_prefix);
                continue;
            }
            visited.insert(inst.module_ref.clone());
            let gc_prefix = if child_last { format!("{}   ", prefix) } else { format!("{}│  ", prefix) };
            print_hier_instances(design, &inst.module_ref, &gc_prefix, visited);
            visited.remove(&inst.module_ref);
        }
    }
}

fn print_hier_markdown(
    design: &Design,
    mod_name: &str,
    depth: usize,
    visited: &mut HashSet<String>,
) {
    let indent = "  ".repeat(depth);
    println!("{}- **{}**", indent, mod_name);

    if visited.contains(mod_name) {
        println!("{}  _(circular)_", indent);
        return;
    }
    visited.insert(mod_name.to_string());
    print_hier_markdown_children(design, mod_name, depth, visited);
    visited.remove(mod_name);
}

fn print_hier_markdown_children(
    design: &Design,
    mod_name: &str,
    depth: usize,
    visited: &mut HashSet<String>,
) {
    let indent = "  ".repeat(depth);
    if let Some(module) = design.modules.get(mod_name) {
        for inst in &module.instances {
            println!("{}  - `{}` ({})", indent, inst.inst_name, inst.module_ref);
            if visited.contains(&inst.module_ref) {
                println!("{}    _(circular)_", indent);
                continue;
            }
            visited.insert(inst.module_ref.clone());
            print_hier_markdown_children(design, &inst.module_ref, depth + 1, visited);
            visited.remove(&inst.module_ref);
        }
    }
}

fn build_hier_json(
    design: &Design,
    mod_name: &str,
    visited: &mut HashSet<String>,
) -> serde_json::Value {
    use serde_json::{json, Value};
    if visited.contains(mod_name) {
        return json!({ "module": mod_name, "circular": true });
    }
    visited.insert(mod_name.to_string());

    let children: Vec<Value> = if let Some(module) = design.modules.get(mod_name) {
        module
            .instances
            .iter()
            .map(|inst| {
                let mut child = build_hier_json(design, &inst.module_ref, visited);
                if let Some(obj) = child.as_object_mut() {
                    obj.insert("instance".to_string(), json!(inst.inst_name));
                }
                child
            })
            .collect()
    } else {
        Vec::new()
    };

    visited.remove(mod_name);
    json!({ "module": mod_name, "children": children })
}

fn print_hier_csv(
    wtr: &mut csv::Writer<std::io::Stdout>,
    design: &Design,
    inst_name: &str,
    mod_name: &str,
    depth: usize,
    visited: &mut HashSet<String>,
) -> Result<()> {
    wtr.write_record([&depth.to_string(), inst_name, mod_name])?;
    if visited.contains(mod_name) {
        return Ok(());
    }
    visited.insert(mod_name.to_string());
    if let Some(module) = design.modules.get(mod_name) {
        for inst in &module.instances {
            print_hier_csv(wtr, design, &inst.inst_name, &inst.module_ref, depth + 1, visited)?;
        }
    }
    visited.remove(mod_name);
    Ok(())
}

// ─── top modules ─────────────────────────────────────────────────────────────

pub fn print_top(tops: &[String], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!("Top modules:");
            for t in tops {
                println!("  {}", t);
            }
            Ok(())
        }
        OutputFormat::Json => {
            use serde_json::json;
            println!("{}", serde_json::to_string_pretty(&json!({ "top_modules": tops }))?);
            Ok(())
        }
        OutputFormat::Markdown => {
            println!("## Top Modules\n");
            for t in tops {
                println!("- `{}`", t);
            }
            Ok(())
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record(["module"])?;
            for t in tops {
                wtr.write_record([t.as_str()])?;
            }
            wtr.flush()?;
            Ok(())
        }
    }
}

// ─── report ──────────────────────────────────────────────────────────────────

pub fn print_report(design: &Design, top: &str, format: OutputFormat) -> Result<()> {
    // Report uses module defaults only (no -P overrides)
    let no_overrides: &[(String, i64)] = &[];
    match format {
        OutputFormat::Text => {
            println!("=== veritool report: top = {} ===\n", top);
            for module in design.modules.values() {
                print_ports_text(module)?;
                println!();
            }
            println!("--- FF hierarchy ---");
            print_ff_hierarchy(design, top, OutputFormat::Text, no_overrides)?;
            Ok(())
        }
        OutputFormat::Json => {
            use serde_json::{json, Value};
            let modules: Vec<Value> = design
                .modules
                .values()
                .map(|m| {
                    let env = ParamEnv::from_module(m);
                    let ports: Vec<Value> = m
                        .ports
                        .iter()
                        .map(|p| {
                            json!({
                                "name": p.name,
                                "direction": format!("{:?}", p.direction).to_lowercase(),
                                "type": format_type(&p.data_type, &p.net_kind),
                                "width": format_range(&p.packed_width),
                            })
                        })
                        .collect();
                    json!({
                        "name": m.name,
                        "ports": ports,
                        "ff_count": count_module_ffs_with_env(m, &env),
                    })
                })
                .collect();
            let report = json!({ "top": top, "modules": modules });
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        OutputFormat::Markdown => {
            println!("# veritool Report\n");
            println!("**Top module:** `{}`\n", top);
            for module in design.modules.values() {
                print_ports_markdown(module)?;
            }
            println!("## FF Hierarchy\n");
            print_ff_hierarchy(design, top, OutputFormat::Markdown, no_overrides)?;
            Ok(())
        }
        OutputFormat::Csv => {
            print_ff_hierarchy(design, top, OutputFormat::Csv, no_overrides)?;
            Ok(())
        }
    }
}

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Return canonical type string for a port/signal.
/// `reg` in Verilog implies logic storage, so we unify:
///   (Logic, Reg)    → "reg"
///   (Logic, Logic)  → "logic"
///   (Wire, Logic)   → "wire"
///   (Wire, Wire)    → "wire"
///   otherwise       → "net_kind data_type"
fn format_type(dtype: &DataType, net_kind: &NetKind) -> String {
    match (dtype, net_kind) {
        (DataType::Reg, NetKind::Logic | NetKind::Var | NetKind::Unknown) => "reg".to_string(),
        (DataType::Logic, NetKind::Logic | NetKind::Var | NetKind::Unknown) => "logic".to_string(),
        (DataType::Logic, NetKind::Wire) => "wire".to_string(),
        (DataType::Bit, NetKind::Logic | NetKind::Var | NetKind::Unknown) => "bit".to_string(),
        _ => {
            let dt = format!("{}", dtype);
            let nk = format!("{}", net_kind);
            if nk.is_empty() || dt == nk { dt } else { format!("{} {}", nk, dt) }
        }
    }
}

fn format_range(range: &Option<Range>) -> String {
    match range {
        Some(r) => format!("[{}:{}]", r.msb, r.lsb),
        None => String::new(),
    }
}

/// Count own FFs of a module given an already-resolved ParamEnv.
fn count_module_ffs_with_env(module: &Module, env: &ParamEnv) -> i64 {
    module
        .ff_decls
        .iter()
        .map(|ff| {
            if let Some(sig) = module.signals.iter().find(|s| s.name == ff.signal_name) {
                calculate_width_with_params(sig, env)
            } else if let Some(port) = module.ports.iter().find(|p| p.name == ff.signal_name) {
                let sig = Signal {
                    name: port.name.clone(),
                    net_kind: port.net_kind.clone(),
                    data_type: port.data_type.clone(),
                    packed_width: port.packed_width.clone(),
                    unpacked_dims: port.unpacked_dims.clone(),
                };
                calculate_width_with_params(&sig, env)
            } else {
                1 // unknown — assume 1-bit
            }
        })
        .sum()
}

/// Build (module_name, own_ffs, total_ffs) rows for `ff` hierarchy output,
/// propagating parameter bindings from parent instances.
///
/// `instance_overrides`: parameter values bound by the parent instantiation.
fn collect_ff_rows(
    design: &Design,
    mod_name: &str,
    instance_overrides: &[(String, i64)],
    visited: &mut HashSet<String>,
) -> Vec<(String, i64, i64)> {
    if visited.contains(mod_name) {
        return Vec::new();
    }
    visited.insert(mod_name.to_string());

    let mut rows = Vec::new();
    if let Some(module) = design.modules.get(mod_name) {
        // Resolve this module's params: defaults + parent-supplied overrides
        let env = ParamEnv::from_module(module).with_overrides(instance_overrides);
        let own = count_module_ffs_with_env(module, &env);

        // Recursively process children
        let mut child_total = 0i64;
        for inst in &module.instances {
            // Evaluate this instance's parameter overrides in the current env
            let child_overrides: Vec<(String, i64)> = inst
                .param_overrides
                .iter()
                .filter_map(|(pname, expr)| {
                    let val = evaluate_expr(expr.trim(), env.as_map())?;
                    Some((pname.clone(), val))
                })
                .collect();

            let child_rows =
                collect_ff_rows(design, &inst.module_ref, &child_overrides, visited);
            if let Some((_, _, child_t)) = child_rows.first() {
                child_total += child_t;
            }
            rows.extend(child_rows);
        }

        // Insert this module's row at the front (DFS pre-order)
        rows.insert(0, (mod_name.to_string(), own, own + child_total));
    }

    visited.remove(mod_name);
    rows
}
