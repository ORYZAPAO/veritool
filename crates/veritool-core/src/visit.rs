use std::path::Path;
use sv_parser::{NodeEvent, RefNode, SyntaxTree, unwrap_node};

use crate::design::{
    ClockEdge, DataType, Design, Direction, FfDecl, Instance, Module, NetKind, ParamDecl, Port,
    Range, ResetKind, Signal,
};

pub fn visit_syntax_tree(tree: &SyntaxTree, file: &Path, design: &mut Design) {
    let mut module_stack: Vec<String> = Vec::new();
    let mut always_stack: Vec<bool> = Vec::new();
    // Track inherited port attributes for ANSI port lists
    let mut last_port_dir = Direction::Input;
    let mut last_port_dtype = DataType::Logic;
    let mut last_port_packed: Option<Range> = None;
    let mut last_port_net_kind = NetKind::Logic;

    for event in tree.into_iter().event() {
        match event {
            NodeEvent::Enter(RefNode::ModuleDeclarationAnsi(m)) => {
                let name = get_module_name(tree, m).unwrap_or_else(|| "<unknown>".to_string());
                if module_stack.is_empty() {
                    design.modules.insert(
                        name.clone(),
                        Module {
                            name: name.clone(),
                            file: file.to_path_buf(),
                            span: (0, 0),
                            params: Vec::new(),
                            ports: Vec::new(),
                            signals: Vec::new(),
                            instances: Vec::new(),
                            ff_decls: Vec::new(),
                        },
                    );
                    // Reset port tracking for new module
                    last_port_dir = Direction::Input;
                    last_port_dtype = DataType::Logic;
                    last_port_packed = None;
                    last_port_net_kind = NetKind::Logic;
                }
                module_stack.push(name);
            }
            NodeEvent::Leave(RefNode::ModuleDeclarationAnsi(_)) => {
                module_stack.pop();
            }
            NodeEvent::Enter(RefNode::ModuleDeclarationNonansi(m)) => {
                let name =
                    get_module_name_nonansi(tree, m).unwrap_or_else(|| "<unknown>".to_string());
                if module_stack.is_empty() {
                    design.modules.insert(
                        name.clone(),
                        Module {
                            name: name.clone(),
                            file: file.to_path_buf(),
                            span: (0, 0),
                            params: Vec::new(),
                            ports: Vec::new(),
                            signals: Vec::new(),
                            instances: Vec::new(),
                            ff_decls: Vec::new(),
                        },
                    );
                }
                module_stack.push(name);
            }
            NodeEvent::Leave(RefNode::ModuleDeclarationNonansi(_)) => {
                module_stack.pop();
            }

            // Only process direct module contents (not nested modules)
            _ if module_stack.len() != 1 => {}

            // ── ANSI port declarations ────────────────────────────────────────
            NodeEvent::Enter(RefNode::AnsiPortDeclarationNet(port)) => {
                let mod_name = module_stack[0].clone();
                if let Some(p) = extract_ansi_net_port(
                    tree,
                    port,
                    &mut last_port_dir,
                    &mut last_port_dtype,
                    &mut last_port_packed,
                    &mut last_port_net_kind,
                ) {
                    if let Some(m) = design.modules.get_mut(&mod_name) {
                        m.ports.push(p);
                    }
                }
            }
            NodeEvent::Enter(RefNode::AnsiPortDeclarationVariable(port)) => {
                let mod_name = module_stack[0].clone();
                if let Some(p) = extract_ansi_variable_port(
                    tree,
                    port,
                    &mut last_port_dir,
                    &mut last_port_dtype,
                    &mut last_port_packed,
                ) {
                    if let Some(m) = design.modules.get_mut(&mod_name) {
                        m.ports.push(p);
                    }
                }
            }

            // ── Non-ANSI port declarations ────────────────────────────────────
            NodeEvent::Enter(RefNode::InputDeclarationNet(decl)) => {
                let mod_name = module_stack[0].clone();
                let ports = extract_input_net_ports(tree, decl);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.ports.extend(ports);
                }
            }
            NodeEvent::Enter(RefNode::InputDeclarationVariable(decl)) => {
                let mod_name = module_stack[0].clone();
                let ports = extract_input_var_ports(tree, decl);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.ports.extend(ports);
                }
            }
            NodeEvent::Enter(RefNode::OutputDeclarationNet(decl)) => {
                let mod_name = module_stack[0].clone();
                let ports = extract_output_net_ports(tree, decl);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.ports.extend(ports);
                }
            }
            NodeEvent::Enter(RefNode::OutputDeclarationVariable(decl)) => {
                let mod_name = module_stack[0].clone();
                let ports = extract_output_var_ports(tree, decl);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.ports.extend(ports);
                }
            }
            NodeEvent::Enter(RefNode::InoutDeclaration(decl)) => {
                let mod_name = module_stack[0].clone();
                let ports = extract_inout_ports(tree, decl);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.ports.extend(ports);
                }
            }

            // ── Signal declarations ───────────────────────────────────────────
            NodeEvent::Enter(RefNode::DataDeclarationVariable(decl)) => {
                let mod_name = module_stack[0].clone();
                let sigs = extract_data_decl_variable(tree, decl);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.signals.extend(sigs);
                }
            }
            NodeEvent::Enter(RefNode::NetDeclarationNetType(decl)) => {
                let mod_name = module_stack[0].clone();
                let sigs = extract_net_decl(tree, decl);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.signals.extend(sigs);
                }
            }

            // ── Module instantiation ──────────────────────────────────────────
            NodeEvent::Enter(RefNode::ModuleInstantiation(inst)) => {
                let mod_name = module_stack[0].clone();
                let insts = extract_module_instantiation(tree, inst);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.instances.extend(insts);
                }
            }

            // ── Parameter declarations (parameter + localparam) ───────────────
            NodeEvent::Enter(RefNode::ParameterDeclaration(param)) => {
                let mod_name = module_stack[0].clone();
                let params = extract_param_decl(tree, param);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.params.extend(params);
                }
            }
            NodeEvent::Enter(RefNode::LocalParameterDeclaration(param)) => {
                let mod_name = module_stack[0].clone();
                let params = extract_localparam_decl(tree, param);
                if let Some(m) = design.modules.get_mut(&mod_name) {
                    m.params.extend(params);
                }
            }

            // ── FF / always blocks ────────────────────────────────────────────
            NodeEvent::Enter(RefNode::AlwaysConstruct(always)) => {
                always_stack.push(check_is_clocked_always(tree, always));
            }
            NodeEvent::Leave(RefNode::AlwaysConstruct(_)) => {
                always_stack.pop();
            }
            NodeEvent::Enter(RefNode::NonblockingAssignment(assign))
                if always_stack.last().copied().unwrap_or(false) =>
            {
                let mod_name = module_stack[0].clone();
                if let Some(ff) = extract_ff_from_nbassign(tree, assign) {
                    if let Some(m) = design.modules.get_mut(&mod_name) {
                        if !m.ff_decls.iter().any(|f| f.signal_name == ff.signal_name) {
                            m.ff_decls.push(ff);
                        }
                    }
                }
            }

            _ => {}
        }
    }
}

// ─── Node name helpers ────────────────────────────────────────────────────────

fn get_module_name(tree: &SyntaxTree, m: &sv_parser::ModuleDeclarationAnsi) -> Option<String> {
    let id = unwrap_node!(m, ModuleIdentifier)?;
    get_identifier_text(tree, id)
}

fn get_module_name_nonansi(
    tree: &SyntaxTree,
    m: &sv_parser::ModuleDeclarationNonansi,
) -> Option<String> {
    let id = unwrap_node!(m, ModuleIdentifier)?;
    get_identifier_text(tree, id)
}

fn get_identifier_text(tree: &SyntaxTree, node: RefNode) -> Option<String> {
    match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
        Some(RefNode::SimpleIdentifier(x)) => {
            tree.get_str(&x.nodes.0).map(|s| s.trim().to_string())
        }
        Some(RefNode::EscapedIdentifier(x)) => {
            tree.get_str(&x.nodes.0)
                .map(|s| s.trim_start_matches('\\').trim().to_string())
        }
        _ => None,
    }
}

// ─── Port extraction ──────────────────────────────────────────────────────────

fn extract_ansi_net_port(
    tree: &SyntaxTree,
    port: &sv_parser::AnsiPortDeclarationNet,
    last_dir: &mut Direction,
    last_dtype: &mut DataType,
    last_packed: &mut Option<Range>,
    last_net_kind: &mut NetKind,
) -> Option<Port> {
    let name_node = unwrap_node!(port, PortIdentifier)?;
    let name = get_identifier_text(tree, name_node)?;

    let has_header = port.nodes.0.is_some();

    let (direction, data_type, packed_width, net_kind) = if has_header {
        let dir = extract_port_direction(tree, port);
        let (dtype, packed, nk) = extract_net_port_type_info(tree, port);
        let dir = dir.unwrap_or(Direction::Input);
        *last_dir = dir;
        *last_dtype = dtype.clone();
        *last_packed = packed.clone();
        *last_net_kind = nk.clone();
        (dir, dtype, packed, nk)
    } else {
        (*last_dir, last_dtype.clone(), last_packed.clone(), last_net_kind.clone())
    };

    let unpacked_dims = extract_unpacked_dims_from_node(tree, port);

    Some(Port {
        name,
        direction,
        net_kind,
        data_type,
        packed_width,
        unpacked_dims,
    })
}

fn extract_ansi_variable_port(
    tree: &SyntaxTree,
    port: &sv_parser::AnsiPortDeclarationVariable,
    last_dir: &mut Direction,
    last_dtype: &mut DataType,
    last_packed: &mut Option<Range>,
) -> Option<Port> {
    let name_node = unwrap_node!(port, PortIdentifier)?;
    let name = get_identifier_text(tree, name_node)?;

    let has_header = port.nodes.0.is_some();

    let (direction, data_type, packed_width) = if has_header {
        let dir = extract_port_direction(tree, port).unwrap_or(Direction::Output);
        let (dtype, packed) = extract_var_port_type_info(tree, port);
        *last_dir = dir;
        *last_dtype = dtype.clone();
        *last_packed = packed.clone();
        (dir, dtype, packed)
    } else {
        (*last_dir, last_dtype.clone(), last_packed.clone())
    };

    let unpacked_dims = extract_unpacked_dims_from_node(tree, port);

    Some(Port {
        name,
        direction,
        net_kind: NetKind::Logic,
        data_type,
        packed_width,
        unpacked_dims,
    })
}

fn extract_port_direction<'a, T: 'a>(tree: &SyntaxTree, node: &'a T) -> Option<Direction>
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    if let Some(RefNode::PortDirection(pd)) = unwrap_node!(node, PortDirection) {
        let text = tree.get_str(pd).unwrap_or("").trim();
        Some(match text {
            "output" => Direction::Output,
            "inout" => Direction::Inout,
            _ => Direction::Input,
        })
    } else {
        None
    }
}

fn extract_net_port_type_info<'a, T: 'a>(
    tree: &SyntaxTree,
    node: &'a T,
) -> (DataType, Option<Range>, NetKind)
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    let net_kind = if let Some(RefNode::NetType(nt)) = unwrap_node!(node, NetType) {
        let text = tree.get_str(nt).unwrap_or("").trim();
        match text {
            "wire" | "tri" | "wand" | "wor" | "triand" | "trior" | "trireg"
            | "tri0" | "tri1" | "uwire" | "supply0" | "supply1" => NetKind::Wire,
            _ => NetKind::Logic,
        }
    } else {
        NetKind::Logic
    };

    let (dtype, packed) = extract_data_type_info(tree, node);
    (dtype, packed, net_kind)
}

fn extract_var_port_type_info<'a, T: 'a>(
    tree: &SyntaxTree,
    node: &'a T,
) -> (DataType, Option<Range>)
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    extract_data_type_info(tree, node)
}

fn extract_data_type_info<'a, T: 'a>(
    tree: &SyntaxTree,
    node: &'a T,
) -> (DataType, Option<Range>)
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    let dtype = if let Some(RefNode::IntegerVectorType(ivt)) =
        unwrap_node!(node, IntegerVectorType)
    {
        let text = tree.get_str(ivt).unwrap_or("").trim();
        match text {
            "logic" => DataType::Logic,
            "reg" => DataType::Reg,
            "bit" => DataType::Bit,
            _ => DataType::Custom(text.to_string()),
        }
    } else if let Some(RefNode::IntegerAtomType(iat)) = unwrap_node!(node, IntegerAtomType) {
        let text = tree.get_str(iat).unwrap_or("").trim();
        match text {
            "byte" => DataType::Byte,
            "shortint" => DataType::ShortInt,
            "int" => DataType::Int,
            "longint" => DataType::LongInt,
            "integer" => DataType::Integer,
            "time" => DataType::Time,
            _ => DataType::Logic,
        }
    } else {
        DataType::Logic
    };

    let packed = extract_first_packed_dim(tree, node);
    (dtype, packed)
}

fn extract_first_packed_dim<'a, T: 'a>(tree: &SyntaxTree, node: &'a T) -> Option<Range>
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    if let Some(RefNode::PackedDimensionRange(pdr)) = unwrap_node!(node, PackedDimensionRange) {
        let const_range = &pdr.nodes.0.nodes.1;
        let msb = tree
            .get_str(&const_range.nodes.0)
            .unwrap_or("")
            .trim()
            .to_string();
        let lsb = tree
            .get_str(&const_range.nodes.2)
            .unwrap_or("")
            .trim()
            .to_string();
        if msb.is_empty() && lsb.is_empty() {
            None
        } else {
            Some(Range { msb, lsb })
        }
    } else {
        None
    }
}

fn extract_unpacked_dims_from_node<'a, T: 'a>(tree: &SyntaxTree, node: &'a T) -> Vec<Range>
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    let mut dims = Vec::new();
    for n in node {
        if let RefNode::UnpackedDimensionRange(udr) = n {
            let const_range = &udr.nodes.0.nodes.1;
            let msb = tree
                .get_str(&const_range.nodes.0)
                .unwrap_or("")
                .trim()
                .to_string();
            let lsb = tree
                .get_str(&const_range.nodes.2)
                .unwrap_or("")
                .trim()
                .to_string();
            dims.push(Range { msb, lsb });
        }
    }
    dims
}

// ─── Non-ANSI port helpers ────────────────────────────────────────────────────

fn extract_input_net_ports(tree: &SyntaxTree, decl: &sv_parser::InputDeclarationNet) -> Vec<Port> {
    extract_ports_from_identifiers(tree, decl, Direction::Input)
}

fn extract_input_var_ports(
    tree: &SyntaxTree,
    decl: &sv_parser::InputDeclarationVariable,
) -> Vec<Port> {
    extract_ports_from_identifiers(tree, decl, Direction::Input)
}

fn extract_output_net_ports(
    tree: &SyntaxTree,
    decl: &sv_parser::OutputDeclarationNet,
) -> Vec<Port> {
    extract_ports_from_identifiers(tree, decl, Direction::Output)
}

fn extract_output_var_ports(
    tree: &SyntaxTree,
    decl: &sv_parser::OutputDeclarationVariable,
) -> Vec<Port> {
    extract_ports_from_identifiers(tree, decl, Direction::Output)
}

fn extract_inout_ports(tree: &SyntaxTree, decl: &sv_parser::InoutDeclaration) -> Vec<Port> {
    extract_ports_from_identifiers(tree, decl, Direction::Inout)
}

fn extract_ports_from_identifiers<'a, T: 'a>(
    tree: &SyntaxTree,
    node: &'a T,
    direction: Direction,
) -> Vec<Port>
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    let (dtype, packed) = extract_data_type_info(tree, node);
    let mut ports = Vec::new();
    for n in node {
        if let RefNode::PortIdentifier(pi) = n {
            if let Some(name) = get_identifier_text(tree, RefNode::PortIdentifier(pi)) {
                ports.push(Port {
                    name,
                    direction,
                    net_kind: NetKind::Logic,
                    data_type: dtype.clone(),
                    packed_width: packed.clone(),
                    unpacked_dims: Vec::new(),
                });
            }
        }
    }
    ports
}

// ─── Signal extraction ────────────────────────────────────────────────────────

fn extract_data_decl_variable(
    tree: &SyntaxTree,
    decl: &sv_parser::DataDeclarationVariable,
) -> Vec<Signal> {
    let (dtype, packed) = extract_data_type_info(tree, decl);
    let net_kind = if let Some(RefNode::Var(_)) = unwrap_node!(decl, Var) {
        NetKind::Var
    } else {
        NetKind::Logic
    };

    let mut signals = Vec::new();
    for node in decl {
        if let RefNode::VariableDeclAssignmentVariable(v) = node {
            if let Some(name_node) = unwrap_node!(v, VariableIdentifier) {
                if let Some(name) = get_identifier_text(tree, name_node) {
                    let unpacked = extract_unpacked_dims_from_node(tree, v);
                    signals.push(Signal {
                        name,
                        net_kind: net_kind.clone(),
                        data_type: dtype.clone(),
                        packed_width: packed.clone(),
                        unpacked_dims: unpacked,
                    });
                }
            }
        }
    }
    signals
}

fn extract_net_decl(
    tree: &SyntaxTree,
    decl: &sv_parser::NetDeclarationNetType,
) -> Vec<Signal> {
    let (dtype, packed) = extract_data_type_info(tree, decl);

    let mut signals = Vec::new();
    for node in decl {
        if let RefNode::NetIdentifier(ni) = node {
            if let Some(name) = get_identifier_text(tree, RefNode::NetIdentifier(ni)) {
                signals.push(Signal {
                    name,
                    net_kind: NetKind::Wire,
                    data_type: dtype.clone(),
                    packed_width: packed.clone(),
                    unpacked_dims: Vec::new(),
                });
            }
        }
    }
    signals
}

// ─── Module instantiation ─────────────────────────────────────────────────────

fn extract_module_instantiation(
    tree: &SyntaxTree,
    inst: &sv_parser::ModuleInstantiation,
) -> Vec<Instance> {
    // inst.nodes.0 = ModuleIdentifier (module type)
    let module_ref = if let Some(id_node) = unwrap_node!(inst, ModuleIdentifier) {
        get_identifier_text(tree, id_node).unwrap_or_default()
    } else {
        return Vec::new();
    };

    let mut instances = Vec::new();

    // inst.nodes.2 = List<Symbol, HierarchicalInstance>
    for node in inst {
        if let RefNode::HierarchicalInstance(hi) = node {
            // hi.nodes.0 = NameOfInstance -> InstanceIdentifier
            if let Some(name_node) = unwrap_node!(hi, InstanceIdentifier) {
                if let Some(inst_name) = get_identifier_text(tree, name_node) {
                    // Extract parameter overrides
                    let param_overrides = extract_param_overrides(tree, hi);
                    instances.push(Instance {
                        inst_name,
                        module_ref: module_ref.clone(),
                        param_overrides,
                    });
                }
            }
        }
    }
    instances
}

fn extract_param_overrides(
    tree: &SyntaxTree,
    hi: &sv_parser::HierarchicalInstance,
) -> Vec<(String, String)> {
    let mut overrides = Vec::new();
    for node in hi {
        if let RefNode::NamedParameterAssignment(npa) = node {
            // npa.nodes.1 = ParameterIdentifier, npa.nodes.2 = Paren<Option<ParamExpression>>
            let param_name = if let Some(pid) = unwrap_node!(npa, ParameterIdentifier) {
                get_identifier_text(tree, pid).unwrap_or_default()
            } else {
                continue;
            };
            let param_val = tree.get_str(npa).unwrap_or("").trim().to_string();
            overrides.push((param_name, param_val));
        }
    }
    overrides
}

// ─── Parameter declarations ───────────────────────────────────────────────────

fn extract_param_decl(
    tree: &SyntaxTree,
    param: &sv_parser::ParameterDeclaration,
) -> Vec<ParamDecl> {
    extract_param_assignments(tree, param)
}

fn extract_localparam_decl(
    tree: &SyntaxTree,
    param: &sv_parser::LocalParameterDeclaration,
) -> Vec<ParamDecl> {
    extract_param_assignments(tree, param)
}

/// Extract `ParamDecl` from any node that contains `ParamAssignment` children.
fn extract_param_assignments<'a, T: 'a>(tree: &SyntaxTree, node: &'a T) -> Vec<ParamDecl>
where
    &'a T: IntoIterator<Item = RefNode<'a>>,
{
    let mut params = Vec::new();
    for n in node {
        if let RefNode::ParamAssignment(pa) = n {
            // pa.nodes: (ParameterIdentifier, Vec<UnpackedDimension>, Option<(Symbol, ConstantParamExpression)>)
            let name = unwrap_node!(pa, ParameterIdentifier)
                .and_then(|id| get_identifier_text(tree, id));
            let value = unwrap_node!(pa, ConstantParamExpression)
                .and_then(|node| {
                    if let RefNode::ConstantParamExpression(cpe) = node {
                        tree.get_str(cpe)
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                    } else {
                        None
                    }
                });
            if let (Some(n), Some(v)) = (name, value) {
                params.push(ParamDecl { name: n, value: v });
            }
        }
    }
    params
}

// ─── FF / always detection ────────────────────────────────────────────────────

fn check_is_clocked_always(_tree: &SyntaxTree, always: &sv_parser::AlwaysConstruct) -> bool {
    // Check for always_ff keyword
    if let Some(RefNode::AlwaysKeyword(kw)) = unwrap_node!(always, AlwaysKeyword) {
        match kw {
            sv_parser::AlwaysKeyword::AlwaysFf(_) => return true,
            sv_parser::AlwaysKeyword::Always(_) => {
                // Check for edge trigger (posedge/negedge)
                return unwrap_node!(always, EdgeIdentifier).is_some();
            }
            _ => return false,
        }
    }
    false
}

fn extract_ff_from_nbassign(
    tree: &SyntaxTree,
    assign: &sv_parser::NonblockingAssignment,
) -> Option<FfDecl> {
    // LHS is assign.nodes.0 (VariableLvalue)
    // Find the base variable name (SimpleIdentifier at the top of the LHS)
    let name_node = unwrap_node!(&assign.nodes.0, HierarchicalVariableIdentifier)?;
    let name = get_identifier_text(tree, name_node)?;
    // Strip off any array index (e.g., "q[0]" → "q")
    let name = name.split('[').next().unwrap_or(&name).trim().to_string();

    Some(FfDecl {
        signal_name: name,
        packed_width: None,
        unpacked_dims: Vec::new(),
        clock_edge: ClockEdge::Posedge,
        reset_kind: ResetKind::None,
    })
}
