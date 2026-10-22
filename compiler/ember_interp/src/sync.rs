//! `[THR-3]`, `[THR-14]` — locks and atomics on the one thread the evaluator runs. A lock's word
//! is the runtime's single-thread form: a `Mutex`'s 1 while held; an `RwLock`'s writer bit or its
//! count of readers. Taking a lock its own thread holds can only wait for ever, as the compiled
//! program would; the evaluator stops and says so. An atomic operation is the plain operation.

use ember_hir::{AtomicOp, MemoryOrder};
use ember_mir::{BinOp, Operand};
use ember_span::Span;
use ember_types::{Ty, TyKind};

use crate::scalar::IntKind;
use crate::{fault, Interp, R};

const WRITER: u32 = 0x8000_0000;
const READERS: u32 = 0x3FFF_FFFF;

impl Interp<'_, '_> {
    fn waits_for_ever<T>(&mut self, what: &str) -> R<T> {
        fault(format!("{what} its own thread holds, which waits for ever: the evaluator runs one thread"))
    }

    /// `ember_mutex_lock` / `ember_rwlock_acquire`, and their `try` forms (`false` when held).
    pub(crate) fn lock(&mut self, word: u64, write: bool, rwlock: bool, trying: bool) -> R<bool> {
        let value = self.mem.read_u32(word)?;
        let free = if !rwlock || write { value == 0 } else { value & WRITER == 0 && value & READERS < READERS };
        if !free {
            if trying {
                return Ok(false);
            }
            return self.waits_for_ever(if rwlock { "a lock of an `RwLock`" } else { "a lock of a `Mutex`" });
        }
        let taken = if !rwlock { 1 } else if write { WRITER } else { value + 1 };
        self.mem.write_u32(word, taken)?;
        Ok(true)
    }

    /// A guard's end: the lock it was taken from, whose value the guard points at (its first
    /// field, so the lock's own address), freed.
    pub(crate) fn guard_drop(&mut self, addr: u64, guard: Ty, lock: &str, write: bool) -> R<()> {
        let TyKind::Struct(id) = *self.types.kind(guard) else { return fault("a guard that is no struct") };
        let Some((_, args)) = self.types.struct_def(id).origin.as_ref() else { return fault("a guard of no lock") };
        let Some(lock_ty) = self.types.all().find_map(|(ty, kind)| match kind {
            TyKind::Struct(found) => {
                let (origin, lock_args) = self.types.struct_def(*found).origin.as_ref()?;
                (origin.as_str() == lock && lock_args == args).then_some(ty)
            }
            _ => None,
        }) else {
            return Ok(());
        };
        let value = self.mem.read_u64(addr)?;
        let word = value + self.field_offset(lock_ty, "word")?;
        let held = self.mem.read_u32(word)?;
        let next = if lock == "std.sync.RwLock" && !write { held.saturating_sub(1) } else { 0 };
        self.mem.write_u32(word, next)?;
        Ok(())
    }

    /// `atomic_call`: the operation on the value `args[0]` points at.
    pub(crate) fn atomic(&mut self, op: AtomicOp, order: Option<MemoryOrder>, value_ref: Ty, args: &[Operand], span: Span) -> R<Option<Vec<u8>>> {
        let TyKind::Ref { inner, .. } = *self.types.kind(value_ref) else { return fault("an atomic's value is passed by its address") };
        if order.is_none() {
            // `ember_atomic_order`: an order held in a variable is checked as the call runs.
            let tag = self.operand(args.last().expect("the order operand"), None)?[0];
            let refused = MemoryOrder::ALL.get(tag as usize).is_none_or(|o| op.refuses(*o));
            if refused {
                let name = MemoryOrder::ALL.get(tag as usize).map_or("?", |o| o.name());
                return self.panic(&format!("`{}` does not support `MemoryOrder.{name}` [THR-14]", op.name()), span);
            }
        }
        let p = self.pointer(&args[0])?;
        let size = self.size(inner);
        let kind = self.int_kind(inner).unwrap_or(IntKind { bits: 8, signed: false });
        let current = self.mem.read(p, size)?.to_vec();
        let operand = |interp: &mut Self, i: usize| interp.operand(&args[i], Some(inner));
        Ok(Some(match op {
            AtomicOp::Load => current,
            AtomicOp::Store => {
                let value = operand(self, 1)?;
                self.mem.write(p, &value)?;
                return Ok(None);
            }
            AtomicOp::Swap => {
                let value = operand(self, 1)?;
                self.mem.write(p, &value)?;
                current
            }
            // The expected value's address; on failure it is given what was there.
            AtomicOp::CompareExchange => {
                let expected = self.pointer(&args[1])?;
                let desired = operand(self, 2)?;
                if self.mem.read(expected, size)? == current.as_slice() {
                    self.mem.write(p, &desired)?;
                    vec![1]
                } else {
                    self.mem.write(expected, &current)?;
                    vec![0]
                }
            }
            AtomicOp::FetchAdd | AtomicOp::FetchSub | AtomicOp::FetchAnd | AtomicOp::FetchOr | AtomicOp::FetchXor => {
                let value = operand(self, 1)?;
                let binary = match op {
                    AtomicOp::FetchAdd => BinOp::Add,
                    AtomicOp::FetchSub => BinOp::Sub,
                    AtomicOp::FetchAnd => BinOp::BitAnd,
                    AtomicOp::FetchOr => BinOp::BitOr,
                    _ => BinOp::BitXor,
                };
                let next = crate::scalar::int_binary(binary, kind, &current, &value);
                self.mem.write(p, &next)?;
                current
            }
            // `ember_atomic_wait32`: a wait whose value is still there sleeps until a wake no
            // other thread can send.
            AtomicOp::Wait => {
                let expected = self.operand(&args[1], None)?;
                if self.mem.read(p, 4)? == &expected[..4] {
                    return self.waits_for_ever("a wait on a value");
                }
                return Ok(None);
            }
            AtomicOp::WakeOne | AtomicOp::WakeAll => return Ok(None),
        }))
    }
}
