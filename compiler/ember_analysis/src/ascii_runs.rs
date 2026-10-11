//! D-498 (ADR-155) — an ASCII byte steps around the UTF-8 decoder.
//!
//! A loop that walks a `str` with `next_char` first tests the byte the decoder
//! would read: one below 0x80 is the character itself and moves the cursor one,
//! inside a copy of the loop that comes back to the same test, so a run of ASCII
//! text stays in a tight inner loop. Every other byte goes to the decoder as
//! before. No check is removed and no new memory is read: the decoder reads
//! that same byte at that same cursor first. For MSVC, a byte the test sends
//! on is handed to the decoder (`StrCharNextWide`), which MSVC otherwise read
//! and tested again (measured: Chinese text 1.09x, then 0.93x of the old loop);
//! for gcc, eight written-out ASCII steps go in front of the inner loop.
use crate::kernels::KernelTarget;
use crate::loop_version::retarget;
use crate::range_facts::{loop_headers, natural_loops, reverse_postorder};
use crate::strength_reduce::{stmt_reads, terminator_reads};
use ember_mir::{
    BasicBlock, BasicBlockId, BinOp, Body, Builtin, CastKind, Const, FuncRef, LocalDecl, LocalId,
    LocalKind, Operand, Place, Projection, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_types::{CommonTypes, Ty, TyKind, TypeTable};
use std::collections::BTreeMap;

/// The copied loop doubles in size; a larger loop is not a short scan.
const MAX_LOOP_BLOCKS: usize = 64;
const MAX_BODY_BLOCKS: usize = 4096;
/// gcc's written-out steps (see `apply`), each a copy of the loop.
const GCC_PREFIX_STEPS: usize = 8;
const MAX_PREFIX_REGION: usize = 8;

struct Site {
    decode: usize,
    /// The loop's blocks other than `decode`, in order: each gets a copy.
    region: Vec<usize>,
    view: LocalId,
    cursor: LocalId,
    reference: LocalId,
    /// The reference the call takes, when nothing else reads it: the ASCII
    /// step leaves it unset, so C keeps the cursor in a register.
    unread_reference: Option<LocalId>,
    character: LocalId,
    next: usize,
}

/// Gives every matching decoder call its ASCII path; returns how many did.
/// `cold` names loops, by body index and header, left as they are: the
/// progress reduction's checked fallbacks, where the copy only costs (measured
/// on MSVC: a call on empty text ×1.27, as the fallback takes that case).
/// `target` shapes it for one C compiler: MSVC's decoder takes the tested
/// byte, gcc's loop gets written-out steps in front (see `apply`).
pub fn step_ascii_runs_all(
    bodies: &mut [Body],
    types: &mut TypeTable,
    common: &CommonTypes,
    cold: &[(usize, usize)],
    target: Option<KernelTarget>,
) -> usize {
    let bytes_ty = types.intern(TyKind::Span { elem: common.u8, mutable: false });
    let mut count = 0;
    for (index, body) in bodies.iter_mut().enumerate() {
        // As the progress reduction: these name blocks or loops a copy would break.
        if !body.call_argument_bindings.is_empty() || !body.hoisted_accesses.is_empty() {
            continue;
        }
        let decodes: Vec<usize> = (0..body.blocks.len()).filter(|&b| is_decode(&body.blocks[b].terminator)).collect();
        if decodes.is_empty() {
            continue;
        }
        // Innermost loop first: an enclosing loop's copy then copies the done inner one.
        let mut ordered: Vec<(usize, usize)> =
            decodes.iter().filter_map(|&d| innermost_loop(body, d).map(|(_, l)| (l.len(), d))).collect();
        ordered.sort_unstable();
        for (_, decode) in ordered {
            let Some((header, region)) = innermost_loop(body, decode) else { continue };
            if cold.contains(&(index, header)) {
                continue;
            }
            if let Some(site) = plan(body, common, decode, region) {
                if body.blocks.len() + (prefix_steps(&site, target) + 1) * stage_size(&site) <= MAX_BODY_BLOCKS {
                    apply(body, common, bytes_ty, &site, target);
                    count += 1;
                }
            }
        }
    }
    count
}

fn is_decode(t: &Terminator) -> bool {
    matches!(t, Terminator::Call { func: FuncRef::Builtin { which: Builtin::StrCharNext, .. }, .. })
}

/// The header and blocks of the smallest loop holding `block`.
fn innermost_loop(body: &Body, block: usize) -> Option<(usize, Vec<usize>)> {
    let order = reverse_postorder(body);
    let headers = loop_headers(body, &order);
    let (header, region) =
        natural_loops(body, &order, &headers).into_iter().filter(|(_, l)| l.contains(&block)).min_by_key(|(_, l)| l.len())?;
    let mut region: Vec<usize> = region.into_iter().collect();
    region.sort_unstable();
    Some((header, region))
}

fn plain(place: &Place) -> Option<LocalId> {
    place.projection.is_empty().then_some(place.local)
}

fn plan(body: &Body, common: &CommonTypes, decode: usize, loop_blocks: Vec<usize>) -> Option<Site> {
    if loop_blocks.len() > MAX_LOOP_BLOCKS {
        return None;
    }
    let block = &body.blocks[decode];
    let Terminator::Call { func: FuncRef::Builtin { arg_ty, .. }, args, dest, next } = &block.terminator else {
        return None;
    };
    let character = plain(dest)?;
    if *arg_ty != common.str_ || args.len() != 2 || body.local(character).ty != common.char_ {
        return None;
    }
    let (Operand::Copy(view), Operand::Copy(reference)) = (&args[0], &args[1]) else { return None };
    let (view, reference) = (plain(view)?, plain(reference)?);
    // The cursor is what the reference holds at the call: its last write in this block.
    let cursor = block.stmts.iter().rev().find_map(|s| match &s.kind {
        StmtKind::Assign { place, rvalue } if plain(place) == Some(reference) => Some(match rvalue {
            Rvalue::Ref { place, mutable: true } => plain(place),
            _ => None,
        }),
        _ => None,
    })??;
    if body.local(cursor).ty != common.usize || body.local(view).ty != common.str_ {
        return None;
    }
    // The test runs before this block's statements, so they may write only
    // locals the test does not read, and no memory.
    let read_by_test = |l: LocalId| l == cursor || l == view;
    for s in &block.stmts {
        let written = match &s.kind {
            StmtKind::Nop => continue,
            StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => *l,
            StmtKind::Assign { place, .. } => plain(place)?,
            StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                if read_by_test(plain(overflow)?) {
                    return None;
                }
                plain(dest)?
            }
            _ => return None,
        };
        if read_by_test(written) {
            return None;
        }
    }
    // Unread past its block: only a call right after a fresh assignment reads it
    // (the progress reduction's copy of this call sets it again).
    let mut read = vec![false; body.locals.len()];
    let r = reference.0 as usize;
    let unread = body.blocks.iter().all(|other| {
        other.stmts.iter().for_each(|s| stmt_reads(s, &mut read));
        let in_stmts = std::mem::take(&mut read[r]);
        terminator_reads(&other.terminator, &mut read);
        let in_call = std::mem::take(&mut read[r]);
        !in_stmts
            && (!in_call
                || other.stmts.iter().any(|s| matches!(&s.kind, StmtKind::Assign { place, .. } if plain(place) == Some(reference))))
    });
    Some(Site {
        decode,
        unread_reference: unread.then_some(reference),
        reference,
        region: loop_blocks.into_iter().filter(|&b| b != decode).collect(),
        view,
        cursor,
        character,
        next: next.0 as usize,
    })
}

fn temp(body: &mut Body, ty: Ty, span: ember_span::Span) -> LocalId {
    body.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span });
    LocalId(body.locals.len() as u32 - 1)
}

/// One stage is `read` and `test` (the byte, then the branch on it), `step`
/// (the ASCII step, into the copied body) and the copies, whose way back to
/// the decoder goes to `back`. Stage 0 is the inner loop: its copy comes back
/// to its own test, which C compilers then take as the inner loop's header
/// (measured: clang lays out a test copied to the bottom of the inner loop with
/// two more instructions per character). For gcc, stages 1 to 8 are written-out
/// steps in front of it, each going on to the next and the last into the loop:
/// gcc unrolls the inner loop 8 times and enters it through a branch on the
/// rest's length modulo 8 that mispredicts, so a wide character or a short run
/// now never reaches it (measured: Chinese text 1.10x, then 1.00x).
fn apply(body: &mut Body, common: &CommonTypes, bytes_ty: Ty, site: &Site, target: Option<KernelTarget>) {
    let span = body.blocks[site.decode].terminator_span;
    let temps = Temps {
        bytes: temp(body, bytes_ty, span),
        byte: temp(body, common.u8, span),
        ascii: temp(body, common.bool_, span),
    };
    let first = body.blocks.len();
    let size = stage_size(site);
    let prefix = prefix_steps(site, target);
    let read_of = |stage: usize| first + stage * size;
    let mut stages = Vec::new();
    for stage in 0..=prefix {
        let back = if stage == 0 || stage == prefix { read_of(0) } else { read_of(stage + 1) };
        stages.extend(stage_blocks(body, common, site, &temps, span, read_of(stage), back));
    }
    // The loop's own way into the decoder now passes the first test.
    let entry = read_of(if prefix > 0 { 1 } else { 0 });
    for &b in &site.region {
        retarget(&mut body.blocks[b].terminator, |t| if t.0 as usize == site.decode { BasicBlockId(entry as u32) } else { t });
    }
    body.blocks.extend(stages);
    // The decoder's block, once only the tests reach it, decodes a lead byte
    // a test read and found 0x80 or above (its statements write neither).
    let is_test = |b: usize| b > first && (b - first) % size == 1;
    if target == Some(KernelTarget::Msvc) && crate::loop_version::predecessors(body)[site.decode].iter().all(|&b| is_test(b)) {
        let Terminator::Call { func: FuncRef::Builtin { which, .. }, args, .. } = &mut body.blocks[site.decode].terminator else {
            unreachable!("planned on a decoder call")
        };
        *which = Builtin::StrCharNextWide;
        *args = vec![
            Operand::Copy(Place::local(site.view)),
            Operand::Copy(Place::local(site.reference)),
            Operand::Copy(Place::local(temps.byte)),
        ];
    }
}

/// The written-out steps: gcc's only, and for a loop small enough to copy 8 times.
fn prefix_steps(site: &Site, target: Option<KernelTarget>) -> usize {
    if target == Some(KernelTarget::Gcc) && site.region.len() <= MAX_PREFIX_REGION { GCC_PREFIX_STEPS } else { 0 }
}

fn stage_size(site: &Site) -> usize {
    3 + site.region.len()
}

struct Temps {
    bytes: LocalId,
    byte: LocalId,
    ascii: LocalId,
}

/// One stage's blocks, the first of them at index `at`.
fn stage_blocks(body: &Body, common: &CommonTypes, site: &Site, t: &Temps, span: ember_span::Span, at: usize, back: usize) -> Vec<BasicBlock> {
    let (test, step) = (at + 1, at + 2);
    let copies: BTreeMap<usize, usize> = site.region.iter().enumerate().map(|(i, &b)| (b, at + 3 + i)).collect();
    let id = |b: usize| BasicBlockId(b as u32);
    let in_copy = |b: BasicBlockId| {
        let b = b.0 as usize;
        id(if b == site.decode { back } else { copies.get(&b).copied().unwrap_or(b) })
    };
    let assign = |place: Place, rvalue: Rvalue| Stmt::new(StmtKind::Assign { place, rvalue }, span);
    let read_block = BasicBlock {
        stmts: Vec::new(),
        terminator: Terminator::Call {
            func: FuncRef::Builtin { which: Builtin::StrAsBytes, arg_ty: common.str_ },
            args: vec![Operand::Copy(Place::local(site.view))],
            dest: Place::local(t.bytes),
            next: id(test),
        },
        terminator_span: span,
    };
    let test_block = BasicBlock {
        stmts: vec![
            assign(
                Place::local(t.byte),
                Rvalue::Use(Operand::Copy(Place { local: t.bytes, projection: vec![Projection::Index(site.cursor)] })),
            ),
            assign(
                Place::local(t.ascii),
                Rvalue::BinaryOp {
                    op: BinOp::Lt,
                    lhs: Operand::Copy(Place::local(t.byte)),
                    rhs: Operand::Const(Const::Int { value: 0x80, ty: common.u8 }),
                },
            ),
        ],
        terminator: Terminator::SwitchInt {
            discr: Operand::Copy(Place::local(t.ascii)),
            targets: vec![(0, id(site.decode))],
            otherwise: id(step),
        },
        terminator_span: span,
    };
    let mut step_stmts = body.blocks[site.decode].stmts.clone();
    step_stmts.retain(|s| !matches!(&s.kind, StmtKind::Assign { place, .. } if site.unread_reference.is_some_and(|r| plain(place) == Some(r))));
    step_stmts.push(assign(
        Place::local(site.character),
        Rvalue::Cast { kind: CastKind::Numeric, operand: Operand::Copy(Place::local(t.byte)), to: common.char_ },
    ));
    step_stmts.push(assign(
        Place::local(site.cursor),
        Rvalue::BinaryOp {
            op: BinOp::Add,
            lhs: Operand::Copy(Place::local(site.cursor)),
            rhs: Operand::Const(Const::Int { value: 1, ty: common.usize }),
        },
    ));
    let step_block = BasicBlock { stmts: step_stmts, terminator: Terminator::Goto(in_copy(id(site.next))), terminator_span: span };
    let mut blocks = vec![read_block, test_block, step_block];
    for &b in &site.region {
        let mut block = body.blocks[b].clone();
        retarget(&mut block.terminator, &in_copy);
        blocks.push(block);
    }
    blocks
}
