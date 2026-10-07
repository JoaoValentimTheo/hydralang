use hydra_types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuiltinParam {
    Exact(Type),
    Printable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinSignature {
    pub name: &'static str,
    pub params: Vec<BuiltinParam>,
    pub return_type: Type,
}

#[must_use]
pub fn builtins() -> Vec<BuiltinSignature> {
    vec![
        BuiltinSignature {
            name: "print",
            params: vec![BuiltinParam::Printable],
            return_type: Type::Unit,
        },
        BuiltinSignature {
            name: "println",
            params: vec![BuiltinParam::Printable],
            return_type: Type::Unit,
        },
    ]
}

#[must_use]
pub fn lookup(name: &str) -> Option<BuiltinSignature> {
    builtins()
        .into_iter()
        .find(|signature| signature.name == name)
}
