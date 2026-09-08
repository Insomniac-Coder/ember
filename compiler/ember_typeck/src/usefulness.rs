//! Exhaustiveness and redundancy for `match` (`[ENM-2]`, `[CTL-5]`).
//!
//! Maranget's usefulness algorithm: a pattern is *useful* with respect to the
//! patterns above it when some value matches it and none of them. An arm that
//! is not useful can never run (`W2091`), and a `match` is exhaustive exactly
//! when a bare `_` would not be useful after every arm.
//!
//! The same walk produces the witnesses `E2090` reports, so "not covered" and
//! "here is what is not covered" cannot disagree.

use ember_hir::{Pattern, PatternKind};
use ember_types::{Ty, TyKind, TypeTable};

/// How many missing values `E2090` lists before it stops.
const MAX_WITNESSES: usize = 8;

/// What a pattern tests for at its outermost level.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Ctor {
    Variant(usize),
    /// An integer, `bool` or `char` value, already narrowed.
    Int(i128),
    /// A type with exactly one constructor: a tuple, a struct or an array.
    Single,
}

/// A pattern reduced to what matching cares about. Bindings are dropped — a
/// binding matches everything — and every sub-pattern is owned, which keeps
/// specialisation from needing a wildcard to point at.
#[derive(Clone, Debug)]
enum Pat {
    Wild,
    Ctor { ctor: Ctor, fields: Vec<Pat> },
    Or(Vec<Pat>),
}

fn lower(pattern: &Pattern) -> Pat {
    match &pattern.kind {
        // An `Error` pattern absorbs: it must not produce a second diagnostic
        // about coverage on top of the one that made it.
        PatternKind::Wild | PatternKind::Error => Pat::Wild,
        PatternKind::Bind { sub: None, .. } => Pat::Wild,
        PatternKind::Bind { sub: Some(sub), .. } => lower(sub),
        PatternKind::Int(value) => Pat::Ctor { ctor: Ctor::Int(*value), fields: Vec::new() },
        PatternKind::Variant { variant, fields, .. } => Pat::Ctor {
            ctor: Ctor::Variant(*variant),
            fields: fields.iter().map(lower).collect(),
        },
        PatternKind::Fields(items) => {
            Pat::Ctor { ctor: Ctor::Single, fields: items.iter().map(lower).collect() }
        }
        PatternKind::Or(alternatives) => Pat::Or(alternatives.iter().map(lower).collect()),
    }
}

/// Whether `pattern` matches anything the patterns in `seen` do not.
pub fn is_useful(types: &TypeTable, seen: &[&Pattern], pattern: &Pattern, ty: Ty) -> bool {
    let matrix: Vec<Vec<Pat>> = seen.iter().map(|p| vec![lower(p)]).collect();
    useful(types, &matrix, &[lower(pattern)], &[ty])
}

/// The values `seen` leaves uncovered, rendered for a diagnostic. Empty when
/// the arms are exhaustive.
pub fn missing_patterns(types: &TypeTable, seen: &[&Pattern], ty: Ty) -> Vec<String> {
    let matrix: Vec<Vec<Pat>> = seen.iter().map(|p| vec![lower(p)]).collect();
    let mut out: Vec<String> = witnesses(types, &matrix, &[ty])
        .into_iter()
        .map(|mut row| row.pop().unwrap_or_else(|| "_".to_string()))
        .collect();
    out.truncate(MAX_WITNESSES);
    out
}

fn useful(types: &TypeTable, matrix: &[Vec<Pat>], row: &[Pat], tys: &[Ty]) -> bool {
    let Some((head, rest)) = row.split_first() else {
        // Every column has been consumed: this row is useful only if nothing
        // above it matched all the way down.
        return matrix.is_empty();
    };
    let head_ty = tys[0];
    match head {
        Pat::Or(alternatives) => alternatives.iter().any(|alternative| {
            let mut row = vec![alternative.clone()];
            row.extend_from_slice(rest);
            useful(types, matrix, &row, tys)
        }),
        Pat::Ctor { ctor, fields } => {
            let field_tys = ctor_fields(types, head_ty, ctor);
            let arity = field_tys.len();
            let mut row = fields.clone();
            row.extend_from_slice(rest);
            let mut sub_tys = field_tys;
            sub_tys.extend_from_slice(&tys[1..]);
            useful(types, &specialize(matrix, ctor, arity), &row, &sub_tys)
        }
        Pat::Wild => {
            let used = used_ctors(matrix);
            let complete = match all_ctors(types, head_ty) {
                Some(all) => all.iter().all(|c| used.contains(c)).then_some(all),
                None => None,
            };
            match complete {
                // Every constructor is accounted for above, so a wildcard is
                // useful only where one of them still leaves something open.
                Some(all) => all.iter().any(|ctor| {
                    let field_tys = ctor_fields(types, head_ty, ctor);
                    let mut row: Vec<Pat> = vec![Pat::Wild; field_tys.len()];
                    row.extend_from_slice(rest);
                    let mut sub_tys = field_tys.clone();
                    sub_tys.extend_from_slice(&tys[1..]);
                    useful(types, &specialize(matrix, ctor, field_tys.len()), &row, &sub_tys)
                }),
                None => useful(types, &default_matrix(matrix), rest, &tys[1..]),
            }
        }
    }
}

fn witnesses(types: &TypeTable, matrix: &[Vec<Pat>], tys: &[Ty]) -> Vec<Vec<String>> {
    if tys.is_empty() {
        return if matrix.is_empty() { vec![Vec::new()] } else { Vec::new() };
    }
    let head_ty = tys[0];
    let used = used_ctors(matrix);
    let all = all_ctors(types, head_ty);
    let complete = matches!(&all, Some(all) if all.iter().all(|c| used.contains(c)));

    let mut out = Vec::new();
    if complete {
        for ctor in all.expect("complete implies a known set") {
            let field_tys = ctor_fields(types, head_ty, &ctor);
            let arity = field_tys.len();
            let mut sub_tys = field_tys;
            sub_tys.extend_from_slice(&tys[1..]);
            for witness in witnesses(types, &specialize(matrix, &ctor, arity), &sub_tys) {
                out.push(rebuild(types, head_ty, &ctor, arity, witness));
                if out.len() >= MAX_WITNESSES {
                    return out;
                }
            }
        }
        return out;
    }

    // Some constructor is missing. Name the missing ones where the set is
    // known — that is what `[ENM-2]` asks `E2090` to list — and fall back to
    // `_` for types with too many values to enumerate.
    let heads: Vec<String> = match all {
        Some(all) => all
            .iter()
            .filter(|c| !used.contains(c))
            .map(|c| render(types, head_ty, c, &vec!["_".to_string(); ctor_fields(types, head_ty, c).len()]))
            .collect(),
        None => vec!["_".to_string()],
    };
    for witness in witnesses(types, &default_matrix(matrix), &tys[1..]) {
        for head in &heads {
            let mut row = vec![head.clone()];
            row.extend_from_slice(&witness);
            out.push(row);
            if out.len() >= MAX_WITNESSES {
                return out;
            }
        }
    }
    out
}

/// Put a constructor back together around the first `arity` witnesses.
fn rebuild(
    types: &TypeTable,
    ty: Ty,
    ctor: &Ctor,
    arity: usize,
    witness: Vec<String>,
) -> Vec<String> {
    let rest = witness[arity.min(witness.len())..].to_vec();
    let fields = witness[..arity.min(witness.len())].to_vec();
    let mut row = vec![render(types, ty, ctor, &fields)];
    row.extend(rest);
    row
}

fn render(types: &TypeTable, ty: Ty, ctor: &Ctor, fields: &[String]) -> String {
    match ctor {
        Ctor::Int(value) => {
            if matches!(types.kind(ty), TyKind::Bool) {
                if *value == 0 { "false".to_string() } else { "true".to_string() }
            } else {
                value.to_string()
            }
        }
        Ctor::Variant(index) => {
            let TyKind::Enum(id) = types.kind(ty) else { return "_".to_string() };
            let def = types.enum_def(*id);
            let name = format!("{}.{}", def.name, def.variants[*index].name);
            if fields.is_empty() { name } else { format!("{name}({})", fields.join(", ")) }
        }
        Ctor::Single => match types.kind(ty) {
            TyKind::Tuple(_) => format!("({})", fields.join(", ")),
            TyKind::Array { .. } => format!("[{}]", fields.join(", ")),
            TyKind::Struct(id) => {
                format!("{}({})", types.struct_def(*id).name, fields.join(", "))
            }
            _ => "_".to_string(),
        },
    }
}

/// The sub-patterns a constructor exposes, by type. Its length is the arity.
fn ctor_fields(types: &TypeTable, ty: Ty, ctor: &Ctor) -> Vec<Ty> {
    match ctor {
        Ctor::Int(_) => Vec::new(),
        Ctor::Variant(index) => match types.kind(ty) {
            TyKind::Enum(id) => types.enum_def(*id).variants[*index]
                .fields
                .iter()
                .map(|f| f.ty)
                .collect(),
            _ => Vec::new(),
        },
        Ctor::Single => match types.kind(ty) {
            TyKind::Tuple(items) => items.clone(),
            TyKind::Struct(id) => types.struct_def(*id).fields.iter().map(|f| f.ty).collect(),
            TyKind::Array { elem, len } => vec![*elem; *len as usize],
            _ => Vec::new(),
        },
    }
}

/// Every constructor of a type, when there are few enough to enumerate. An
/// integer or a float has too many, so a `match` on one is exhaustive only
/// through a wildcard.
fn all_ctors(types: &TypeTable, ty: Ty) -> Option<Vec<Ctor>> {
    match types.kind(ty) {
        TyKind::Enum(id) => {
            Some((0..types.enum_def(*id).variants.len()).map(Ctor::Variant).collect())
        }
        TyKind::Bool => Some(vec![Ctor::Int(0), Ctor::Int(1)]),
        TyKind::Tuple(_) | TyKind::Struct(_) | TyKind::Array { .. } => Some(vec![Ctor::Single]),
        _ => None,
    }
}

fn used_ctors(matrix: &[Vec<Pat>]) -> Vec<Ctor> {
    let mut out: Vec<Ctor> = Vec::new();
    fn walk(pat: &Pat, out: &mut Vec<Ctor>) {
        match pat {
            Pat::Ctor { ctor, .. } => {
                if !out.contains(ctor) {
                    out.push(ctor.clone());
                }
            }
            Pat::Or(alternatives) => alternatives.iter().for_each(|a| walk(a, out)),
            Pat::Wild => {}
        }
    }
    for row in matrix {
        if let Some(head) = row.first() {
            walk(head, &mut out);
        }
    }
    out
}

/// `S(c, P)` — the rows that can still match once the first column is known
/// to be `ctor`, with that column replaced by the constructor's fields.
fn specialize(matrix: &[Vec<Pat>], ctor: &Ctor, arity: usize) -> Vec<Vec<Pat>> {
    let mut out = Vec::new();
    for row in matrix {
        let Some((head, rest)) = row.split_first() else { continue };
        push_specialized(head, rest, ctor, arity, &mut out);
    }
    out
}

fn push_specialized(
    head: &Pat,
    rest: &[Pat],
    ctor: &Ctor,
    arity: usize,
    out: &mut Vec<Vec<Pat>>,
) {
    match head {
        Pat::Wild => {
            let mut row = vec![Pat::Wild; arity];
            row.extend_from_slice(rest);
            out.push(row);
        }
        Pat::Ctor { ctor: found, fields } if found == ctor => {
            let mut row = fields.clone();
            row.extend_from_slice(rest);
            out.push(row);
        }
        Pat::Ctor { .. } => {}
        Pat::Or(alternatives) => {
            for alternative in alternatives {
                push_specialized(alternative, rest, ctor, arity, out);
            }
        }
    }
}

/// `D(P)` — the rows that match when the first column is a constructor none
/// of the rows tested for.
fn default_matrix(matrix: &[Vec<Pat>]) -> Vec<Vec<Pat>> {
    let mut out = Vec::new();
    for row in matrix {
        let Some((head, rest)) = row.split_first() else { continue };
        push_default(head, rest, &mut out);
    }
    out
}

fn push_default(head: &Pat, rest: &[Pat], out: &mut Vec<Vec<Pat>>) {
    match head {
        Pat::Wild => out.push(rest.to_vec()),
        Pat::Ctor { .. } => {}
        Pat::Or(alternatives) => {
            for alternative in alternatives {
                push_default(alternative, rest, out);
            }
        }
    }
}
