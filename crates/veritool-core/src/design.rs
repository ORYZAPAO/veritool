use std::collections::HashMap;
use std::path::PathBuf;

use indexmap::IndexMap;
use serde::Serialize;
use sv_parser::SyntaxTree;

use crate::params::ParamEnv;

pub struct Design {
    pub modules: IndexMap<String, Module>,
    pub files: Vec<PathBuf>,
    /// Parsed syntax tree for each source file, retained so generate
    /// constructs can be re-resolved with instance-specific parameter
    /// overrides (see `resolve_instances`).
    pub syntax_trees: HashMap<PathBuf, SyntaxTree>,
}

impl Design {
    /// Resolve a module's instance list under the given parameter environment.
    /// Re-evaluates `generate if/case/for` constructs with `env` so that
    /// instance-specific parameter overrides are reflected in the chosen
    /// generate branches and loop counts. Falls back to the module's
    /// default-resolved `instances` if the source syntax tree is unavailable.
    pub fn resolve_instances(&self, module_name: &str, env: &ParamEnv) -> Vec<Instance> {
        let Some(module) = self.modules.get(module_name) else {
            return Vec::new();
        };
        match self.syntax_trees.get(&module.file) {
            Some(tree) => crate::visit::resolve_instances_with_params(tree, module_name, env),
            None => module.instances.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Module {
    pub name: String,
    pub file: PathBuf,
    pub span: (usize, usize),
    pub params: Vec<ParamDecl>,
    pub ports: Vec<Port>,
    pub signals: Vec<Signal>,
    pub instances: Vec<Instance>,
    pub ff_decls: Vec<FfDecl>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParamDecl {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Port {
    pub name: String,
    pub direction: Direction,
    pub net_kind: NetKind,
    pub data_type: DataType,
    pub packed_width: Option<Range>,
    pub unpacked_dims: Vec<Range>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum NetKind {
    Wire,
    Logic,
    Reg,
    Var,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DataType {
    Bit,
    Logic,
    Reg,
    Byte,
    ShortInt,
    Int,
    LongInt,
    Integer,
    Time,
    Real,
    ShortReal,
    Double,
    Signed,
    Unsigned,
    Custom(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct Range {
    pub msb: String,
    pub lsb: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Signal {
    pub name: String,
    pub net_kind: NetKind,
    pub data_type: DataType,
    pub packed_width: Option<Range>,
    pub unpacked_dims: Vec<Range>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Instance {
    pub inst_name: String,
    pub module_ref: String,
    pub param_overrides: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FfDecl {
    pub signal_name: String,
    pub packed_width: Option<Range>,
    pub unpacked_dims: Vec<Range>,
    pub clock_edge: ClockEdge,
    pub reset_kind: ResetKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ClockEdge {
    Posedge,
    Negedge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ResetKind {
    Sync,
    Async,
    None,
}
