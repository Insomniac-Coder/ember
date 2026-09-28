//! `[CLO-3]` — a callable parameter is an implicit generic: each function
//! passed to it monomorphises the callee, and a call through it is a direct
//! call.
//!
//! A call that passes a known Ember function to a callable parameter (the
//! function named, or a lambda, or a local that only ever holds that one
//! function) goes to a copy of the callee made for that function, in which
//! every call through the parameter is a direct call to it: one the C compiler
//! can inline, as it cannot a call through a function pointer it does not see
//! the target of. The copy keeps the callee's signature, so the argument is
//! still passed and any other use of the parameter is unchanged. A copy that
//! passes the parameter on specialises the next callee the same way, so a
//! function forwarded through several calls, or to itself, is still called
//! directly; each copy is made once.

use std::collections::{HashMap, HashSet};

use ember_mir::{Body, Const, FuncRef, LocalId, Operand, Rvalue, StmtKind, Terminator};

/// Specialise every call that passes a known function to a callable
/// parameter; returns how many copies were made.
pub fn specialize_callable_arguments_all(bodies: &mut Vec<Body>) -> usize {
    // Ember functions a function value can call directly.
    let native: HashSet<String> = bodies
        .iter()
        .filter(|body| body.abi.is_none() && !body.is_extern_declaration && !body.is_abstract)
        .map(|body| body.symbol.clone())
        .collect();
    let mut index: HashMap<String, usize> =
        bodies.iter().enumerate().map(|(i, body)| (body.symbol.clone(), i)).collect();
    // A copy made for a callee and the functions its parameters hold.
    let mut copies: HashMap<(String, Vec<(usize, String)>), String> = HashMap::new();
    // The functions each body's parameters are known to hold (copies only).
    let mut known_params: HashMap<usize, Vec<(usize, String)>> = HashMap::new();
    let mut work: Vec<usize> = (0..bodies.len()).collect();
    let mut made = 0;
    while let Some(caller) = work.pop() {
        let held = known_functions(&bodies[caller], known_params.get(&caller), &native);
        for block in 0..bodies[caller].blocks.len() {
            let Terminator::Call { func: FuncRef::Direct { symbol, .. }, args, .. } = &bodies[caller].blocks[block].terminator
            else {
                continue;
            };
            let Some(&callee) = index.get(symbol) else { continue };
            if !native.contains(symbol) || bodies[callee].class_virtual_slot.is_some() {
                continue;
            }
            // Each argument that is a known function, at a parameter the
            // callee never reassigns and calls through or passes on.
            let known: Vec<(usize, String)> = args
                .iter()
                .enumerate()
                .filter_map(|(k, arg)| {
                    let function = match arg {
                        Operand::Const(Const::Fn(function)) if native.contains(function) => function.clone(),
                        Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => {
                            held.get(&place.local)?.clone()
                        }
                        _ => return None,
                    };
                    let param = LocalId(k as u32 + 1);
                    (k < bodies[callee].arg_count && calls_or_forwards(&bodies[callee], param)
                        && !reassigned(&bodies[callee], param))
                        .then_some((k, function))
                })
                .collect();
            if known.is_empty() {
                continue;
            }
            let key = (symbol.clone(), known.clone());
            let target = match copies.get(&key) {
                Some(target) => target.clone(),
                None => {
                    let mut copy = bodies[callee].clone();
                    let suffix: Vec<String> = known.iter().map(|(k, function)| format!("{k}_{function}")).collect();
                    copy.symbol = format!("{}__via_{}", bodies[callee].symbol, suffix.join("__"));
                    copy.emit_if_used = false;
                    direct_calls(&mut copy, &known);
                    let target = copy.symbol.clone();
                    bodies.push(copy);
                    let at = bodies.len() - 1;
                    index.insert(target.clone(), at);
                    known_params.insert(at, known.clone());
                    work.push(at);
                    copies.insert(key, target.clone());
                    made += 1;
                    target
                }
            };
            if let Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } = &mut bodies[caller].blocks[block].terminator {
                *symbol = target;
            }
        }
    }
    made
}

/// The locals of `body` that only ever hold one known Ember function: a
/// local assigned only that function's constant (never borrowed), or a
/// parameter a copy was made for.
fn known_functions(
    body: &Body,
    params: Option<&Vec<(usize, String)>>,
    native: &HashSet<String>,
) -> HashMap<LocalId, String> {
    let mut values: HashMap<LocalId, Option<String>> = HashMap::new();
    let mut borrowed = HashSet::new();
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if let Rvalue::Ref { place: target, .. } = rvalue {
                        borrowed.insert(target.local);
                    }
                    if place.projection.is_empty() {
                        let value = match rvalue {
                            Rvalue::Use(Operand::Const(Const::Fn(function))) if native.contains(function) => {
                                Some(function.clone())
                            }
                            _ => None,
                        };
                        let slot = values.entry(place.local).or_insert(value.clone());
                        if *slot != value {
                            *slot = None;
                        }
                    }
                }
                StmtKind::CheckedBinaryOp { dest, .. } if dest.projection.is_empty() => {
                    values.insert(dest.local, None);
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &block.terminator {
            values.insert(dest.local, None);
        }
    }
    let mut known: HashMap<LocalId, String> = values
        .into_iter()
        .filter(|(local, _)| local.0 as usize > body.arg_count && !borrowed.contains(local))
        .filter_map(|(local, value)| Some((local, value?)))
        .collect();
    for (k, function) in params.into_iter().flatten() {
        let param = LocalId(*k as u32 + 1);
        if !borrowed.contains(&param) {
            known.insert(param, function.clone());
        }
    }
    known
}

/// Whether `param` is called through, or passed whole to another call.
fn calls_or_forwards(body: &Body, param: LocalId) -> bool {
    let whole = |operand: &Operand| matches!(operand, Operand::Copy(place) | Operand::Move(place) if place.local == param && place.projection.is_empty());
    body.blocks.iter().any(|block| match &block.terminator {
        Terminator::Call { func: FuncRef::Indirect { operand, .. }, .. } if whole(operand) => true,
        Terminator::Call { func: FuncRef::Direct { .. }, args, .. } => args.iter().any(whole),
        _ => false,
    })
}

/// Whether anything in `body` writes `param` or borrows it.
fn reassigned(body: &Body, param: LocalId) -> bool {
    body.blocks.iter().any(|block| {
        block.stmts.iter().any(|stmt| match &stmt.kind {
            StmtKind::Assign { place, rvalue } => {
                place.local == param || matches!(rvalue, Rvalue::Ref { place, .. } if place.local == param)
            }
            StmtKind::CheckedBinaryOp { dest, .. } => dest.local == param,
            _ => false,
        }) || matches!(&block.terminator, Terminator::Call { dest, .. } if dest.local == param)
    })
}

/// In a copy, every call through a parameter that holds a known function
/// becomes a direct call to it.
fn direct_calls(body: &mut Body, known: &[(usize, String)]) {
    for block in &mut body.blocks {
        let Terminator::Call { func, .. } = &mut block.terminator else { continue };
        let FuncRef::Indirect { operand: Operand::Copy(place) | Operand::Move(place), .. } = func else { continue };
        if !place.projection.is_empty() {
            continue;
        }
        if let Some((_, function)) = known.iter().find(|(k, _)| LocalId(*k as u32 + 1) == place.local) {
            *func = FuncRef::Direct { symbol: function.clone(), latebound: false };
        }
    }
}
