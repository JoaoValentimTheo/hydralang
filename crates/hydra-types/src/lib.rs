use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Unit,
    Never,
    // The compiler can infer types from previously bound tuples. Sharing is
    // essential: repeated (t, t) bindings must not duplicate t's type tree.
    Tuple(Rc<[Type]>),
    List(Rc<Type>),
}

fn aggregate_key(ty: &Type) -> Option<(u8, usize)> {
    match ty {
        Type::Tuple(fields) => Some((0, Rc::as_ptr(fields) as *const () as usize)),
        Type::List(element) => Some((1, Rc::as_ptr(element) as usize)),
        _ => None,
    }
}

// Tuple type identity is structural, including field order and arity. Avoid
// recursively walking malformed internal type trees during comparison.
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        let mut pending = vec![(self, other)];
        let mut visited = HashSet::new();
        while let Some((left, right)) = pending.pop() {
            match (left, right) {
                (Self::Tuple(a), Self::Tuple(b)) if a.len() == b.len() => {
                    // Types contain no NaN-like values: matching shared nodes
                    // are equal. Visit each pair of distinct DAG nodes once.
                    if Rc::ptr_eq(a, b) {
                        continue;
                    }
                    let pair = (aggregate_key(left), aggregate_key(right));
                    if !visited.insert(pair) {
                        continue;
                    }
                    pending.extend(a.iter().zip(b.iter()));
                }
                (Self::List(a), Self::List(b)) => {
                    if Rc::ptr_eq(a, b) {
                        continue;
                    }
                    if visited.insert((aggregate_key(left), aggregate_key(right))) {
                        pending.push((a, b));
                    }
                }
                (Self::Int, Self::Int)
                | (Self::Float, Self::Float)
                | (Self::Bool, Self::Bool)
                | (Self::String, Self::String)
                | (Self::Unit, Self::Unit)
                | (Self::Never, Self::Never) => {}
                _ => return false,
            }
        }
        true
    }
}

impl Eq for Type {}

// The hash must follow the same structural identity without host recursion.
impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash a structural fingerprint bottom-up. Caching by shared tuple
        // storage avoids traversing logical DAG expansions exponentially; the
        // result is independent of the amount of physical sharing.
        let mut fingerprints = HashMap::new();
        let mut scheduled = HashSet::new();
        let mut pending = vec![(self, false)];
        while let Some((ty, finish)) = pending.pop() {
            if let Some(key) = aggregate_key(ty) {
                if fingerprints.contains_key(&key) {
                    continue;
                }
                if finish {
                    let mut hasher = DefaultHasher::new();
                    std::mem::discriminant(ty).hash(&mut hasher);
                    match ty {
                        Self::Tuple(fields) => {
                            fields.len().hash(&mut hasher);
                            for field in fields.iter() {
                                field_fingerprint(field, &fingerprints).hash(&mut hasher);
                            }
                        }
                        Self::List(element) => {
                            field_fingerprint(element, &fingerprints).hash(&mut hasher)
                        }
                        _ => unreachable!(),
                    }
                    fingerprints.insert(key, hasher.finish());
                } else if scheduled.insert(key) {
                    pending.push((ty, true));
                    match ty {
                        Self::Tuple(fields) => {
                            pending.extend(fields.iter().rev().map(|field| (field, false)))
                        }
                        Self::List(element) => pending.push((element, false)),
                        _ => unreachable!(),
                    }
                }
            }
        }
        field_fingerprint(self, &fingerprints).hash(state);
    }
}

fn field_fingerprint(ty: &Type, fingerprints: &HashMap<(u8, usize), u64>) -> u64 {
    if let Some(key) = aggregate_key(ty) {
        fingerprints[&key]
    } else {
        let mut hasher = DefaultHasher::new();
        std::mem::discriminant(ty).hash(&mut hasher);
        hasher.finish()
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Type {
    #[must_use]
    pub fn tuple(fields: Vec<Type>) -> Self {
        Self::Tuple(Rc::from(fields))
    }

    #[must_use]
    pub fn list(element: Type) -> Self {
        Self::List(Rc::new(element))
    }

    /// Maximum number of tuple layers along any structural path. Shared
    /// subtrees are visited only once, so `(t, t)` chains stay linear.
    #[must_use]
    pub fn tuple_depth(&self) -> usize {
        self.aggregate_depth()
    }

    /// Longest combined List/Tuple structural path, memoized by shared node.
    #[must_use]
    pub fn aggregate_depth(&self) -> usize {
        let mut depths = HashMap::<(u8, usize), usize>::new();
        let mut scheduled = HashSet::new();
        let mut pending = vec![(self, false)];
        while let Some((ty, finishing)) = pending.pop() {
            let Some(key) = aggregate_key(ty) else {
                continue;
            };
            if depths.contains_key(&key) {
                continue;
            }
            if finishing {
                let child_depth = match ty {
                    Self::Tuple(fields) => fields
                        .iter()
                        .filter_map(aggregate_key)
                        .map(|key| depths[&key])
                        .max()
                        .unwrap_or(0),
                    Self::List(element) => aggregate_key(element).map_or(0, |key| depths[&key]),
                    _ => 0,
                };
                depths.insert(key, child_depth.saturating_add(1));
            } else if scheduled.insert(key) {
                pending.push((ty, true));
                match ty {
                    Self::Tuple(fields) => {
                        pending.extend(fields.iter().rev().map(|field| (field, false)))
                    }
                    Self::List(element) => pending.push((element, false)),
                    _ => unreachable!(),
                }
            }
        }
        aggregate_key(self).map_or(0, |key| depths[&key])
    }

    #[must_use]
    pub fn contains_list(&self) -> bool {
        let mut pending = vec![self];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            let Some(key) = aggregate_key(ty) else {
                continue;
            };
            if !seen.insert(key) {
                continue;
            }
            match ty {
                Self::List(_) => return true,
                Self::Tuple(fields) => pending.extend(fields.iter()),
                _ => {}
            }
        }
        false
    }

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
        self.fmt_bounded(f, 0, &mut 2048)
    }
}

impl Type {
    // Source types contain at most 64 tuple layers. Internal malformed types
    // must not recurse indefinitely when rendered for diagnostics.
    fn fmt_bounded(
        &self,
        f: &mut fmt::Formatter<'_>,
        depth: usize,
        remaining: &mut usize,
    ) -> fmt::Result {
        if *remaining == 0 {
            return f.write_str("...");
        }
        *remaining -= 1;
        match self {
            Self::Int => f.write_str("Int"),
            Self::Float => f.write_str("Float"),
            Self::Bool => f.write_str("Bool"),
            Self::String => f.write_str("String"),
            Self::Unit => f.write_str("Unit"),
            Self::Never => f.write_str("Never"),
            Self::List(element) => {
                if depth >= 64 {
                    return f.write_str("<invalid List type>");
                }
                f.write_str("List<")?;
                element.fmt_bounded(f, depth + 1, remaining)?;
                f.write_str(">")
            }
            Self::Tuple(fields) => {
                if depth >= 64 || fields.is_empty() || fields.len() > 64 {
                    return f.write_str("<invalid tuple type>");
                }
                f.write_str("(")?;
                for (index, ty) in fields.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    if *remaining == 0 {
                        f.write_str("...")?;
                        break;
                    }
                    ty.fmt_bounded(f, depth + 1, remaining)?;
                }
                if fields.len() == 1 {
                    f.write_str(",")?;
                }
                f.write_str(")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Type;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::rc::Rc;

    #[test]
    fn tuple_depth_uses_longest_path_and_handles_shared_internal_dags() {
        assert_eq!(Type::Int.tuple_depth(), 0);
        assert_eq!(Type::Never.tuple_depth(), 0);
        let mut ty = Type::Int;
        for expected in 1..=1024 {
            ty = Type::tuple(vec![Type::Bool, ty.clone(), ty]);
            assert_eq!(ty.tuple_depth(), expected);
        }
        assert_eq!(Type::tuple(vec![Type::Int, Type::Bool]).tuple_depth(), 1);
    }

    #[test]
    fn inferred_tuple_type_dag_shares_storage_and_preserves_structural_hashing() {
        // Forty binary expansions represent over a trillion logical leaves;
        // physical nodes and comparison/hash work must stay bounded.
        let mut shared = Type::tuple(vec![Type::Int]);
        let mut independently_built = Type::tuple(vec![Type::Int]);
        for _ in 0..40 {
            let previous = shared.clone();
            shared = Type::tuple(vec![previous.clone(), previous]);
            let previous = independently_built.clone();
            independently_built = Type::tuple(vec![previous.clone(), previous]);
        }
        let Type::Tuple(fields) = &shared else {
            unreachable!();
        };
        let (Type::Tuple(left), Type::Tuple(right)) = (&fields[0], &fields[1]) else {
            unreachable!();
        };
        assert!(Rc::ptr_eq(left, right));
        assert_eq!(shared, independently_built);

        let mut shared_hash = DefaultHasher::new();
        let mut independent_hash = DefaultHasher::new();
        shared.hash(&mut shared_hash);
        independently_built.hash(&mut independent_hash);
        assert_eq!(shared_hash.finish(), independent_hash.finish());
        assert!(shared.to_string().len() < 10_000);

        let mut unequal = Type::tuple(vec![Type::Bool]);
        for _ in 0..40 {
            unequal = Type::tuple(vec![unequal.clone(), unequal]);
        }
        assert_ne!(shared, unequal);
    }

    #[test]
    fn tuple_type_identity_and_hash_handle_deep_internal_structures() {
        let (mut left, mut right, mut different) = (Type::Int, Type::Int, Type::Bool);
        for _ in 0..1024 {
            left = Type::tuple(vec![left]);
            right = Type::tuple(vec![right]);
            different = Type::tuple(vec![different]);
        }
        assert!(left == right);
        assert!(left != different);

        let mut left_hash = DefaultHasher::new();
        let mut right_hash = DefaultHasher::new();
        left.hash(&mut left_hash);
        right.hash(&mut right_hash);
        assert_eq!(left_hash.finish(), right_hash.finish());
        assert_ne!(Type::tuple(vec![Type::Int]), Type::Int);
        assert_ne!(
            Type::tuple(vec![Type::Int, Type::Bool]),
            Type::tuple(vec![Type::Bool, Type::Int])
        );
    }

    #[test]
    fn tuple_type_display_is_bounded_for_malformed_internal_nesting() {
        let mut ty = Type::Int;
        for _ in 0..64 {
            ty = Type::tuple(vec![ty]);
        }
        let valid = ty.to_string();
        assert!(!valid.contains("<invalid tuple type>"));
        assert!(valid.contains("Int"));

        let malformed = Type::tuple(vec![ty]);
        let rendered = malformed.to_string();
        assert!(rendered.contains("<invalid tuple type>"));
        assert!(!rendered.contains("Int"));
        assert_eq!(Type::tuple(vec![]).to_string(), "<invalid tuple type>");
        assert_eq!(
            Type::tuple(vec![Type::Int; 65]).to_string(),
            "<invalid tuple type>"
        );
    }
}
