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

fn parse(text: StrView) -> Result[Map[String, int], ConfigError]:
    out: Map[String, int] = {}
    for n, line in text.lines().enumerate():
        key, sep, value = line.partition("=")
        if sep.is_empty():
            return Err(ConfigError.Syntax(line=n + 1))
        match value.trim().parse[int]():
            Ok(v):
                out[key.trim()] = v
            Err(_):
                return Err(ConfigError.NotNumber(line=n + 1, key=String(key.trim())))
    return Ok(out)

fn main() -> Result[void]:
    text = fs.read_to_string("app.cfg")?            # io.Error -> AnyError
    cfg = parse(text).context("reading app.cfg")?   # ConfigError -> AnyError, with a message
    println(cfg)
    return Ok(())
```

## XIII.1 `Result`, `Option` and `?`

* `[ERR-1]` *(changed in 0.9.9)* `Option[T]` is absence: `Some(v)` or `None`. `Result[T, E = AnyError]`
  is success or failure: `Ok(v)` or `Err(e)`. Both are ordinary enums in the prelude, and neither
  parameter is bounded: `Result[int, int]` is a `Result`. An operation that needs more of `E` says so
  (ODR-050): `?` into another error type needs `From[E]` (`[ERR-2]`), printing one needs `Display`,
  walking its causes needs `Error` (or is `AnyError`'s own, `[ERR-8]`), and converting one into
  `AnyError` needs `E: Error`.
* `[ERR-9]` *(new in 0.9.9)* *(changed in 0.9.15)* The error parameter defaults to `AnyError` (§XIII.2), so
  `fn load(path: StrView) -> Result[Texture]` is a function that can fail with any error. A library that
  wants callers to `match` on its failures names a concrete error type.
* `[ERR-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* `e?` on a `Result[T, E]` in a function returning `Result[U, F]`
  evaluates to the `Ok` payload, or returns `Err(F.from(err))`. On an `Option[T]` in a function
  returning `Option[U]` it evaluates to the `Some` payload or returns `None`. When `E` and `F` are the
  same type the error is handed over with no call. `?` in a function returning neither is `E2180`.
  `?` on an `Option` in a function returning `Result`, or on a `Result` in a function returning
  `Option`, is `E2181`, whose fix-its are `.ok_or(err)?` and `.ok()?`.
* `[ERR-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Option` and `Result` provide `is_some`/`is_none`,
  `is_ok`/`is_err`, `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`, `map`, `map_err`,
  `and_then`, `or_else`, `ok`, `err`, `ok_or`, `ok_or_else`, `filter`, `take`, `replace`, `as_ref`,
  `as_mut`, `iter`, and `context` (`[ERR-10]`). `unwrap` and `expect` panic with the error's `Display`
  text. `Option[T]`'s `fn as_ref() -> Option[ref T]` and `fn as_mut() -> Option[ref mut T]` give a
  view of the payload where it is, whatever `T` is (ODR-094): a pattern binds a `Copy` payload by
  value (`[GRM-13]`), so these are how a function hands out a view into an `Option` it holds; the
  view points into the `Option`, and `as_mut` is called through a name that may change it.
  The methods that take a function are eager: each takes its object over (`owns this`), takes the
  function as a type parameter (`[CLO-14]`) and calls it at most once, before it returns, handing
  the payload over to it, except `filter`, whose function reads it (ODR-025):
  * `Option[T]`: `[U, F: once fn(owns T) -> U]` above `fn map(owns f: F) owns this -> Option[U]`;
    `[U, F: once fn(owns T) -> Option[U]]` above `fn and_then(owns f: F) owns this -> Option[U]`;
    `[F: once fn(T) -> bool]` above `fn filter(owns f: F) owns this -> Option[T]`;
    `[F: once fn() -> Option[T]]` above `fn or_else(owns f: F) owns this -> Option[T]`;
    `[F: once fn() -> T]` above `fn unwrap_or_else(owns f: F) owns this -> T`; and
    `[E, F: once fn() -> E]` above `fn ok_or_else(owns f: F) owns this -> Result[T, E]`;
  * `Result[T, E]`: `[U, F: once fn(owns T) -> U]` above `fn map(owns f: F) owns this -> Result[U, E]`;
    `[F, G: once fn(owns E) -> F]` above `fn map_err(owns f: G) owns this -> Result[T, F]`;
    `[U, F: once fn(owns T) -> Result[U, E]]` above
    `fn and_then(owns f: F) owns this -> Result[U, E]`;
    `[F, G: once fn(owns E) -> Result[T, F]]` above `fn or_else(owns f: G) owns this -> Result[T, F]`;
    and `[F: once fn(owns E) -> T]` above `fn unwrap_or_else(owns f: F) owns this -> T`.
* `[ERR-5]` `Result` is `@must_use`: discarding one is `W2190` (an error under `-D warnings`). A value
  is used when it is bound, returned, passed, matched, or has `?` or a method applied.
* `[ERR-7]` **Identity conversion.** The prelude provides `From[T]` for every `T` (`from(v) = v`), so
  `?` propagates an unchanged error type. A user `extend T implements From[T]` is `E2041`.

## XIII.2 Error types and `AnyError`

* `[ERR-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Error` is the interface of error types:
  `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8).
  `@derive(Error)` on an enum or struct implements `Display` from `@error("…")` attributes — one per variant, whose `{name}` and
  `{0}` placeholders name the variant's fields — and implements `Debug` if it is not already derived.
  A variant marked `@from` whose single field is an error type gets `From` for that type and returns it
  from `source`.
* `[ERR-8]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **`AnyError`** is the
  prelude's "any error" type: a `dyn Error` held alone (`owns dyn Error`) plus an optional chain of
  context messages. It implements `From[E]` for every `E: Error`, so `?` converts any error into it,
  and `Display`, `Debug`, `source`, `[E]` above `fn downcast_ref() -> Option[ref E]`, and `[E]` above
  `fn is() -> bool`. **`AnyError` does not implement `Error`** — otherwise `[ERR-7]` and
  `[ERR-8]` would both supply `From[AnyError]` for `AnyError`. An `AnyError` passed through `?` into
  another `AnyError` is handed over (`[ERR-7]`). `AnyError.msg(owns text: String)`, a `static fn`,
  is an error whose text is `text`, so a function whose error type is `AnyError` fails with a message as `return
  Err(AnyError.msg(f"no such file: {p}"))`.
* `[ERR-10]` *(new in 0.9.9)* `r.context(msg)` on a `Result[T, E]` whose `E` converts into `AnyError`
  (`AnyError: From[E]`, `[ERR-8]`) returns a `Result[T, AnyError]`
  whose error, if any, displays as `msg: <original>`; `r.with_context(fn() => …)` builds the message
  only on failure. Context messages stack.
* `[ERR-11]` *(new in 0.9.9)* **The cost of `AnyError`.** Converting an error into `AnyError`
  allocates, once, on the failure path; that path carries `Alloc` (`[EFF-1]`). A concrete error type
  moves through `?` without allocating, which is why `@noalloc` code names its error types.
* `[ERR-12]` *(new in 0.9.9)* An `Err` returned from `main` (`[FN-8]`) prints `error: <Display>` to
  standard error, then one `caused by: <Display>` line for each cause in its chain — an `AnyError`'s
  own chain (`[ERR-8]`), or each `source` of an `E: Error` — and exits with status 1. An `E` that is
  `Display` but not `Error` has no chain and prints its message alone (ODR-050).

## XIII.3 Panics or errors

* `[ERR-13]` *(new in 0.9.9)* *(changed in 0.9.15)* The standard library follows one convention, and user code SHOULD: an
  operation whose failure is a bug in the caller panics (`xs[i]` out of range, `m[k]` for a missing
  key, integer overflow, `unwrap` on `None`); an operation whose failure depends on the world or on
  input returns `Result` or `Option` (`xs.get(i)`, `m.get(k)`, `parse`, every I/O call). Each
  panicking accessor has a non-panicking twin, and the panic message names it
  (`index 7 is out of bounds for a length of 3; use .get(i) for an Option`). The one exception is console input,
  which panics at end of input (`[STD-10]`), as Python's `input` raises.
* `[ERR-6]` *(changed in 0.9.10)* A foreign function's status code becomes a `Result` at the binding
  (Part XVI): an `@ffi(status=VkResult, ok=VK_SUCCESS, error=VkError)` overlay turns a `VkResult`
  return into `Result[void, VkError]`.

Python's `try`, `except`, `raise` and `finally` are not Ember syntax; each is diagnosed with the Ember
form (`[DIA-21]`): `raise e` → `return Err(e)`, a `try` body → `?` on each fallible call, `except E`
→ `match` on the result, `finally` → `defer:`.
