use std::path::PathBuf;

use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct Design {
    pub modules: IndexMap<String, Module>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub direction: Direction,
    pub net_kind: NetKind,
    pub data_type: DataType,
    pub packed_width: Option<Range>,
    pub unpacked_dims: Vec<Range>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetKind {
    Wire,
    Logic,
    Reg,
    Var,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct Range {
    pub msb: String,
    pub lsb: String,
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub name: String,
    pub net_kind: NetKind,
    pub data_type: DataType,
    pub packed_width: Option<Range>,
    pub unpacked_dims: Vec<Range>,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub inst_name: String,
    pub module_ref: String,
    pub param_overrides: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct FfDecl {
    pub signal_name: String,
    pub packed_width: Option<Range>,
    pub unpacked_dims: Vec<Range>,
    pub clock_edge: ClockEdge,
    pub reset_kind: ResetKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockEdge {
    Posedge,
    Negedge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetKind {
    Sync,
    Async,
    None,
}
