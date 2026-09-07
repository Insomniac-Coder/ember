# Part XIII — Error Handling

* `[ERR-1]` Recoverable errors are values: `Result[T, E]` with `E: Error`. `Option[T]` for absence.
* `[ERR-2]` `expr?` in a function returning `Result[U, F]` evaluates to the `Ok` payload or returns `Err(F.from(e))`; in a function returning `Option`, returns `None`. Using `?` elsewhere is `E2180`.
* `[ERR-3]` `std.error.Error` is an interface (`Display + Debug + source()`); `Box[dyn Error]` is the "any error" type; `@derive(Error)` on an enum generates `Display` from `@error("message {field}")` variant attributes and `From` impls from `@from` fields.
* `[ERR-4]` `Result` and `Option` are ordinary enums in `std.core` with the full combinator set (`map`, `and_then`, `unwrap_or`, `unwrap_or_else`, `ok_or`, `expect`, `is_some`, `as_ref`, `take`, `context("msg")`).
* `[ERR-5]` `@must_use` is applied to `Result`; ignoring a `Result` value is `W2190` (error under `-Dwarnings`).
* `[ERR-6]` FFI status codes are converted at the boundary by the binding (Part XVI §6): a C function returning `VkResult` with an `@ffi(status=VkResult, ok=VK_SUCCESS)` overlay becomes a `Result[void, VkError]` in Ember.

```ember
@derive(Error, Debug)
enum AssetError:
    @error("file not found: {path}")
    NotFound(path: String)
    @error("io: {0}")
    Io(@from io.Error)
    @error("bad header at byte {offset}")
    Corrupt(offset: usize)

fn load(path: str) -> Result[Texture, AssetError]:
    bytes = fs.read(path)?                     # io.Error → AssetError.Io via From
    header = parse_header(bytes).ok_or(AssetError.Corrupt(0))?
    ...
```

---

