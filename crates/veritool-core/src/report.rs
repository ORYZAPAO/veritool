use crate::design::{NetKind, DataType};
use std::fmt;

impl fmt::Display for NetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetKind::Wire => write!(f, "wire"),
            NetKind::Logic => write!(f, "logic"),
            NetKind::Reg => write!(f, "reg"),
            NetKind::Var => write!(f, "var"),
            NetKind::Unknown => write!(f, ""),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Bit => write!(f, "bit"),
            DataType::Logic => write!(f, "logic"),
            DataType::Reg => write!(f, "reg"),
            DataType::Byte => write!(f, "byte"),
            DataType::ShortInt => write!(f, "shortint"),
            DataType::Int => write!(f, "int"),
            DataType::LongInt => write!(f, "longint"),
            DataType::Integer => write!(f, "integer"),
            DataType::Time => write!(f, "time"),
            DataType::Real => write!(f, "real"),
            DataType::ShortReal => write!(f, "shortreal"),
            DataType::Double => write!(f, "double"),
            DataType::Signed => write!(f, "signed"),
            DataType::Unsigned => write!(f, "unsigned"),
            DataType::Custom(name) => write!(f, "{}", name),
        }
    }
}
