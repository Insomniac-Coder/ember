# K20-anyerror-send

Question: Can a worker thread return Result[int], whose error is AnyError?
Severity: high; impact rank 20
Members: J6-16; primary J6-16
Why together: Single question.
Critic's notes: An ordinary program is refused today without saying so. A refuses `?` of a one-thread error into AnyError even in programs with one thread. Say that cost in the option.

## Finding J6-16
Rules: ERR-8, ERR-3, THR-2, THR-8; file tasks/spec-0.9.9/parts/p13-errors.md lines 87-99; kind undecided_case; severity high
Quote from Hardened_11:
prelude's "any error" type: a `dyn Error` held alone (`owns dyn Error`) plus an optional chain of
  context messages. It implements `From[E]` for every `E: needs Error`, so `?` converts any error into it,

The finder's question:
Is `AnyError` `Send`? Does a worker thread that returns `Result[int]` compile? If AnyError is Send, may an
error that holds a one-thread owner be converted into one?

Program:
```
import std.fs
import std.thread

fn load(path: String) -> Result[int]:
    text = fs.read_to_string(path)?
    return Ok(text.len())

fn main() -> Result[void]:
    h = thread.spawn(owns fn() => load(String("a.cfg")))   # R = Result[int, AnyError] must be Send
    n = h.join()?
    println(n)
    return Ok(())
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.7)
Reason:
thread.spawn needs its result `R` to be Send (THR-10, p11 175-176). AnyError holds an `owns dyn Error` (ERR-8,
p13 93-94; std/src/core.em:92-99). `dyn I` is Send only when `I` requires it (THR-8, p11 43ff, spec 5376), and
Error requires only Debug and Display (ERR-3, p13 87-88; core.em:80). So by the rules AnyError is not Send and
a worker returning `Result[int]` is refused, which the text never says; making it Send instead would let
`From[E]` for every `E: needs Error` put a one-thread owner into a value sent to another thread. No verdict.

options:
[{'letter': 'A', 'summary': 'AnyError can go to another thread; it takes only errors that can', 'code': 'h =
thread.spawn(owns fn() => load("level1.cfg"))', 'outcome': "Compiles and prints the file's length. AnyError is
Send and ThreadShared, and `?` converts only errors that are too; an error holding a one-thread owner (a
`shares` field) is refused at the `?` with E2040 naming the field, help: hold a copy, or name the error
type."}, {'letter': 'B', 'summary': 'AnyError stays on its thread', 'code': 'fn load(path: StrView) ->
Result[int, io.Error]:', 'outcome': 'The `Result[int]` program is refused at the spawn line (E7015: the result
holds an `AnyError`, which may hold a one-thread owner); a worker names a concrete error type, as here, and
converts it after `join`.'}]

recommendation:
{'letter': 'A', 'reason': "Returning errors from worker threads is ordinary code; A makes it compile at no
cost (as Rust's anyhow requires Send and Sync), and the rare error holding a one-thread owner is refused at
the `?` with a line to write, never a race.", 'checklist': '1, 2, 3, 7, 8, 9'}

