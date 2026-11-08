# K07-raw-pointer-struct-safe-call

Question: May a safe C call take a struct that holds raw pointers?
Severity: high; impact rank 7
Members: J8-08; primary J8-08
Why together: Single question.
Critic's notes: Memory corruption from safe code. A changes XVI.4's vkCreateBuffer entry. Rewrite that example once, together with K46 (J8-14, `device: borrowed`). The example's `ref_to_ptr(ref mut x)` takes whatever spelling K51 (J2-11) keeps.

## Finding J8-08
Rules: FFI-10, FFI-56, FFI-11, FFI-30a; file tasks/spec-0.9.9/parts/p16-ffi.md lines 59-61, 82-84, 239-241, 596-598; kind undecided_case; severity high
Quote from Hardened_11:
(verdict 136): `safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and
  result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete (given with

The finder's question:
May a `safe fn` (or an `@ffi(safe)` function-pointer typedef) take a `@layout(c)` struct holding raw pointer
fields, by value or through a `borrowed one` pointer? Safe code can fill such a struct with `ref_to_ptr`
(safe, ODR-075) and any length, and C then reads or writes through it.

Program:
```
@layout(c)
struct Buf:
    data: *mut u8
    len: usize

unsafe extern "C":
    @ffi(no_callbacks)
    safe fn buf_zero(b: Buf)          # compiles by FFI-10's list?

fn main():
    x: u8 = 0
    buf_zero(Buf(data=ref_to_ptr(ref mut x), len=4096))   # all safe code; C writes 4096 bytes
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.65)
Reason:
FFI-10 (p16-ffi.md 59-61) accepts a `safe fn` when "every parameter and result is a scalar, a `@layout(c)`
value type, or a pointer whose contract is complete", and lines 82-84 refuse a raw pointer even with complete
facts. Wrapped in a `@layout(c)` struct, the same raw pointer passes the list. Contract words attach only to
parameters, and FFI-30a (210-212) bars an overlay from touching fields, so nothing can state a field pointer's
count or lifetime. Safe code can fill such a field with any address (`ref_to_ptr` is safe, p04-types.md
340-342, ODR-075). FFI-56 (596-598) uses the same list for `@ffi(safe)` typedefs. The spec's own XVI.4 overlay
asserts `vkCreateBuffer(pCreateInfo: borrowed one)` safe, and its create-info struct holds
`pQueueFamilyIndices` and `pNext`. No verdict covers this.

options:
[{'letter': 'A', 'summary': 'A `@layout(c)` type that holds a raw pointer, by value or behind a contracted
pointer, counts as a raw pointer. A `safe fn` (or an `@ffi(safe)` typedef) taking one is refused. The safe
interface is an ordinary wrapper that builds the struct from a checked view inside `unsafe:`.', 'code':
'unsafe extern "C":\n    @safety("`buf.data` points at `buf.len` writable bytes")\n    @ffi(no_callbacks)\n
unsafe fn clear_pixels(buf: PixelBuf)\n\nfn clear(ref mut bytes: MutSpan[u8]):\n    # SAFETY: the struct
describes exactly the bytes of `bytes`\n    unsafe:\n        clear_pixels(PixelBuf(data=bytes.as_mut_ptr(),
len=bytes.len()))', 'outcome': "`safe fn clear_pixels(buf: PixelBuf)` is E5002, naming the field `data`. The
wrapper compiles, and safe callers can only clear bytes they own. The XVI.4 example's `vkCreateBuffer` becomes
an `unsafe fn` entry with `@safety`, or gets a wrapper."}, {'letter': 'B', 'summary': "As today. `safe fn` is
the declarer's promise and also covers the struct's pointer fields.", 'code': 'unsafe extern "C":\n
@ffi(no_callbacks)\n    safe fn clear_pixels(buf: PixelBuf)', 'outcome': 'The example compiles, and C
overwrites 4,095 bytes past `x` (memory corruption in safe code). `ember tcb` lists the assertion.'}]

recommendation:
{'letter': 'A', 'reason': "A raw pointer is refused on a safe C call whether it travels alone or in a struct,
and C's view of the memory comes from a span the compiler checked. This keeps Rust-level safety with no
run-time cost.", 'checklist': '5 and 6 (one rule for raw pointers, no struct-shaped way around it), 8, 2
(correct programs keep a safe wrapper), 7'}

