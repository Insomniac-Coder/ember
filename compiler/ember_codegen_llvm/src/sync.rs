//! `[THR-14]` — atomics as LLVM's own atomic instructions (`load atomic`, `store atomic`,
//! `atomicrmw`, `cmpxchg`), at the orders the program names; an order held in a variable is
//! checked as the runtime's `atomic_order` checks it and then switched on, one arm per order, as
//! Clang lowers the C backend's `__atomic` builtins. A 128-bit value goes through the runtime's
//! compare-exchange loops (`atomic128_*`). `[THR-9]` — `wait` and `wake` are the runtime's.

use crate::func::{FnCx, Param, Val};
use crate::types::{Repr, Sc};
use crate::{Module, Result, global_name, unsupported};
use ember_mir::{AtomicOp, Builtin, Const, MemoryOrder, Operand, Place};
use ember_types::{Ty, TyKind};

/// `[THR-10]` — a thread's packet: the runtime's `ember_thread` head (40 bytes), the callable value
/// it runs (40), and the slot for its result.
struct Packet {
    size: u64,
    align: u64,
    result: u64,
}

const HEAD: u64 = 40;
const VALUE: u64 = 40;

fn packet(m: &Module<'_>, result: Ty) -> Packet {
    let (size, align) = m.ty.size_align(result);
    let align = align.max(1);
    let at = (HEAD + VALUE).div_ceil(align) * align;
    let whole = align.max(8);
    Packet { size: (at + size).div_ceil(whole) * whole, align: whole, result: at }
}

/// The packet's `run` (the value called on its state, its result kept, the value dropped) and,
/// when the result needs it, the `drop_result` a result nobody took gets.
fn thread_entries(m: &mut Module<'_>, result: Ty) -> Result<(String, Option<String>)> {
    let mangled = m.ty.table.mangled(result);
    let packet = packet(m, result);
    let run = format!("ember.thread.run.{mangled}");
    let returns = !m.ty.is_zst(result);
    if !m.helper_defined(&run) {
        crate::glue::define_glue(m, &run, &["ptr"], "", |cx| {
            let native_at = cx.gep("%p0", HEAD);
            let native = cx.load(Sc::Ptr, &native_at, 8);
            let state = cx.gep("%p0", HEAD + 16);
            let slot = cx.gep("%p0", packet.result);
            let (_, align) = cx.m.ty.size_align(result);
            cx.call_c(&native, returns.then_some(result), &[Param::Ptr], vec![Val::Imm(Sc::Ptr, state.clone())], returns.then_some((slot.as_str(), align)));
            cx.drop_callable_value(&format!("%p0"), HEAD);
            Ok(())
        })?;
    }
    let drop = if returns && m.ty.table.needs_drop(result) {
        let drop = format!("ember.thread.drop.{mangled}");
        if !m.helper_defined(&drop) {
            crate::glue::define_glue(m, &drop, &["ptr"], "", |cx| {
                let slot = cx.gep("%p0", packet.result);
                let (_, align) = cx.m.ty.size_align(result);
                cx.drop_at(&slot, align.min(16), result)
            })?;
        }
        Some(global_name(&drop))
    } else {
        None
    };
    Ok((global_name(&run), drop))
}

/// `[JOB-1]` — the entry and drop of a job given a callable value, which its state holds.
fn job_value_entries(m: &mut Module<'_>) -> Result<(String, String)> {
    let (run, drop) = ("ember.job.run_fn", "ember.job.drop_fn");
    if !m.helper_defined(run) {
        crate::glue::define_glue(m, run, &["ptr"], "", |cx| {
            let native = cx.load(Sc::Ptr, "%p0", 8);
            let state = cx.gep("%p0", 16);
            cx.call_c(&native, None, &[Param::Ptr], vec![Val::Imm(Sc::Ptr, state)], None);
            Ok(())
        })?;
        crate::glue::define_glue(m, drop, &["ptr"], "", |cx| {
            cx.drop_callable_value("%p0", 0);
            Ok(())
        })?;
    }
    Ok((global_name(run), global_name(drop)))
}

/// `[JOB-1]` — the entry and (where the environment needs one) drop of a job a closure is: on
/// the environment its state holds.
fn job_closure_entries(m: &mut Module<'_>, closure: &str, env: Ty) -> Result<(String, Option<String>)> {
    let run = format!("ember.job.run.{closure}");
    if !m.helper_defined(&run) {
        let Some(body) = m.bodies.get(closure).copied() else { return unsupported("a job closure without a body") };
        let by_pointer = body.args().next().is_some_and(|(_, decl)| matches!(m.ty.kind(decl.ty), TyKind::Ref { .. } | TyKind::Ptr { .. }));
        let closure = closure.to_string();
        crate::glue::define_glue(m, &run, &["ptr"], "", |cx| {
            let env_value = if by_pointer { Val::Imm(Sc::Ptr, "%p0".to_string()) } else { cx.value_in("%p0", env) };
            cx.call_body(&closure, vec![env_value], None)
        })?;
    }
    let drop = if m.ty.table.needs_drop(env) {
        let drop = format!("ember.job.drop.{}", m.ty.table.mangled(env));
        if !m.helper_defined(&drop) {
            crate::glue::define_glue(m, &drop, &["ptr"], "", |cx| {
                let (_, align) = cx.m.ty.size_align(env);
                cx.drop_at("%p0", align.min(16), env)
            })?;
        }
        Some(global_name(&drop))
    } else {
        None
    };
    Ok((global_name(&run), drop))
}

fn ordering(order: MemoryOrder) -> &'static str {
    match order {
        MemoryOrder::Relaxed => "monotonic",
        MemoryOrder::Acquire => "acquire",
        MemoryOrder::Release => "release",
        MemoryOrder::AcqRel => "acq_rel",
        MemoryOrder::SeqCst => "seq_cst",
    }
}

impl<'a> FnCx<'_, 'a> {
    /// The callable value at `offset` in `base` ended: its descriptor's `drop`, where it has one,
    /// on its state.
    fn drop_callable_value(&mut self, base: &str, offset: u64) {
        let descriptor_at = self.gep(base, offset + 8);
        let descriptor = self.load(Sc::Ptr, &descriptor_at, 8);
        let state = self.gep(base, offset + 16);
        let has = self.value(format!("icmp ne ptr {descriptor}, null"));
        let (look, call, done) = (self.label(), self.label(), self.label());
        self.emit(format!("br i1 {has}, label %{look}, label %{done}"));
        self.start(&look);
        let drop_at = self.gep(&descriptor, 16);
        let drop = self.load(Sc::Ptr, &drop_at, 8);
        let drops = self.value(format!("icmp ne ptr {drop}, null"));
        self.emit(format!("br i1 {drops}, label %{call}, label %{done}"));
        self.start(&call);
        self.emit(format!("call void {drop}(ptr {state})"));
        self.emit(format!("br label %{done}"));
        self.start(&done);
    }

    /// `dest` (a struct) zeroed and its field `name` set to `value` (`ir` typed).
    fn make_with(&mut self, dest: &Place, name: &str, ir: &str, value: &str) -> Result<()> {
        let ty = self.place_ty(dest);
        let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
        let (size, _) = self.m.ty.size_align(ty);
        self.zero(&at, align, size);
        let Some(offset) = crate::arenas::field_offset(self.m, ty, name) else {
            return unsupported(format!("a `{}` without `{name}`", self.m.ty.table.display(ty)));
        };
        let field = self.gep(&at, offset);
        self.emit(format!("store {ir} {value}, ptr {field}, align 8"));
        Ok(())
    }

    /// `[THR-10]`, `[THR-11]` — a thread started on the callable value at `value` (40 bytes),
    /// added to the scope's tasks when scoped: its packet.
    fn spawn(&mut self, result: Ty, value: &str, tasks: Option<&str>, span: ember_span::Span) -> Result<String> {
        let packet = packet(self.m, result);
        let (run, drop) = thread_entries(self.m, result)?;
        let slot = self.alloca(8, 8);
        self.call_rt("alloc", vec![Val::Imm(Sc::I(64), packet.size.to_string()), Val::Imm(Sc::I(64), packet.align.to_string())], Some((&slot, 8)));
        let thread = self.load(Sc::Ptr, &slot, 8);
        let callable = self.gep(&thread, HEAD);
        self.copy(&callable, 8, value, 8, VALUE, false);
        let loc = self.loc(span);
        self.call_rt(
            "thread_start",
            vec![
                Val::Imm(Sc::Ptr, thread.clone()),
                Val::Imm(Sc::Ptr, run),
                Val::Imm(Sc::Ptr, drop.unwrap_or_else(|| "null".to_string())),
                Val::Imm(Sc::I(32), packet.size.to_string()),
                Val::Imm(Sc::I(32), packet.align.to_string()),
                loc,
            ],
            None,
        );
        if let Some(tasks) = tasks {
            self.call_rt("scope_add", vec![Val::Imm(Sc::Ptr, tasks.to_string()), Val::Imm(Sc::Ptr, thread.clone())], None);
        }
        Ok(thread)
    }

    /// `[THR-5]`, `[THR-10]`, `[THR-11]`, `[JOB-1]`..`[JOB-3]`, `[PAR-1]`..`[PAR-4]` — a thread, scope,
    /// job or parallel-loop builtin, or `None` for another.
    pub(crate) fn concurrency_builtin(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place, span: ember_span::Span) -> Option<Result<()>> {
        Some(match which {
            Builtin::ThreadSpawn => (|| {
                let TyKind::Fn { ret, .. } = *self.m.ty.kind(arg_ty) else { return unsupported("a thread that runs no callable value") };
                let value = self.operand(&args[0])?;
                let (value, _) = self.spill(&value, VALUE, 8);
                let thread = self.spawn(ret, &value, None, span)?;
                self.make_with(dest, "thread", "ptr", &thread)
            })(),
            Builtin::ThreadJoin | Builtin::ScopedJoin => (|| {
                let TyKind::Struct(id) = *self.m.ty.kind(arg_ty) else { return unsupported("a join of no handle") };
                let def = self.m.ty.table.struct_def(id).clone();
                let Some((origin, type_args)) = &def.origin else { return unsupported("a join of no handle") };
                let scoped = origin.as_str() == "std.thread.ScopedJoinHandle";
                let result = type_args[0];
                let handle = self.operand(&args[0])?;
                let (size, align) = self.m.ty.size_align(arg_ty);
                let (handle, _) = self.spill(&handle, size, align);
                let thread = self.word_of(&handle, arg_ty, "thread")?;
                self.call_rt(if scoped { "scope_join" } else { "thread_join" }, vec![Val::Imm(Sc::Ptr, thread.clone())], None);
                let packet = packet(self.m, result);
                let slot = self.gep(&thread, packet.result);
                if let Some((at, at_align)) = self.dest_of(dest)? {
                    let (size, align) = self.m.ty.size_align(result);
                    self.copy(&at, at_align, &slot, align.min(16), size, false);
                }
                if !scoped {
                    self.call_rt("thread_free", vec![Val::Imm(Sc::Ptr, thread)], None);
                }
                Ok(())
            })(),
            // `[THR-5]` — a scope's task list; its end (the drop) joins them.
            Builtin::ThreadScope => (|| {
                let tasks = self.alloca(8, 8);
                self.call_rt("scope_new", Vec::new(), Some((&tasks, 8)));
                let tasks = self.load(Sc::Ptr, &tasks, 8);
                self.make_with(dest, "tasks", "ptr", &tasks)
            })(),
            // `[THR-11]` — the task's callable value: made here from the closure's environment, or
            // given, for a function.
            Builtin::ScopeSpawn { closure } => (|| {
                let TyKind::Struct(id) = *self.m.ty.kind(arg_ty) else { return unsupported("a scoped spawn of no handle") };
                let result = self.m.ty.table.struct_def(id).origin.as_ref().map(|(_, args)| args[0]).expect("a handle names its result");
                let scope = self.pointer(&args[0])?;
                let scope_ty = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let tasks = self.word_of(&scope, scope_ty, "tasks")?;
                let value = match (closure, &args[..]) {
                    (Some(_), [_, function @ Operand::Const(Const::Fn(_)), env]) => {
                        let slot = self.alloca(VALUE, 8);
                        self.make_callable(&[function.clone(), env.clone()], &slot, 8)?;
                        slot
                    }
                    _ => {
                        let value = self.operand(&args[1])?;
                        self.spill(&value, VALUE, 8).0
                    }
                };
                let thread = self.spawn(result, &value, Some(&tasks), span)?;
                self.make_with(dest, "thread", "ptr", &thread)
            })(),
            Builtin::JobScope => (|| {
                let ty = self.place_ty(dest);
                let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
                let (size, _) = self.m.ty.size_align(ty);
                self.zero(&at, align, size);
                Ok(())
            })(),
            // `[JOB-1]`..`[JOB-3]` — the job's entry, drop and state (its closure's environment,
            // or the callable value), the scope's count, and the jobs first.
            Builtin::JobSubmit { closure } | Builtin::JobScopeSubmit { closure, .. } => (|| {
                let scope = match which {
                    Builtin::JobScopeSubmit { .. } => {
                        let scope = self.pointer(&args[0])?;
                        let scope_ty = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                        let Some(offset) = crate::arenas::field_offset(self.m, scope_ty, "pending") else { return unsupported("a job scope without its count") };
                        self.gep(&scope, offset)
                    }
                    _ => "null".to_string(),
                };
                let (deps, count, stride, offset) = match which {
                    Builtin::JobScopeSubmit { after: true, .. } => {
                        let (words, align) = self.view_words(&args[1])?;
                        let pointer = self.load(Sc::Ptr, &words, align.min(8));
                        let len_at = self.gep(&words, 8);
                        let len = self.load(Sc::I(64), &len_at, align.min(8));
                        let (record, _) = self.m.ty.size_align(arg_ty);
                        let Some(id_at) = crate::arenas::field_offset(self.m, arg_ty, "id") else { return unsupported("a scoped job without its id") };
                        (pointer, len, record.to_string(), id_at.to_string())
                    }
                    _ => ("null".to_string(), "0".to_string(), "0".to_string(), "0".to_string()),
                };
                let (run, drop, state, size, align) = match (closure, &args[..]) {
                    (Some(_), [.., Operand::Const(Const::Fn(function)), env]) => {
                        let Some(env_ty) = self.operand_ty(env).or_else(|| self.m.callables.closures.get(function.as_str()).copied()) else {
                            return unsupported("a job whose environment has no type");
                        };
                        let (run, drop) = job_closure_entries(self.m, function, env_ty)?;
                        let (size, align) = self.m.ty.size_align(env_ty);
                        let value = self.operand(env)?;
                        let (state, _) = self.spill(&value, size.max(1), align);
                        (run, drop.unwrap_or_else(|| "null".to_string()), state, size, align)
                    }
                    _ => {
                        let (run, drop) = job_value_entries(self.m)?;
                        let value = self.operand(args.last().expect("the job's callable value"))?;
                        let (state, _) = self.spill(&value, VALUE, 8);
                        (run, drop, state, VALUE, 8)
                    }
                };
                let loc = self.loc(span);
                let id = self.alloca(8, 8);
                let word = |text: String| Val::Imm(Sc::I(64), text);
                self.call_rt(
                    "job_submit",
                    vec![
                        Val::Imm(Sc::Ptr, scope),
                        Val::Imm(Sc::Ptr, run),
                        Val::Imm(Sc::Ptr, drop),
                        Val::Imm(Sc::Ptr, state),
                        word(size.to_string()),
                        word(align.to_string()),
                        Val::Imm(Sc::Ptr, deps),
                        word(count),
                        word(stride),
                        word(offset),
                        loc,
                    ],
                    Some((&id, 8)),
                );
                let id = self.load(Sc::I(64), &id, 8);
                self.make_with(dest, "id", "i64", &id)
            })(),
            Builtin::JobWait => (|| {
                let ty = self.operand_ty(&args[0]).unwrap_or(arg_ty);
                let job = self.operand(&args[0])?;
                let (size, align) = self.m.ty.size_align(ty);
                let (job, _) = self.spill(&job, size, align);
                let Some(offset) = crate::arenas::field_offset(self.m, ty, "id") else { return unsupported("a job without its id") };
                let id_at = self.gep(&job, offset);
                let id = self.load(Sc::I(64), &id_at, 8);
                self.call_rt("job_wait", vec![Val::Imm(Sc::I(64), id)], None);
                Ok(())
            })(),
            Builtin::ParallelFor { .. } => self.parallel_for(args, span),
            _ => return None,
        })
    }

    /// `[PAR-1]`..`[PAR-4]` — the runtime runs the chunks on the pool and here; with reductions, a
    /// window of 32 chunks' results and their stamps live in this frame.
    fn parallel_for(&mut self, args: &[Operand], span: ember_span::Span) -> Result<()> {
        let Operand::Const(Const::Fn(chunk)) = &args[3] else { return unsupported("a parallel loop whose body is not a function") };
        let start = self.word(&args[0])?;
        let end = self.word(&args[1])?;
        let size = self.word(&args[2])?;
        let env = match &args[4] {
            Operand::Const(Const::Void) => "null".to_string(),
            env => {
                let ty = self.operand_ty(env);
                let value = self.operand(env)?;
                let (size, align) = ty.map_or((8, 8), |ty| self.m.ty.size_align(ty));
                self.spill(&value, size.max(1), align).0
            }
        };
        let chunk_entry = self.chunk_entry(chunk)?;
        let (fold, acc, acc_size, window, stamps) = match (args.get(5), args.get(6)) {
            (Some(Operand::Const(Const::Fn(combine))), Some(acc)) => {
                let Some(TyKind::Ref { inner, .. }) = self.operand_ty(acc).map(|ty| self.m.ty.kind(ty).clone()) else {
                    return unsupported("running results that are not borrowed");
                };
                let fold = self.fold_entry(combine)?;
                let acc = self.pointer(acc)?;
                let (size, align) = self.m.ty.size_align(inner);
                let window = self.alloca((32 * size).max(1), align.max(1));
                self.zero(&window, align.max(1), 32 * size);
                let stamps = self.alloca(32 * 8, 8);
                self.zero(&stamps, 8, 32 * 8);
                (fold, acc, size, window, stamps)
            }
            _ => ("null".to_string(), "null".to_string(), 0, "null".to_string(), "null".to_string()),
        };
        let loc = self.loc(span);
        self.call_rt(
            "parallel_for",
            vec![
                Val::Imm(Sc::I(64), start),
                Val::Imm(Sc::I(64), end),
                Val::Imm(Sc::I(64), size),
                Val::Imm(Sc::Ptr, chunk_entry),
                Val::Imm(Sc::Ptr, fold),
                Val::Imm(Sc::Ptr, env),
                Val::Imm(Sc::Ptr, acc),
                Val::Imm(Sc::I(64), acc_size.to_string()),
                Val::Imm(Sc::Ptr, window),
                Val::Imm(Sc::Ptr, stamps),
                Val::Imm(Sc::I(32), "32".to_string()),
                loc,
            ],
            None,
        );
        Ok(())
    }

    /// `[PAR-1]` — the entry the runtime calls for one chunk: the chunk function on the loop's
    /// environment, `lo` and `hi`, its reductions stored to `partial`.
    fn chunk_entry(&mut self, chunk: &str) -> Result<String> {
        let symbol = format!("ember.parallel.chunk.{chunk}");
        if self.m.helper_defined(&symbol) {
            return Ok(global_name(&symbol));
        }
        let Some(function) = self.m.bodies.get(chunk).copied() else { return unsupported("a parallel loop body without a body") };
        let params: Vec<Ty> = function.args().map(|(_, decl)| decl.ty).collect();
        let ret = function.return_ty();
        let chunk = chunk.to_string();
        crate::glue::define_glue(self.m, &symbol, &["ptr", "i64", "i64", "ptr"], "", |cx| {
            let mut arguments = Vec::new();
            if params.len() == 3 {
                let env = params[0];
                arguments.push(match cx.m.ty.kind(env) {
                    TyKind::Ref { .. } | TyKind::Ptr { .. } => Val::Imm(Sc::Ptr, "%p0".to_string()),
                    _ => cx.value_in("%p0", env),
                });
            }
            arguments.push(Val::Imm(Sc::I(64), "%p1".to_string()));
            arguments.push(Val::Imm(Sc::I(64), "%p2".to_string()));
            let (_, align) = cx.m.ty.size_align(ret);
            let dest = (!cx.m.ty.is_zst(ret)).then_some(("%p3", align));
            cx.call_body(&chunk, arguments, dest)
        })?;
        Ok(global_name(&symbol))
    }

    /// `[PAR-3]` — the fold of one chunk's reductions into the running ones.
    fn fold_entry(&mut self, combine: &str) -> Result<String> {
        let symbol = format!("ember.parallel.fold.{combine}");
        if self.m.helper_defined(&symbol) {
            return Ok(global_name(&symbol));
        }
        let Some(function) = self.m.bodies.get(combine).copied() else { return unsupported("a combiner without a body") };
        let params: Vec<Ty> = function.args().map(|(_, decl)| decl.ty).collect();
        let ret = function.return_ty();
        let combine = combine.to_string();
        crate::glue::define_glue(self.m, &symbol, &["ptr", "ptr", "ptr"], "", |cx| {
            let (size, align) = cx.m.ty.size_align(ret);
            // Both read before the result is written over the running one.
            let (a, b) = (cx.alloca(size.max(1), align), cx.alloca(size.max(1), align));
            cx.copy(&a, align, "%p1", align, size, false);
            cx.copy(&b, align, "%p2", align, size, false);
            let argument = |cx: &mut FnCx<'_, '_>, ty: Ty, at: &str| match cx.m.ty.kind(ty) {
                TyKind::Ref { .. } | TyKind::Ptr { .. } => Val::Imm(Sc::Ptr, at.to_string()),
                _ => cx.value_in(at, ty),
            };
            let arguments = vec![argument(cx, params[0], &a), argument(cx, params[1], &b)];
            cx.call_body(&combine, arguments, Some(("%p1", align)))
        })?;
        Ok(global_name(&symbol))
    }

    /// `[THR-5]` — a scope's end joins its threads; `[JOB-2]` — a job scope's waits for its
    /// jobs; `[THR-10]` — a handle dropped unjoined detaches its thread.
    pub(crate) fn concurrency_drop(&mut self, origin: &str, at: &str, ty: Ty) -> Result<()> {
        match origin {
            "std.thread.Scope" => {
                let tasks = self.word_of(at, ty, "tasks")?;
                self.call_rt("scope_end", vec![Val::Imm(Sc::Ptr, tasks)], None);
            }
            "std.jobs.JobScope" => {
                let Some(offset) = crate::arenas::field_offset(self.m, ty, "pending") else { return unsupported("a job scope without its count") };
                let pending = self.gep(at, offset);
                self.call_rt("job_scope_end", vec![Val::Imm(Sc::Ptr, pending)], None);
            }
            _ => {
                let thread = self.word_of(at, ty, "thread")?;
                self.call_rt("thread_detach", vec![Val::Imm(Sc::Ptr, thread)], None);
            }
        }
        Ok(())
    }

    /// `[THR-14]` — an atomic operation on the value `args[0]` points at.
    pub(crate) fn atomic(&mut self, op: AtomicOp, order: Option<MemoryOrder>, value_ref: Ty, args: &[Operand], dest: &Place, span: ember_span::Span) -> Result<()> {
        let TyKind::Ref { inner, .. } = *self.m.ty.kind(value_ref) else { return unsupported("an atomic not through its address") };
        let pointer = self.pointer(&args[0])?;
        match op {
            AtomicOp::Wait => {
                let expected = self.word(&args[1])?;
                let expected = self.int_convert(&expected, 64, 32, false);
                self.call_rt("atomic_wait32", vec![Val::Imm(Sc::Ptr, pointer), Val::Imm(Sc::I(32), expected)], None);
                return Ok(());
            }
            AtomicOp::WakeOne | AtomicOp::WakeAll => {
                let name = if op == AtomicOp::WakeOne { "atomic_wake_one" } else { "atomic_wake_all" };
                self.call_rt(name, vec![Val::Imm(Sc::Ptr, pointer)], None);
                return Ok(());
            }
            _ => {}
        }
        let sc = match self.m.ty.repr(inner) {
            Repr::Scalar(sc @ (Sc::Bool | Sc::I(_) | Sc::Ptr)) => sc,
            _ => return unsupported(format!("an atomic `{}`", self.m.ty.table.display(inner))),
        };
        if sc == Sc::I(128) {
            return self.atomic128(op, pointer, args, dest);
        }
        // The value as memory holds it (a `bool` a byte).
        let (ty, align) = match sc {
            Sc::Bool => ("i8".to_string(), 1),
            Sc::Ptr => ("ptr".to_string(), 8),
            Sc::I(bits) => (format!("i{bits}"), u64::from(bits / 8)),
            _ => unreachable!("matched above"),
        };
        let operand = |cx: &mut Self, index: usize| -> Result<String> {
            let Val::Imm(_, value) = cx.operand(&args[index])? else { return unsupported("an atomic operand held in memory") };
            Ok(if sc == Sc::Bool { cx.value(format!("zext i1 {value} to i8")) } else { value })
        };
        let value = match op {
            AtomicOp::Store | AtomicOp::Swap | AtomicOp::FetchAdd | AtomicOp::FetchSub | AtomicOp::FetchAnd | AtomicOp::FetchOr | AtomicOp::FetchXor => {
                Some(operand(self, 1)?)
            }
            AtomicOp::CompareExchange => Some(operand(self, 2)?),
            _ => None,
        };
        let expected_at = if op == AtomicOp::CompareExchange { Some(self.pointer(&args[1])?) } else { None };
        // The result, written by whichever order's arm runs.
        let result = (op != AtomicOp::Store).then(|| self.alloca(align.max(1), align.max(1)));
        let arm = |cx: &mut Self, order: MemoryOrder| {
            let o = ordering(order);
            let text = match op {
                AtomicOp::Load => format!("load atomic {ty}, ptr {pointer} {o}, align {align}"),
                AtomicOp::Store => {
                    cx.emit(format!("store atomic {ty} {}, ptr {pointer} {o}, align {align}", value.as_deref().unwrap_or_default()));
                    return;
                }
                AtomicOp::CompareExchange => {
                    let expected_at = expected_at.as_deref().expect("a compare-exchange's expected value");
                    let expected = cx.value(format!("load {ty}, ptr {expected_at}, align {align}"));
                    let pair = cx.value(format!(
                        "cmpxchg ptr {pointer}, {ty} {expected}, {ty} {} {o} {}, align {align}",
                        value.as_deref().unwrap_or_default(),
                        ordering(order.on_failure())
                    ));
                    let seen = cx.value(format!("extractvalue {{ {ty}, i1 }} {pair}, 0"));
                    let swapped = cx.value(format!("extractvalue {{ {ty}, i1 }} {pair}, 1"));
                    // C's compare-exchange writes what it found where the expected value was.
                    let (failed, done) = (cx.label(), cx.label());
                    cx.emit(format!("br i1 {swapped}, label %{done}, label %{failed}"));
                    cx.start(&failed);
                    cx.emit(format!("store {ty} {seen}, ptr {expected_at}, align {align}"));
                    cx.emit(format!("br label %{done}"));
                    cx.start(&done);
                    let byte = cx.value(format!("zext i1 {swapped} to i8"));
                    cx.emit(format!("store i8 {byte}, ptr {}, align 1", result.as_deref().expect("a result")));
                    return;
                }
                _ => {
                    let rmw = match op {
                        AtomicOp::Swap => "xchg",
                        AtomicOp::FetchAdd => "add",
                        AtomicOp::FetchSub => "sub",
                        AtomicOp::FetchAnd => "and",
                        AtomicOp::FetchOr => "or",
                        _ => "xor",
                    };
                    format!("atomicrmw {rmw} ptr {pointer}, {ty} {} {o}, align {align}", value.as_deref().unwrap_or_default())
                }
            };
            let loaded = cx.value(text);
            cx.emit(format!("store {ty} {loaded}, ptr {}, align {align}", result.as_deref().expect("a result")));
        };
        match order {
            Some(order) => arm(self, order),
            None => {
                // `[THR-14]` — an order held in a variable: checked, then switched on.
                let Val::Imm(Sc::I(tag_bits), tag) = self.operand(args.last().expect("the order operand"))? else {
                    return unsupported("an atomic order that is not a tag");
                };
                let refused: Vec<usize> = MemoryOrder::ALL.iter().enumerate().filter(|(_, order)| op.refuses(**order)).map(|(index, _)| index).collect();
                let tag = self.int_convert(&tag, tag_bits, 8, false);
                let mut bad = self.value(format!("icmp ugt i8 {tag}, 4"));
                for index in &refused {
                    let this = self.value(format!("icmp eq i8 {tag}, {index}"));
                    bad = self.value(format!("or i1 {bad}, {this}"));
                }
                let (refuse, check_ok) = (self.label(), self.label());
                self.emit(format!("br i1 {bad}, label %{refuse}, label %{check_ok}"));
                self.start(&refuse);
                let name = self.m.bytes(op.name().as_bytes());
                let loc = self.loc(span);
                self.call_rt("atomic_order_refused", vec![Val::Imm(Sc::I(8), tag.clone()), Val::Imm(Sc::Ptr, name), loc], None);
                self.start(&check_ok);
                let done = self.label();
                let arms: Vec<(usize, MemoryOrder, String)> = MemoryOrder::ALL
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !refused.contains(index))
                    .map(|(index, order)| (index, *order, self.label()))
                    .collect();
                let cases: Vec<String> = arms.iter().map(|(index, _, label)| format!("i8 {index}, label %{label}")).collect();
                let unreachable = self.label();
                self.emit(format!("switch i8 {tag}, label %{unreachable} [ {} ]", cases.join(" ")));
                self.start(&unreachable);
                self.emit("unreachable");
                for (_, order, label) in &arms {
                    self.start(label);
                    arm(self, *order);
                    self.emit(format!("br label %{done}"));
                }
                self.start(&done);
            }
        }
        let Some(result) = result else { return Ok(()) };
        if op == AtomicOp::CompareExchange {
            let byte = self.load(Sc::I(8), &result, 1);
            let swapped = self.value(format!("icmp ne i8 {byte}, 0"));
            return self.store_result(dest, &swapped, Sc::Bool, false);
        }
        let loaded = self.value(format!("load {ty}, ptr {result}, align {align}"));
        match sc {
            Sc::Bool => {
                let truth = self.value(format!("icmp ne i8 {loaded}, 0"));
                self.store_result(dest, &truth, Sc::Bool, false)
            }
            sc => self.store_result(dest, &loaded, sc, false),
        }
    }

    /// `[THR-14]` — a 128-bit atomic: the runtime's 16-byte compare-exchange, and a loop over it
    /// for each read-modify-write (the runtime's `atomic128_fetch_*`), on the bits (two's
    /// complement makes the signed operations the unsigned ones).
    fn atomic128(&mut self, op: AtomicOp, pointer: String, args: &[Operand], dest: &Place) -> Result<()> {
        let at = Val::Imm(Sc::Ptr, pointer.clone());
        match op {
            AtomicOp::Load => return self.call_rt_into("atomic128_load", vec![at], dest),
            AtomicOp::Store => {
                let value = self.operand(&args[1])?;
                self.call_rt("atomic128_store", vec![at, value], None);
                return Ok(());
            }
            AtomicOp::Swap => {
                let value = self.operand(&args[1])?;
                return self.call_rt_into("atomic128_swap", vec![at, value], dest);
            }
            AtomicOp::CompareExchange => {
                let expected = self.pointer(&args[1])?;
                let value = self.operand(&args[2])?;
                return self.call_rt_into("atomic128_cas", vec![at, Val::Imm(Sc::Ptr, expected), value], dest);
            }
            _ => {}
        }
        let instruction = match op {
            AtomicOp::FetchAdd => "add",
            AtomicOp::FetchSub => "sub",
            AtomicOp::FetchAnd => "and",
            AtomicOp::FetchOr => "or",
            _ => "xor",
        };
        let Val::Imm(_, operand) = self.operand(&args[1])? else { return unsupported("a 128-bit atomic operand held in memory") };
        let old = self.alloca(16, 16);
        self.call_rt("atomic128_load", vec![at.clone()], Some((&old, 16)));
        let (again, done) = (self.label(), self.label());
        self.emit(format!("br label %{again}"));
        self.start(&again);
        let seen = self.load(Sc::I(128), &old, 16);
        let next = self.value(format!("{instruction} i128 {seen}, {operand}"));
        let swapped = self.alloca(1, 1);
        self.call_rt("atomic128_cas", vec![at, Val::Imm(Sc::Ptr, old.clone()), Val::Imm(Sc::I(128), next)], Some((&swapped, 1)));
        let swapped = self.load(Sc::Bool, &swapped, 1);
        self.emit(format!("br i1 {swapped}, label %{done}, label %{again}"));
        self.start(&done);
        let previous = self.load(Sc::I(128), &old, 16);
        self.store_result(dest, &previous, Sc::I(128), false)
    }
}
