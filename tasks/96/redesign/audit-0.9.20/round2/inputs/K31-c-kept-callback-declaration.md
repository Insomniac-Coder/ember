# K31-c-kept-callback-declaration

Question: How is a C function that keeps an Ember callback declared and called?
Severity: high; impact rank 31
Members: J8-01; primary J8-01
Why together: Single question.
Critic's notes: Its call word follows K12. A named function given to `register_cb` takes `owns` under K12's unified rule. K53 (J8-03) and K76 (J8-18) use this spelling in their examples.

## Finding J8-01
Rules: FFI-10, FFI-21, FFI-52, FFI-11; file tasks/spec-0.9.9/parts/p16-ffi.md lines 82-84, 400-417, 489-499; kind example_mismatch; severity high
Quote from Hardened_11:
complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is
      safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

The finder's question:
Does FFI-52's own `register_cb` declaration compile, and what is its callable Ember signature? It is a `safe
fn` with a raw `*mut void` parameter, which FFI-10 says is E5002. It writes no result, yet `setup` returns its
value as `Retained[ForeignCallback]`. It is called with one argument where two are written.

Program:
```
unsafe extern "C":
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())   # one argument; or `owns fn() ...` (verdict 158)?
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.75)
Reason:
The spec's only hand declaration of a C function that keeps a callback (FFI-52, p16-ffi.md 489-499) breaks
FFI-10 in three ways. It is a `safe fn` with raw `*mut void` parameters, which FFI-10 lines 82-84 refuse. It
writes no result, yet `setup` returns its value as `Retained[ForeignCallback]`. It is called with one argument
where two are declared. FFI-10 says how a `count(n)` length leaves the callable signature (lines 85-92,
ODR-074), and nothing says the same for `user_data`. FFI-11's ownership row (line 290) gives a callback no
Ember type, only "`callback(retained)` (`[FFI-21]`)". FFI-21 (lines 403-405) moves the function into a block C
keeps, which is a hand-over, and verdicts 147 and 158 put `owns` at the call for a hand-over. Neither verdict
covers a foreign callback parameter, because the parameter is never written as `owns`. No rule, verdict or ODR
settles any of this.

options:
[{'letter': 'A', 'summary': 'The declaration writes the Ember side: `owns cb: fn()`, the user-data pointer as
a hidden witness (like a counted length), and the token as its result. The call says `owns`, as it does for
every argument to an `owns` parameter.', 'code': 'unsafe extern "C":\n    @ffi(param(cb, callback(retained,
user_data=ud), threads(creator)), destroys(unregister))\n    safe fn register_cb(owns cb: fn(), ud: *mut void)
-> Retained[ForeignCallback]   # `ud` is written at its C position and left out of the call\n\nfn setup(owns
game: shares mut Game) -> Retained[ForeignCallback]:\n    return register_cb(owns fn() owns game:
game.add_point())\n# a `callback(borrowed)` parameter is written `cb: fn()` and called with no word, as a
function given to a call that runs it then', 'outcome': 'Compiles. `register_cb(fn() owns game: ...)` without
`owns` is the error that asks for `owns`, as `take(xs)` is. Under `threads(any)` the parameter is `owns cb:
thread_shared fn()`. Calling it with two arguments is the error for too many arguments.'}, {'letter': 'B',
'summary': 'The same declaration with a plain parameter `cb: fn()`. The call takes no word, because the
capture list already says what C keeps.', 'code': 'unsafe extern "C":\n    @ffi(param(cb, callback(retained,
user_data=ud), threads(creator)), destroys(unregister))\n    safe fn register_cb(cb: fn(), ud: *mut void) ->
Retained[ForeignCallback]\n\nfn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:\n    return
register_cb(fn() owns game: game.add_point())', 'outcome': 'Compiles. A function is handed over here with no
word at the call, unlike every other `owns` hand-over.'}]

recommendation:
{'letter': 'A', 'reason': 'C keeps the function, so it is handed over, and the language already writes a
hand-over as `owns` on the parameter and at the call, a function written in the call included. The user-data
witness is written exactly as the counted-length witness is.', 'checklist': '2 (callback APIs become
declarable), 4 (hand-over written, plumbing hidden), 5 and 11 (`owns` and the ABI-witness form reused), 16
(the call-site `owns` verdicts reach foreign calls)'}

