---

# Part XIII — Error Handling

A failure the caller can handle is a **value**: a function that can fail returns `Result`, and `?`
passes a failure up. A failure that means the program is wrong — an index out of range, an overflow,
a broken invariant — is a **panic**, which stops the process (`[PAN-1]`). There are no exceptions and
no hidden control flow: every place a function can return early is marked by `?` or `return`.

```ember
import std.fs

@derive(Error)
enum ConfigError:
    @error("line {line}: expected `key = value`")
    Syntax(line: int)
    @error("line {line}: `{key}` is not a number")
    NotNumber(line: int, key: String)

fn parse(text: str) -> Result[Map[String, int], ConfigError]:
    out: Map[String, int] = {}
    for n, line in text.lines().enumerate():
        key, sep, value = line.partition("=")
        if sep.is_empty():
            return Err(ConfigError.Syntax(line=n + 1))
        match value.trim().parse[int]():
            Ok(v):
                out[key.trim()] = v
            Err(_):
                return Err(ConfigError.NotNumber(line=n + 1, key=key.trim().to_string()))
    return Ok(out)

fn main() -> Result[void]:
    text = fs.read_to_string("app.cfg")?            # io.Error -> AnyError
    cfg = parse(text).context("reading app.cfg")?   # ConfigError -> AnyError, with a message
    println(cfg)
    return Ok(())
```

## XIII.1 `Result`, `Option` and `?`

* `[ERR-1]` *(changed in 0.9.9)* `Option[T]` is absence: `Some(v)` or `None`. `Result[T, E = AnyError]`
  is success or failure: `Ok(v)` or `Err(e)`, where `E: Error`. Both are ordinary enums in the prelude.
* `[ERR-9]` *(new in 0.9.9)* The error parameter defaults to `AnyError` (§XIII.2), so
  `fn load(path: str) -> Result[Texture]` is a function that can fail with any error. A library that
  wants callers to `match` on its failures names a concrete error type.
* `[ERR-2]` *(changed in 0.9.9)* `e?` on a `Result[T, E]` in a function returning `Result[U, F]`
  evaluates to the `Ok` payload, or returns `Err(F.from(err))`. On an `Option[T]` in a function
  returning `Option[U]` it evaluates to the `Some` payload or returns `None`. When `E` and `F` are the
  same type the conversion is a move with no call. `?` in a function returning neither is `E2180`.
  `?` on an `Option` in a function returning `Result`, or on a `Result` in a function returning
  `Option`, is `E2181`, whose fix-its are `.ok_or(err)?` and `.ok()?`.
* `[ERR-4]` `Option` and `Result` provide `is_some`/`is_none`, `is_ok`/`is_err`, `unwrap`, `expect`,
  `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`, `map`, `map_err`, `and_then`, `or_else`, `ok`,
  `err`, `ok_or`, `ok_or_else`, `filter`, `take`, `replace`, `as_ref`, `as_mut`, `iter`, and
  `context` (`[ERR-10]`). `unwrap` and `expect` panic with the error's `Display` text.
* `[ERR-5]` `Result` is `@must_use`: discarding one is `W2190` (an error under `-D warnings`). A value
  is used when it is bound, returned, passed, matched, or has `?` or a method applied.
* `[ERR-7]` **Identity conversion.** The prelude provides `From[T]` for every `T` (`from(v) = v`), so
  `?` propagates an unchanged error type. A user `extend T implements From[T]` is `E2041`.

## XIII.2 Error types and `AnyError`

* `[ERR-3]` *(changed in 0.9.9)* `Error` is the interface of error types: `Display + Debug` with
  `source(self) -> Option[ref dyn Error]` (default `None`) (§IV.8). `@derive(Error)` on an enum or
  struct implements `Display` from `@error("…")` attributes — one per variant, whose `{name}` and
  `{0}` placeholders name the variant's fields — and implements `Debug` if it is not already derived.
  A variant marked `@from` whose single field is an error type gets `From` for that type and returns it
  from `source`.
* `[ERR-8]` *(changed in 0.9.9)* **`AnyError`** is the prelude's "any error" type: an owned, boxed
  `dyn Error` plus an optional chain of context messages. It implements `From[E]` for every `E: Error`,
  so `?` converts any error into it, and `Display`, `Debug`, `source`, `downcast_ref[E]() -> Option[ref
  E]` and `is[E]() -> bool`. **`AnyError` does not implement `Error`** — otherwise `[ERR-7]` and
  `[ERR-8]` would both supply `From[AnyError]` for `AnyError`. An `AnyError` passed through `?` into
  another `AnyError` is moved (`[ERR-7]`).
* `[ERR-10]` *(new in 0.9.9)* `r.context(msg)` on a `Result[T, E]` returns a `Result[T, AnyError]`
  whose error, if any, displays as `msg: <original>`; `r.with_context(fn() => …)` builds the message
  only on failure. Context messages stack.
* `[ERR-11]` *(new in 0.9.9)* **The cost of `AnyError`.** Converting an error into `AnyError`
  allocates, once, on the failure path; that path carries `Alloc` (`[EFF-1]`). A concrete error type
  moves through `?` without allocating, which is why `@noalloc` code names its error types.
* `[ERR-12]` *(new in 0.9.9)* An `Err` returned from `main` (`[FN-8]`) prints `error: <Display>` to
  standard error, then one `caused by: <Display>` line for each `source` in the chain, and exits with
  status 1.

## XIII.3 Panics or errors

* `[ERR-13]` *(new in 0.9.9)* The standard library follows one convention, and user code SHOULD: an
  operation whose failure is a bug in the caller panics (`xs[i]` out of range, `m[k]` for a missing
  key, integer overflow, `unwrap` on `None`); an operation whose failure depends on the world or on
  input returns `Result` or `Option` (`xs.get(i)`, `m.get(k)`, `parse`, every I/O call). Each
  panicking accessor has a non-panicking twin, and the panic message names it
  (`index 7 out of range for length 3; use .get(i) for an Option`). The one exception is console input,
  which panics at end of input (`[STD-10]`), as Python's `input` raises.
* `[ERR-6]` A foreign function's status code becomes a `Result` at the binding (Part XVI): an
  `@ffi(status=VkResult, ok=VK_SUCCESS)` overlay turns a `VkResult` return into
  `Result[void, VkError]`.

Python's `try`, `except`, `raise` and `finally` are not Ember syntax; each is diagnosed with the Ember
form (`[DIA-21]`): `raise e` → `return Err(e)`, a `try` body → `?` on each fallible call, `except E`
→ `match` on the result, `finally` → `defer:`.
