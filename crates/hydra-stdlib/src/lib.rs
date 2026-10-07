use hydra_types::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinId {
    Print,
    Println,
}

impl BuiltinId {
    pub const ALL: [Self; 2] = [Self::Print, Self::Println];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Print => "print",
            Self::Println => "println",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuiltinParam {
    Exact(Type),
    Printable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinSignature {
    pub id: BuiltinId,
    pub name: &'static str,
    pub params: Vec<BuiltinParam>,
    pub return_type: Type,
}

#[must_use]
pub fn builtins() -> Vec<BuiltinSignature> {
    BuiltinId::ALL.into_iter().map(signature).collect()
}

#[must_use]
pub fn signature(id: BuiltinId) -> BuiltinSignature {
    BuiltinSignature {
        id,
        name: id.name(),
        params: vec![BuiltinParam::Printable],
        return_type: Type::Unit,
    }
}

#[must_use]
pub fn lookup(name: &str) -> Option<BuiltinSignature> {
    BuiltinId::ALL
        .into_iter()
        .find(|id| id.name() == name)
        .map(signature)
}

#[cfg(test)]
mod tests {
    use super::{BuiltinId, builtins, lookup, signature};
    use std::collections::BTreeSet;

    #[test]
    fn registry_covers_every_builtin_id_once_and_round_trips_names() {
        let registry = builtins();
        let ids: BTreeSet<_> = registry.iter().map(|entry| entry.id).collect();
        let names: BTreeSet<_> = registry.iter().map(|entry| entry.name).collect();

        assert_eq!(ids.len(), BuiltinId::ALL.len());
        assert_eq!(names.len(), BuiltinId::ALL.len());

        for id in BuiltinId::ALL {
            let entry = signature(id);
            assert_eq!(lookup(entry.name).map(|found| found.id), Some(id));
        }
    }
}
