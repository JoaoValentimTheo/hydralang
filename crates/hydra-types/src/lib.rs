use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Unit,
    Never,
}

impl Type {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Int" => Some(Self::Int),
            "Float" => Some(Self::Float),
            "Bool" => Some(Self::Bool),
            "String" => Some(Self::String),
            "Unit" => Some(Self::Unit),
            "Never" => Some(Self::Never),
            _ => None,
        }
    }

    #[must_use]
    pub fn join(left: &Self, right: &Self) -> Option<Self> {
        if left == right {
            return Some(left.clone());
        }
        if *left == Self::Never {
            return Some(right.clone());
        }
        if *right == Self::Never {
            return Some(left.clone());
        }
        None
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => f.write_str("Int"),
            Self::Float => f.write_str("Float"),
            Self::Bool => f.write_str("Bool"),
            Self::String => f.write_str("String"),
            Self::Unit => f.write_str("Unit"),
            Self::Never => f.write_str("Never"),
        }
    }
}
