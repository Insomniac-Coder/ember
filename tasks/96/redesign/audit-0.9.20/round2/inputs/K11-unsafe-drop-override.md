# K11-unsafe-drop-override

Question: May a class's drop, or an override of a safe method, be an unsafe fn?
Severity: high; impact rank 11
Members: J5-16; primary J5-16
Why together: Single question.
Critic's notes: Unsafe code reached from safe code with nobody vouching (principle 11). A also needs a rule that an interface implementation's signature must match, which J5's notes found missing.

## Finding J5-16
Rules: UNS-1, UNS-12, DRP-8, DSP-2; file tasks/spec-0.9.9/parts/p09-memory.md lines 267-268; kind undecided_case; severity medium
Quote from Hardened_11:
through `*T`/`*mut T`; offset a pointer; cast between pointers or pointers and integers; call an
`unsafe fn` (`Allocator.deallocate` and `reallocate` among them, `[ALC-2]`), or through a value of an

The finder's question:
May a class declare `unsafe fn drop()`, or override a safe `virtual` method (or implement a safe interface
method) with an `unsafe fn`? If so, who vouches when the object ends in safe code, or when the method is
called through the base?

Program:
```
class Window:
    handle: *mut u8

    @safety("`this.handle` came from `create_window` and is destroyed only here")
    unsafe fn drop():
        destroy_window(this.handle)

fn main():
    w = open_window()          # a safe function returning a Window
    println("open")
    # w ends here, in safe code: an unsafe fn runs with no unsafe block and no SAFETY note
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
The grammar (p03 63) allows `unsafe` before any method, `override` included, and [UNS-12] makes the whole body
unsafe code. [UNS-1] requires unsafe code to call an `unsafe fn`, while [DRP-1]/[DRP-8] run `drop` wherever
the value ends, and [DSP-2]/[DSP-5] send a safe virtual call to any override. No rule refuses `unsafe fn
drop()` or an `unsafe` override, and none says who vouches (grep 'unsafe fn drop', 'override.*unsafe':
nothing). Verdicts 60, 118 and 119 do not reach this case.

options:
[{'letter': 'A', 'summary': '`unsafe` is refused on `drop`, on an override of a safe method, and on an
implementation of a safe interface method.', 'code': '# SAFETY: these declarations match window.h\nunsafe
extern "C":\n    safe fn create_window() -> *mut u8\n    @safety("`w` came from `create_window` and is
destroyed once")\n    unsafe fn destroy_window(w: *mut u8)\n\nclass Window:\n    handle: *mut u8\n\n    fn
drop():                               # safe: it runs wherever a Window ends\n        # SAFETY: `this.handle`
came from `create_window`, and drop runs once\n        unsafe:\n            destroy_window(this.handle)\n\nfn
main():\n    w = Window(handle=create_window())\n    println("open")', 'outcome': 'In the example, `unsafe fn
drop()` is refused: "`drop` runs wherever a `Window` ends, in safe code, so it cannot be an `unsafe fn`; write
`fn drop():` and put the call in an `unsafe:` block with a `# SAFETY:` note". This version compiles, prints
open, and destroys the window when `w` ends. An `unsafe` override of a safe `virtual` method is refused the
same way.'}, {'letter': 'B', 'summary': 'Allowed, but every end of such an object is unsafe, so it must be
ended inside `unsafe:`.', 'code': '# SAFETY: these declarations match window.h\nunsafe extern "C":\n    safe
fn create_window() -> *mut u8\n    @safety("`w` came from `create_window` and is destroyed once")\n    unsafe
fn destroy_window(w: *mut u8)\n\nclass Window:\n    handle: *mut u8\n\n    @safety("`this.handle` came from
`create_window` and is destroyed only here")\n    unsafe fn drop():\n        destroy_window(this.handle)\n\nfn
main():\n    w = Window(handle=create_window())\n    println("open")\n    # SAFETY: the window\'s handle came
from create_window\n    unsafe:\n        mem.drop(owns w)', 'outcome': 'The example is E3100 at the end of
`main`, where `w` ends outside unsafe code. This version compiles. Every place a `Window` may end (a list of
them cleared, a field replaced) needs the same, and a safe virtual call that may reach an `unsafe` override is
E3100 too.'}]

recommendation:
{'letter': 'A', 'reason': 'A `drop` and an override are called by code that cannot vouch for them (the end of
a block, a safe virtual call), so the vouching belongs inside them, in an `unsafe:` block with its note. Rust
does the same: `Drop::drop` is safe.', 'checklist': "8 (a forgotten block is a refusal), 9 (the message gives
a line that compiles), 11 (unsafe only where someone vouches), 13 (no unsafe path hidden in a block's end)"}

