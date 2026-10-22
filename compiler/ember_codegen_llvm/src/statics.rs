//! `[STA-3]` — a lazily initialised static: its one storage, and the accessor that runs its
//! initialiser the first time any thread asks. The runtime holds one lock across a thread's chain
//! of initialisations (`rt_static_begin`), so another thread waits, and a static reached again on
//! its own chain is the initialisation-cycle panic: the C backend's `lazy_static_definitions`.

use crate::func::{FnCx, Val};
use crate::{Module, Result, global_name};
use ember_types::Ty;

/// The accessor of the static `name` that `init` initialises, written the first time.
pub(crate) fn lazy_static(m: &mut Module<'_>, name: &str, init: &str, ty: Ty) -> Result<String> {
    let accessor = format!("{init}_static");
    let ir_name = global_name(&accessor);
    if m.helper_defined(&accessor) {
        return Ok(ir_name);
    }
    let (size, align) = m.ty.size_align(ty);
    let storage = global_name(&format!("{init}.storage"));
    let state = global_name(&format!("{init}.state"));
    m.global(format!("{storage} = internal global [{} x i8] zeroinitializer, align {}", size.max(1), align.max(1)));
    m.global(format!("{state} = internal global i32 0, align 4"));
    let shown = name.rsplit('.').next().unwrap_or(name);
    let mut cx = FnCx::glue(m);
    let text = cx.m.bytes(shown.as_bytes());
    let first = cx.alloca(4, 4);
    cx.call_rt("rt_static_begin", vec![Val::Imm(crate::types::Sc::Ptr, state.clone()), Val::Imm(crate::types::Sc::Ptr, text)], Some((&first, 4)));
    let first = cx.load(crate::types::Sc::I(32), &first, 4);
    let runs = cx.value(format!("icmp ne i32 {first}, 0"));
    let (run, done) = (cx.label(), cx.label());
    cx.emit(format!("br i1 {runs}, label %{run}, label %{done}"));
    cx.start(&run);
    cx.call_body(init, Vec::new(), (size > 0).then_some((storage.as_str(), align)))?;
    cx.call_rt("rt_static_end", vec![Val::Imm(crate::types::Sc::Ptr, state)], None);
    cx.emit(format!("br label %{done}"));
    cx.start(&done);
    let attrs = crate::func::fn_attrs(cx.m);
    let text = format!(
        "define internal ptr {ir_name}() {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}  ret ptr {storage}\n}}\n\n",
        cx.allocas_text(),
        cx.out_text()
    );
    cx.m.fns.push_str(&text);
    Ok(ir_name)
}
