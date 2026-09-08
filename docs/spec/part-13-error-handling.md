# Part XIII — Error Handling

* `[ERR-1]` Recoverable errors are values: `Result[T, E]` with `E: Error`. `Option[T]` for absence.
* `[ERR-2]` `expr?` in a function returning `Result[U, F]` evaluates to the `Ok` payload or returns `Err(F.from(e))`; in a function returning `Option[U]`, `?` on an `Option` returns `None`. `?` on a `Result` in a function returning `Option` is `E2180`. **When the source error type and `F` are the same type, the conversion MUST be lowered to a move with no call.**
* `[ERR-3]` `std.error.Error` is an interface (`Display + Debug + source()`); `Box[dyn Error]` is the "any error" type; `@derive(Error)` on an enum generates `Display` from `@error("message {field}")` variant attributes and `From` impls from `@from` fields.
* `[ERR-4]` `Result` and `Option` are ordinary enums in `std.core` with the full combinator set (`map`, `and_then`, `unwrap_or`, `unwrap_or_else`, `ok_or`, `expect`, `is_some`, `as_ref`, `take`, `context("msg")`).
* `[ERR-5]` `@must_use` is applied to `Result`; ignoring a `Result` value is `W2190` (error under `-Dwarnings`).
* `[ERR-6]` FFI status codes are converted at the boundary by the binding (Part XVI §6): a C function returning `VkResult` with an `@ffi(status=VkResult, ok=VK_SUCCESS)` overlay becomes a `Result[void, VkError]` in Ember.
* `[ERR-7]` **Identity conversion.** `std.core` provides `extend[T] T implements From[T]: fn from(owned v: T) -> T: return v`. It is declared in the module that declares `From`, satisfying `[TYP-20]`, and does not overlap a user impl `From[A] for B` unless `A == B`, which no user may write (`E2041`). This is what makes `?` propagate an unchanged error type.
* `[ERR-8]` **Erasure conversion.** `std.core` provides `extend[E: Error] Box[dyn Error] implements From[E]`, so a function returning `Result[T, Box[dyn Error]]` may `?` any error. **`Box[dyn Error]` MUST NOT implement `Error`**: `[ERR-7]` and `[ERR-8]` would otherwise both supply `From[Box[dyn Error]] for Box[dyn Error]` and every use of the type would be `E2041`. Consequently `Error.source()` returns `Option[ref dyn Error]` (`[ERR-3]`) and never a `Box[dyn Error]`, and an already-erased error is re-erased by move under `[ERR-7]`.

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

