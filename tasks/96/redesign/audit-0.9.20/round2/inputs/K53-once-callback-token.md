# K53-once-callback-token

Question: Does a one-shot C callback give a token, and must it name an unregister function?
Severity: medium; impact rank 53
Members: J8-03; primary J8-03
Why together: Single question.
Critic's notes: Its declarations use K31's spelling.

## Finding J8-03
Rules: FFI-21, FFI-23, FFI-52, FFI-53; file tasks/spec-0.9.9/parts/p16-ffi.md lines 400-423, 541-544; kind undecided_case; severity medium
Quote from Hardened_11:
`retained` returns a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the
  callback through the declared `destroys(f)` (`E5041` if none is declared), and with `once` the
  trampoline ends the block after the first call.

The finder's question:
Does a call whose parameter is `callback(once)` return a `Retained` token and need `destroys(f)` (E5041)? Does
a `callback(retained)` parameter given a capture-free function with no `user_data` (`atexit`, `signal`) need
`destroys(f)` and return a token? Does ending such a token after the once-callback has already run call
`destroys(f)` again?

Program:
```
unsafe extern "C":
    @ffi(param(func, callback(retained)))
    safe fn atexit(func: extern "C" fn()) -> c_int        # C has no unregister: E5041?
    @ffi(param(cb, callback(once, user_data=req)))
    safe fn fs_read_async(path: CStrView, cb: extern "C" fn(*mut void, i32), req: *mut void)

fn flush_logs():
    println("bye")

fn main():
    atexit(flush_logs)                       # result c_int or a token?
    fs_read_async(c"save.dat", fn(n: i32) owns copy of n: println(n))   # token? destroys needed?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
FFI-21 (p16-ffi.md 415-417) gives the token and E5041 only to `retained`: "with `once` the trampoline ends the
block after the first call". FFI-23 (541-542) defines the token as what "a registering call returns for a
callback C keeps", and the glossary (p24-appx-f.md 29) and FFI-52 (454, "not yet unregistered") count a `once`
callback as kept. So the rules do not say whether a `once` registration returns a token, needs `destroys(f)`,
or what ending such a token after the run does (calling C's unregister for a request C already finished). The
finder's second case (`atexit`, a capture-free function with no user data) is settled: FFI-21's first sentence
(line 400) lets a C function-pointer parameter take a capture-free Ember function as an `extern "C" fn` with
no `callback(...)` word, and FFI-52 (457-458) counts "every Ember function given to C as an `extern "C" fn`
value" as runnable. Only the `once` question is left.

options:
[{'letter': 'A', 'summary': '`once` is like `retained`: it returns a token and needs `destroys(f)`. Ending the
token before the run unregisters, and ending it after the run does nothing.', 'code': '    @ffi(param(cb,
callback(once, user_data=req), threads(creator)), destroys(fs_cancel))\n    safe fn fs_read_async(path:
CStrView, owns cb: fn(i32), req: *mut void) -> Retained[ForeignCallback]\n\n    token =
fs_read_async(c"save.dat", owns fn(n: i32) shares mut loader: loader.loaded += n)', 'outcome': 'Compiles when
the library has a cancel function. The declaration in the example (no cancel function) is E5041, so a library
without one cannot be declared safe.'}, {'letter': 'B', 'summary': '`once` never gives a token and never needs
`destroys(f)`. The block ends when C makes the one call.', 'code': '    @ffi(param(cb, callback(once,
user_data=req), threads(creator)))\n    safe fn fs_read_async(path: CStrView, owns cb: fn(i32), req: *mut
void)\n\n    fs_read_async(c"save.dat", owns fn(n: i32) shares mut loader: loader.loaded += n)', 'outcome':
'The example compiles and prints the bytes read. A library that can cancel cannot pass that on to Ember: with
`destroys(f)` written, the word is refused or ignored.'}, {'letter': 'C', 'summary': 'A token exactly when
`destroys(f)` is written. Without it, the behaviour is B. With it, the behaviour is A: ending the token before
the run cancels, and after the run does nothing (the used-up mark).', 'code': '    # no cancel function: no
token\n    @ffi(param(cb, callback(once, user_data=req), threads(creator)))\n    safe fn fs_read_async(path:
CStrView, owns cb: fn(i32), req: *mut void)\n    # a cancel function: a token\n    @ffi(param(cb,
callback(once, user_data=req), threads(creator)), destroys(http_cancel))\n    safe fn http_get(url: CStrView,
owns cb: fn(i32), req: *mut void) -> Retained[ForeignCallback]', 'outcome': "Both declarations compile. The
example prints the bytes read, and `http_get`'s token cancels the request if it ends before the callback has
run."}]

recommendation:
{'letter': 'C', 'reason': 'The declaration means only what it says: a written `destroys` gives a token, and no
`destroys` gives none. Both real kinds of one-shot API become safe. Nothing runs twice, because the used-up
mark is checked when the token ends.', 'checklist': '2 (accepts both API kinds), 7 (general), 15 (meaning from
what is written), 3 (the existing used-up mark, no new run-time cost)'}

