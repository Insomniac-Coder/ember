# 02 reader-2: the words of spec lines 1491-3177 (Parts V-IX)

Slice: `docs/spec-source/Ember_v0.9.10_Hardened_35.md` lines 1491-3177: Part V declarations, VI
expressions and statements, VII ownership and borrowing, VIII classes and reference counting, IX memory
facilities. Mains power checked at start: `Online`.

Terms used below, defined once:
- **word**: anything a programmer writes to say something: a keyword (`owned`), an `@` attribute
  (`@borrows`), or a library type or function that acts like a language word (`Box`, `mem.take`).
- **owner's test**: for every name on a line that binds it, point at the word that says what it is (the
  owner's own thing, a copy, or handed over) and the word that says whether it may change it.
- **plain English**: a reader who knows Python but not Rust can tell what the word does from the word.
- **on the thing**: the word sits on the thing it describes and names it (`fn() with copied score:`
  passes, `copied fn():` does not).
- **value type**: a struct, enum, tuple, `Array`, `Map`, `String`: one owner, moved on assignment.
- **class handle**: the pointer a `class` value is; copying it shares the same object and adds 1 to a
  count (a "retain"); dropping it subtracts 1 (a "release").

Code checked: every snippet is in `target/redesign3/02-reader-2/`, checked with
`target/redesign3/bin/ember.exe check <file>`. Outputs are pasted in section 3. Nothing was run.

## 1. What fails, most important first

These are the words in this slice that fail one of the owner's tests. Everything else in section 2 is
keep.

1. **`owned` does not mean the same for every type** (the owner's own rule: "`owned` means ownership
   handed over, nothing else").
   - `owned fn` lambda (2045, 2055, 2061, 2064, 2074, 2088-2090, 2131, 2136, 2493-2494, 2656): moves a
     `String`/`Array` capture, silently copies an `int`. Today: `t1` (button that never changes
     `score`) is accepted with no message; `t10` shows `names` moved (E3040) while `score` was copied in
     the same closure. Also the word sits on `fn`, not on the captures. Agreed row 6 already replaces it
     with `fn() with copied score:`; the same clause shape should carry `with owned names`. Verdict:
     the `owned fn` prefix is removed; the word `owned` stays for hand-over.
   - `owned` parameter (1547, 1560-1561, 1566, 2570): "moved in (copied if `Copy`; a handle is copied
     without a retain)". For an `int` the caller keeps its value (`t9` accepted); for an `Array` the
     caller loses it (`t6b`, E3040); for a class handle FN-9 (1572-1575) says the callee gets a
     *retained copy* and the caller keeps its own (`t6` accepted, `a.v` used after `take(a)`). Three
     meanings by type. And 1561 ("copied without a retain") contradicts 1573-1574 ("a copy, retained")
     inside this slice.
   - `mem.drop(owned x)` (2261-2263): for a class handle, the variable **is** moved ("though a handle
     is `Copy`", `t7`: E3040), the opposite of what an ordinary `owned` parameter does to a handle
     (`t6`). The same word on the same type means two things depending on which function declares it.
2. **`mut self` on a class method does not mean "may change"** (1740-1742, 2701, 2718-2729, 2759,
   2780-2785). Every class method may already write its fields with plain `self` (CLS-7, 1738-1740;
   `t4`: `fn open(self): self.angle = 90.0` accepted). On a class, `mut self` means "hold a write lock
   on every field for the whole call so per-access checks can be removed". On a struct, `mut self`
   means "may change" and plain `self` cannot write (`t14`, E3023). The agreed row 3 shape
   (`fn note(self, msg: String) changes log:`) says what a method changes; the whole-object write
   should be that clause naming every field (placeholder `changes all`), and `mut self` on a class is
   refused with that advice. `mut self` in `init` (1658-1659, 1717-1718) "is accepted and means the
   same" as `self`: a form accepted that does not do what its words say: refuse it, message "`init`
   may always set fields; write `fn init(self, …)`". (`t4` shows it accepted today.)
3. **Plain `self`, a plain parameter and a `for` variable may change a class object with no word**:
   `fn open(self)` (1705-1706), `fn heal_all(players: Array[Player])` with
   `for p in players: p.health = 100.0` (2566-2568, `t8` accepted), `fn bump(h: Node): h.v += 1`
   (`t11` accepted). FN-1 (1554-1555) says the borrowed default "cannot assign through it", which is
   false for class handles. The owner's test asks for the word that says "may change"; there is none.
   For the words: `self`, the default parameter mode and `for` keep their spelling, but the default is
   not "stated and cannot surprise" for a class. Phase 2 (area C/B) owns the fix; the agreed row 3
   `changes` clause is the obvious shape.
4. **`let` on a field says "fixed" and the field still changes** (1693, 1748-1750, 2757). "`let`
   fixes the binding, not the value": `let items: Array[int]` refuses `self.items = []` (`t2b`,
   E2104) but accepts `self.items.push(v)` (`t2`). For a value-type field there is no "binding"
   separate from the value: both lines change the same list. The owner's test fails: the word that
   says "may change" says no and the field changes. Keep the word, change its meaning for value-type
   fields to "never changes after `init`" (a handle field keeps "never re-pointed"), or rename it.
5. **`.clone()` does not copy a class object** (2273-2274: "Cloning a class handle copies the handle").
   `b = a.clone(); b.health = 5` changes `a` (`t5` accepted); `b = a.clone(); mem.drop(a)` keeps the
   object alive through `b` (`t15` accepted). In Python, `copy.copy` makes a new object. Agreed row 9
   makes `copied items` the word for a copy; `.clone()` merges into `copied`, which must mean a new
   value for every type it is allowed on (refused on a class unless the class says how to copy
   itself). Every help text that says "clone the value" (E3040, 2252) changes with it.
6. **`Box`**: the owner already named it (`owned` is the one-owner field that replaces `Box`). Uses in
   this slice: 1536, 1902, 2120, 2179, 2183, 2348, 2509, 2863, 2893-2896, 2986. Each needs its new
   spelling: `Box[dyn Iterator[Item = Y]]` (2179) becomes `owned dyn Iterator[Item = Y]`; the
   allocator parameter (`Box[T, A]`, 2986) and `b.get()` (2893) need a place in the new form.
7. **`Cell` and `RefCell` (and their guards `Ref`, `RefMut`, methods `borrow`, `borrow_mut`) do not say
   what they do** (1536, 1555, 2258, 2870, 3099-3137). `fn advance(s: Sprite): s.frame.set(…)` (3105)
   changes `s` with nothing on its line saying so; the only word is `Cell` on a field elsewhere, and
   "cell" does not say "may change through a read-only borrow". Placeholders for the word planner:
   `Cell[T]` -> `Settable[T]` (get/set/replace, never lends a reference); `RefCell[T]` -> `Checked[T]`
   with `read()`/`write()` (the names `RwLock` already uses; CELL-8, 3132, says they have the same
   shape), guards `ReadGuard`/`WriteGuard`.
8. **`@borrows(p)` sits on a separate line above the function, not on the result it describes**
   (2372-2398, 2470). The agreed rows 3 and 7 put what a function changes or reads after the
   signature (`changes log`, `reads renderer`). The same place fits: placeholder
   `fn find_word(haystack: str, needle: str) -> Option[str] from haystack:`. Rename (position and
   spelling); meaning kept.
9. **`assert_disjoint` does not assert** (3145-3174): it returns `Result` and never panics;
   `assert_disjoint_or_panic` is the one that asserts. A Python reader expects `assert` to stop the
   program. Placeholder: `check_disjoint` (returns `Ok`/`Err`) and `assert_disjoint` (panics).
10. **`str`** (1504, 1533, 1797, 1818, 1928, 1954-1959, 1987, 2325, 2357-2361, 2392, 2510, 2533,
    2877, 3031): a borrowed view of someone else's text. In Python `str` is the owned string, so the
    word misleads in exactly the wrong direction (agreed row 5 depends on telling a view from your own
    `String`). `t12`: `fn first(xs: Array[String]) -> str` makes `names.push` fail later (E3021) and
    nothing in the signature's words says the result points into `xs`. Rename candidate; Part IV
    (reader 01) owns the type, so the builder should merge both readers' verdicts.
11. **`MutSpan` needs `mut` said twice**: `fn normalize(mut xs: MutSpan[f32])` (2520, 3147, FN-1a
    1562-1564). Without `mut` the write is refused (`t3`, E3023). The type already says "may change";
    the second `mut` adds nothing a reader needs. Keep both words; flag to the builder as an annoyance
    (area A).
12. **`with h:` as "keep `h` alive"** (2663) does not say what it does; `mem.keep_alive(h)` (same line)
    does. Merge into `mem.keep_alive(h)`. (`t13` shows `with a:` accepted.) Separately, agreed row 6
    gives `with` a second job, the capture list `fn() with copied score:`; the builder should check the
    two uses read as one meaning ("with these, do this").
13. **`upgrade`** (2604, 2660, 2810-2815) is Rust's word for "get the object if it is still alive".
    Placeholder `w.get() -> Option[O]` or `w.alive()`. Low priority.
14. **`@no_runtime_checks`** is reserved "naming the version" (1875-1876), which reads as a promise of a
    switch that turns safety checks off, while EXC-1 (2708-2709) and DSJ-8 (3173) say no such setting
    exists. Remove it from the reserved list.
15. **A bare interface name is a counted, shared, dynamically dispatched handle with no word**
    (`s: Shape`, OBJ-2, 2631-2634), while a one-owner one needs `dyn` (`Box[dyn I]`). The owner's own
    example writes the one-owner case as `no_share(dyn Pool)`; the shared case has no word at all.
    Keep `dyn`; flag the bare form to the builder (area D).

Smaller notes (keep, but the builder should know):
- `fn(A) -> R` means a compile-time bound as a parameter type and an owned callable value elsewhere
  (CLO-3, 2077-2099); same spelling, two meanings by position.
- CLO-15 (2129-2137) decides capture by move from which function receives the lambda
  (`thread.spawn(fn() => work(x))` moves `x`): the compiler decides what the code does not say. With
  the agreed `with owned x` / `with copied x` clause the rule is not needed.
- A `ref mut` local written `r = e` writes through `r` (BRW-1, 2286-2287); in Python `r = e` re-binds.
  For the readability attacker (18).
- `Copy` covers class handles, `Shared`, `Weak` (2240, 2270, 2806, 2904), whose "copy" is a share
  (retain), not a copy.
- `void` is the type and `()` the value (`Result[void, E]` returns `Ok(())`, 1615-1616).
- `@assume_noalloc(call)` is an attribute on an expression (1873) while ATT-2 (1837) lists only four
  attributes allowed on statements and ATT-4 says one attribute per line. Reader 03 owns `[EFF-7]`.
- `@sync` (2622, 2713, 2730, 2813, 2910): a Java reader reads "synchronized" (locked methods); it
  means "shareable between threads, fields fixed after `init`". Reader 03 owns `[THR-1]`.
- `handle` names two things: a class handle (counted pointer, prose and error text) and `Handle[T]`
  (pool index + generation, 3089). Only `Handle[T]` is written in code; keep.

## 2. Every word in the slice

Columns: lines (uses in code or in a code span); meaning; same meaning everywhere and for every type;
plain English; on the thing; verdict. "Y" = yes, "N" = no. Line numbers are of the spec file.

### 2a. Keywords (from II.4 and contextual ones) that appear in this slice

| word | lines | meaning in this slice | same | plain | on thing | verdict |
|---|---|---|---|---|---|---|
| `owned` | param 1547, 1560-1566, 2570; receiver 1581, 2377, 2971, 3046; lambda 1908, 2045, 2055, 2061-2064, 2074, 2088-2090, 2109, 2131, 2136, 2493-2494, 2656; callable param 2081, 2125; loop 1982, 1986; `mem` 2261-2263; 2187, 2354, 2370, 2375, 2697 | hand-over of a value; for `Copy` a copy; for a class handle a retained copy (param) or a move (`mem.drop`); on `fn` a mix of move and copy per capture | N | Y for hand-over | N for `owned fn` | keep the word; remove the `owned fn` prefix (captures named in `with owned x, copied y`); make `owned` on a parameter mean hand-over for every type (section 1.1) |
| `mut` | param 1547, 1558-1564, 2065, 2083, 2296-2297, 2441, 2520, 3147; handle param 1576-1577, 1744; struct receiver 1581, 1633, 2499, 2935, 2968, 2990; class receiver 1740-1742, 2701, 2718-2729, 2759-2760, 2780-2785; `init` 1658, 1717; gen method 2201; `static mut` 1805; `ref mut` 1906, 2280-2320, 2379, 2543, 2698, 2901, 2938, 2971, 3029 | may change the caller's place (value) / re-point the caller's handle / lock every field (class receiver) / nothing (init) / foreign mutable static | N | Y | Y | keep for parameters, `ref mut`, `static mut`; class `mut self` merges into the agreed `changes` clause; `mut self` in `init` refused (section 1.2) |
| `ref` | type 1981, 1944, 2537, 2898, 2954; `ref mut` type 1988, 2543, 2901, 2938, 3029; expression 2280-2281, 2303, 2311, 2319-2320, 2668, 2674, 2698, 2742; return `ref mut` 1906 | a borrow of someone's place; `ref mut` may change it | Y | Y ("refers to") | Y | keep (agreed row 8 keeps `ref mut path`) |
| `self` | struct 1630-1635, 1674; class 1696-1706, 1712-1745, 2199-2219, 2596-2598; init 1640, 1657; 2364, 2373-2383 | the receiver; read-only on a struct, writable on a class, the frame-retained handle in a `gen fn` | N (by type) | Y | Y | keep; the class default needs the agreed `changes` clause (section 1.3) |
| `Self` | 1654-1656, 1718, 1799, 2273 | the type being declared | Y | Y | Y | keep |
| `let` | 1693, 1748-1750, 2757 | class field assignable only in `init`; contents still changeable | N (by type) | N for value-type fields | Y | keep the word, change its meaning on value-type fields or rename (section 1.4) |
| `static` | items 1613, 1803-1819, 2086, 2245, 3000, 3023; region 1580, 2399, 2494 | one value for the whole program / the region that lasts the whole program | Y | Y-ish (C term) | Y | keep |
| `const` | 1613, 1628, 1794-1802 | compile-time constant, inlined | Y | Y | Y | keep |
| `fn` | 1547-3147 (93 lines) | declaration, lambda, callable type; as a parameter type a bound, elsewhere a stored value (CLO-3) | N (by position) | Y | Y | keep; note CLO-3 and CLO-15 (section 1, smaller notes) |
| `once` | 1590, 1593, 2117-2125 | callable at most once | Y | Y | Y | keep |
| `gen` | 2149, 2155, 2168-2216 | the function is a generator | Y | Y (short) | Y | keep (the return type `Generator[…]` says it too; no test fails) |
| `yield` | 1614, 2152-2220, 2495 | suspend and produce a value | Y | Y (Python's) | Y | keep |
| `unsafe` | `unsafe fn` 1591-1597, 3009, 3039-3046; block 2538, 3016, 3091; interface 2976, 2989, 2997; field/static 1806-1807; `unsafe asm` 3037; 3020-3033, 3171 | on a declaration: the caller or implementer must keep a promise the compiler cannot check; on a block: the programmer keeps it here | N (two roles, as Rust) | partly | Y | keep (with the `# SAFETY:` note, UNS-8); area E may split the two roles |
| `dyn` | 1735, 2179, 2753, 2767, 2863, 2893-2895 | dynamic dispatch through an interface table | Y | N (jargon) but owner used it | Y | keep; flag bare interface types (section 1.15) |
| `some` | 2175 | an opaque type that implements an interface | Y | Y ("some iterator") | Y | keep |
| `class` | 1654, 1692, 1702, 1799, 2213, 2556, 2591, 2856, 2864 | heap object with identity, counted, shared by handles | Y | Y (Python's reference meaning) | Y | keep |
| `struct` | 1503, 1624, 1646, 1656, 1760, 1798, 2047, 2437, 2916, 3077, 3102, 3108 | value type | Y | Y | Y | keep |
| `enum` | 1654, 1665, 1669, 1798, 2861 | tagged union | Y | Y | Y | keep |
| `interface` | 1755, 1772, 1782, 2896, 2989 | set of required methods | Y | Y | Y | keep |
| `implements` | 1763, 1768, 2997 | `extend T implements I:` | Y | Y | Y | keep |
| `extend` | 1654, 1752, 1763, 1768, 1780, 1799, 2997 | add methods / implement an interface for a type | Y | Y | Y | keep |
| `open` | 1692, 1723, 2719, 2767 | class may be subclassed | Y | Y | Y | keep |
| `abstract` | 1723 | class may be subclassed, not built directly | Y | Y | Y | keep |
| `virtual` | 1699, 1724, 1733, 2307, 2790 | method may be replaced in a subclass | Y | Y-ish (C++ term) | Y | keep |
| `override` | 1705, 1724-1727, 2792 | this method replaces a virtual one | Y | Y | Y | keep |
| `super` | 1728, 1731 | the base class's `init` | Y | Y (Python's) | Y | keep |
| `pub` | 1507-1524, 1547, 1624-1626, 1639-1640, 1694, 1755, 1800, 2989, 3039 | visible to dependants; `pub(package)`, `pub(read)` read-only outside the module | Y | Y | Y | keep |
| `import` | 1502-1519, 1542, 1816, 2211, 3075, 3145 | bind a module | Y | Y | Y | keep |
| `from` | 1512-1519, 1542, 1816, 1869 (`@from`), 2211, 3075, 3145 | `from m import x` | Y | Y | Y | keep |
| `as` | 1512 (import alias), 1684 (enum to number), 1931 (`as`, `as?`, `as!`), 2795, 3010 | rename on import; convert; `as?` downcast to `Option`, `as!` downcast or panic | N (import vs conversion, as Python's `import … as`) | Y; `as?`/`as!` N (Swift marks) | Y | keep (reader 01 owns TYP-6) |
| `is` | 1925-1937, 2795 | handle identity; `is None` | Y | Y (Python's) | Y | keep |
| `with` | 1896-1897, 2013-2014, 2663, 2964, 3112, 3133 | bind for a block and drop at its end; `with e:` keep a temporary alive; `with h:` keep a handle alive | N (`with h:`) | Y except `with h:` | Y | keep; `with h:` merges into `mem.keep_alive(h)` (section 1.12) |
| `defer` | 2015-2018, 2230 | run a block when the enclosing block exits | Y | Y | Y | keep |
| `match` | 1675, 1681, 2011, 2022, 2604, 3148 | pattern match | Y | Y | Y | keep |
| `if` `elif` `else` | 1890-1896, 1968-1976, 2010, 2021-2033, 2157, 3013 | conditionals; `else` of a loop runs without `break` | Y | Y | Y | keep |
| `while` | 1894, 1976, 2010, 2151 | loop | Y | Y | Y | keep |
| `for` | 1548, 1966-1997, 2156-2161, 2218, 2442, 2522, 2562-2567, 2603, 2655, 2699, 2717, 2922, 3150-3153 | iterate; element is `ref T`, for a class a handle that may write the object | N for class elements | Y | Y | keep; the class default flagged (section 1.3) |
| `in` | same as `for`; 1924 (`x in c`) | loop source; membership | Y (Python's two) | Y | Y | keep |
| `break` `continue` `return` `pass` | 2010-2024; returns passim; `pass` 1646, 1700, 2020 | as Python | Y | Y | Y | keep |
| `and` `or` `not` | 1890, 1924-1926, 1936, 1970, 1977-1978 | logic; `not in`, `is not` | Y | Y | Y | keep |
| `true` | 1694 | boolean | Y | Y | Y | keep |
| `void` | 1602, 1609, 1615-1616, 2171, 2179, 2876, 2955 | no value | Y | Y | Y | keep (note `Ok(())` value vs `void` type) |
| `type` | 1776-1777 | associated type in an interface | Y | Y | Y | keep |
| `extern` | 1592-1597, 2097, 2198, 3022 | C function / C function pointer | Y | Y | Y | keep |
| `c` | 1504 (`import c`), 1843/3057 (`@layout(c)`) | the C language | Y | Y | Y | keep |
| `overlay` | 1854, 1872 (named as "overlay item") | a Part XVI construct, named only | - | - | - | keep (reader 03 owns it) |
| `rename` | 1871 (`@ser(rename="…")`) | serialised field name | Y | Y | Y | keep |

### 2b. Library words that act as language words

| word | lines | meaning | same | plain | on thing | verdict |
|---|---|---|---|---|---|---|
| `Box` | 1536, 1902, 2120, 2179, 2183, 2348, 2509, 2863, 2893-2896, 2986 | one owner on the heap; `Box[dyn I]` | Y | N | Y | rename -> `owned T` field (owner's decision; section 1.6) |
| `Shared` | 1536, 2270, 2509, 2611-2621, 2665-2668, 2801-2820, 2865, 2880, 2897-2910 | counted shared ownership on one thread; `get`/`get_mut` checked at run time | Y | Y | Y | keep (agreed row 9 names a `shared` way out; the builder should make them one word) |
| `SyncShared` | 2615, 2666, 2805, 2814, 2866, 2906-2908 | `Shared` across threads, read-only payload | Y | partly ("Sync") | Y | keep |
| `Weak` | 1536, 2270, 2594, 2597, 2660, 2802-2823, 2867, 2880, 2905 | handle that does not keep its object alive | Y | Y | Y | keep |
| `upgrade` | 2604, 2660, 2810-2815 | get the object if still alive | Y | N | Y | rename (placeholder `get`/`alive`; section 1.13) |
| `Cell` | 1536, 1555, 2258, 2327, 2870, 3099-3124 | value replaceable through a read-only borrow | Y | N | Y | rename (placeholder `Settable`; section 1.7) |
| `RefCell` | 1536, 1555, 2348, 2669, 2870, 3099-3137 | run-time checked borrow | Y | N | Y | rename (placeholder `Checked` with `read`/`write`; section 1.7) |
| `Ref`, `RefMut` | 2510, 3125-3129 | guards of a `RefCell` | Y | N | Y | rename with `RefCell` |
| `borrow`, `borrow_mut` | 3112, 3125-3126 | open a read / write guard on a `RefCell` | Y | N (jargon) | Y | rename with `RefCell` (`read`/`write`) |
| `Span` | 1534, 1909, 1928, 1948, 1984, 1999, 2155, 2325, 2416-2420, 2439, 2510-2546, 2668, 3012, 3147, 3164 | read-only view of contiguous elements | Y | Y | Y | keep |
| `MutSpan` | 1534, 1562, 1928, 1984, 1999, 2315, 2325, 2438, 2517-2549, 2920, 2938-2946, 2972, 3147, 3164 | changeable view | Y | Y | Y | keep (needs `mut` twice; section 1.11) |
| `str` | 1504, 1533, 1797, 1818, 1928, 1954-1959, 1987, 2325, 2335, 2357-2361, 2392, 2510, 2533, 2877, 3031 | view of text | Y | N (Python's `str` is owned) | Y | rename (section 1.10; merge with reader 01) |
| `String` | 1533, 1603, 1757, 1819, 1932, 1944, 1953-1965, 2028, 2048, 2348, 2533, 2557, 2592, 2862 | owned text | Y | Y | Y | keep |
| `clone` (`.clone()`, `Clone`) | 1537, 1566, 1648-1651, 2252, 2273, 2894-2896 | deep copy for values; a second handle for a class | N | N for classes | Y | merge into `copied` (agreed row 9; section 1.5) |
| `Copy` | 1537, 1556, 1560, 1566, 1623, 1643-1652, 1683, 1686, 1749, 1797, 1907, 1943, 1997, 2064, 2070, 2093, 2240, 2262, 2269-2272, 2327, 2347-2353, 2375, 2388, 2403, 2539, 2623, 2687-2688, 2700-2737, 2806, 2869, 2904, 3047, 3089, 3117-3124, 3137 | duplicated on use, source stays valid; for handles a share | N (handles) | Y for values | Y | keep; handles should not be called `Copy` (section 1, smaller notes) |
| `Drop`, `drop` | 1537, 1643, 1650, 1729, 1736, 1745, 2248, 2499-2515 | the end-of-life method | Y | Y | Y | keep |
| `init` | 1640, 1657-1659, 1696, 1712-1734, 1748 | constructor | Y | Y (Python's `__init__`) | Y | keep |
| `Default` | 1537, 1652, 1870, 2260, 2508, 2939-2943, 3117 | the type has a default value | Y | Y | Y | keep |
| `mem.take` | 1559, 1910, 2260, 2265, 2507 | move out, leave the default | Y | Y | Y | keep |
| `mem.replace` | 1559, 1910, 2261 | move out, put a new value | Y | Y | Y | keep |
| `mem.swap` | 2261 | exchange two places | Y | Y | Y | keep |
| `mem.drop` | 1575, 2261-2263, 2500, 2659 | end a value now; moves even a class handle | N vs `owned` param | Y | Y | keep; the clash with `owned` is section 1.1 |
| `mem.forget` | 2263 | never drop | Y | Y | Y | keep |
| `mem.keep_alive` | 2663 | keep a handle alive to here | Y | Y | Y | keep (absorbs `with h:`) |
| `Option`, `Some`, `None` | 1532, 1685, 1926-1944, 1978, 1983, 2177-2178, 2314-2316, 2381, 2392-2394, 2508, 2537, 2604-2606, 2631, 2810-2813, 2947, 2954-2959, 3012-3017, 3086, 3090, 3128 | maybe a value | Y | Y | Y | keep |
| `Result`, `Ok`, `Err` | 1532, 1602-1616, 1930, 2951, 2955-2959, 3097, 3149-3159 | success or error | Y | Y | Y | keep |
| `Mutex` | 1804, 1816, 1819, 2871, 3132 | lock | Y | Y | Y | keep |
| `RwLock` | 1804, 2511, 2871, 3132 | read/write lock | Y | Y | Y | keep |
| `Atomic` | 1804, 2871 | atomic value | Y | Y | Y | keep |
| `Once` | 1805 | run-once cell for a static | Y | Y | Y | keep |
| `Pool` | 2869, 3075, 3082, 3093 | slots reached by generation-checked handles | Y | Y | Y | keep |
| `Handle` | 2631, 2869, 3075, 3089-3092 | index + generation into a `Pool` | Y | Y | Y | keep |
| `Send` | 1537, 2106, 2907-2908, 3032, 3047, 3123 | may move to another thread | Y | partly | Y | keep (reader 03) |
| `Sync` | 1537, 1746, 1804-1806, 2107, 2615, 2665-2666, 2907-2908, 3032, 3047, 3123, 3132 | may be shared between threads | Y | partly | Y | keep (reader 03) |
| `Arena` | 2402, 2868, 2920, 2927, 2933, 2945-2946 | bump allocator, all freed at once | Y | Y | Y | keep |
| `FixedArena` | 2402, 2868, 2945-2948 | arena that never grows | Y | Y | Y | keep |
| `ScopedArena` | 2402, 2963 | arena scope freed at block end | Y | Y | Y | keep |
| `UnsafeCell` | 2327, 3045-3049 | raw interior mutability | Y | N | Y | keep (unsafe, expert-only; renamed with `Cell` if the planner wants a family) |
| `MaybeUninit` | 2258, 2969-2972, 3036 | maybe-uninitialised storage | Y | Y | Y | keep |
| `Zeroable` | 2915, 2941, 2976-2981, 3036 | all-zero bytes are valid | Y | Y | Y | keep |
| `Generator` | 1537, 2149, 2155, 2170, 2174, 2207, 2216 | generator frame type | Y | Y | Y | keep |
| `Iterator` | 1537, 1983, 2175-2179 | `next()` | Y | Y | Y | keep |
| `Never` | 1504, 1531, 2228 | no value ever | Y | Y | Y | keep |
| `get_mut` | 2901-2903, 2955, 2959 | changeable borrow of a `Shared` / container element | Y | Y | Y | keep |
| `as_mut_span` | 1563, 2528, 2536, 2548 | changeable view of an `Array` | Y | Y | Y | keep |
| `reborrow` | 2539 | lend a `MutSpan` on again | Y | N (jargon) | Y | keep (rarely written) |
| `iter_mut` | 2313, 2522, 2542, 2699, 2956 | iterate yielding `ref mut` | Y | Y | Y | keep |
| `get_unchecked` | 2538, 3023, 3091 | index with no check (unsafe) | Y | Y | Y | keep |
| `assert_disjoint` | 3145-3169 | check two views do not overlap, return `Ok`/`Err` | Y | N (does not assert) | Y | rename (section 1.9) |
| `assume_disjoint` | 3171 | unchecked promise (unsafe) | Y | Y | Y | keep |
| `transmute` | 3023, 3034 | reinterpret bits (unsafe) | Y | N (jargon) | Y | keep (expert, unsafe) |
| `asm` | 3037 | inline assembly | Y | Y | Y | keep |
| `Allocator`, `Global`, `System` | 2986-3000 | allocator interface; the configured one; the platform's | Y | Y | Y | keep |
| `size_of`, `align_of`, `offset_of` | 3060, 3070 | compile-time layout queries | Y | Y | Y | keep |

### 2c. Attributes (every `@name` in the slice; most appear only in the V.8 table, 1840-1876, whose rule lives in another Part)

| attribute | lines | meaning | verdict |
|---|---|---|---|
| `@derive`, `@no_derive` | 1623, 1644-1653, 1686, 1842, 2915, 2978 | ask for / opt out of a standard implementation | keep |
| `@borrows` | 1859, 2372-2398, 2404, 2408, 2470 | the result borrows exactly these parameters | rename: move onto the signature as a clause (section 1.8) |
| `@view` | 1856, 2325 | documentation that the type holds views; checked one way (`E2030`) | keep |
| `@must_drop` | 1861, 2202, 2263 | the value's `drop` must run (no `forget`, no `yield` while live) | keep |
| `@sync` | 1857, 2622, 2713, 2730, 2813, 2910 | class shareable between threads, fields fixed after `init` | keep here; reader 03 judges the spelling (section 1, smaller notes) |
| `@safety` | 1860, 2197, 3039 | the caller's obligation of an `unsafe fn` | keep |
| `@static_safe` | 1846, 3048, 3169 | Part X effect | keep (reader 03) |
| `@noalloc` | 1846, 1932, 1960, 2233, 2506, 2948, 3169 | no heap allocation | keep |
| `@nosync`, `@noblock`, `@noio`, `@nolock`, `@realtime`, `@deterministic` | 1846, 1833, 3169 | Part X effects | keep (reader 03) |
| `@nopanic` | 1846, 1875 | `@nopanic(explicit)`; bare form reserved | keep (reader 03 should say what `explicit` means in plain words) |
| `@overflow` | 1847 | overflow policy | keep |
| `@fastmath`, `@fp` | 1848 | float contraction | keep |
| `@inline`, `@noinline`, `@cold`, `@hot` | 1849 | code generation hints | keep |
| `@simd`, `@parallel`, `@unroll` | 1837-1838, 1850 | loop attributes | keep |
| `@must_use` | 1851 | result must be used | keep |
| `@deprecated` | 1852 | deprecated item | keep |
| `@export`, `@export_table` | 1853, 2198 | C export | keep |
| `@ffi` | 1808, 1854 | foreign item facts | keep |
| `@test`, `@bench`, `@should_panic` | 1855 | tests | keep |
| `@reflect` | 1858 | reflection | keep |
| `@allow` | 1837, 1862 | silence named warnings | keep |
| `@non_exhaustive` | 1863 | enum may gain variants | keep |
| `@component`, `@soa` | 1864 | data layout of components | keep |
| `@always_specialize`, `@never_specialize` | 1865 | generic code generation | keep |
| `@reloadable`, `@noreload`, `@renamed_from`, `@reinit_on_reload`, `@allow_reload_terminate` | 1866 | hot reload (Annex B) | keep |
| `@prelude` | 1867 | standard-library prelude module | keep |
| `@attribute` | 1828, 1868 | user-defined attribute struct | keep |
| `@error`, `@from` | 1869 | error enum variants | keep |
| `@default` | 1870 | default variant | keep |
| `@ser` | 1871 | serialisation | keep |
| `@grade` | 1872 | overlay trust grade | keep (reader 03) |
| `@assume_noalloc` | 1873 | an expression attribute inside `unsafe` | keep; placement clash with ATT-2/ATT-4 flagged |
| `@layout`, `@packed`, `@align` | 1843, 3055-3069 | layout | keep |
| `@repr` | 1664, 1844, 3062 | enum discriminant type | keep |
| `@gpu_layout` | 1845 | GPU buffer layout | keep |
| `@gpu`, `@allocator` | 1876 | reserved | keep reserved |
| `@no_runtime_checks` | 1876 | reserved | remove (section 1.14) |

Words on the must-cover list that occur in this slice only as ordinary English, not as the language
word: `where`, `false`, `safe`, `threads`, `union`, `copy`. Not in the slice at all: `comptime`, `cpp`,
`language`, `hide`, `async`, `await`, `macro`, `Retained`, `.copy()`, and `@dataclass`. None of the
must-cover words that appear in the slice as a language word is left unjudged.

## 3. What today's Ember does (real output)

All files in `C:\Users\ism19\Code\ember\target\redesign3\02-reader-2\`, checked with
`target/redesign3/bin/ember.exe check <file>`.

`t1_owned_fn_int.em`, the button with `owned fn(): score += 1`: **no output (accepted)**. The
button changes a private copy of `score`.

`t10_owned_fn_two_types.em`: `f = owned fn(): println(score, names.len())`, then `score += 1` and
`names.len()`:
```
error[E3040]: `names` has been moved out of
 --> t10_owned_fn_two_types.em:8:13
  |
4 |     f = owned fn(): println(score, names.len())
  |         --------------------------------------- moved here
...
8 |     println(names.len())
  |             ^^^^^ used here after the move
  |
  = help: clone the value, or borrow it instead of moving it
```
(`score` was copied, so `score += 1` is accepted; `names` was moved.)

`t6_owned_handle.em` (`fn take(owned n: Node)`, then `a.v` after `take(a)`): only
`warning[L1001]: \`x\` is never read`; **accepted**. `t9_owned_int_param.em` (same with `int`): only
the L1001 warning; **accepted**. `t6b_owned_array.em` (same with `Array[int]`):
```
error[E3040]: `a` has been moved out of
 --> t6b_owned_array.em:7:13
  |
6 |     x = take(a)
  |              - moved here
7 |     println(a.len())
  |             ^ used here after the move
  |
  = help: clone the value, or borrow it instead of moving it
```

`t7_mem_drop_handle.em` (`mem.drop(a)` on a class handle, then `a.v`):
```
error[E3040]: `a` has been moved out of
 --> t7_mem_drop_handle.em:7:13
  |
6 |     mem.drop(a)
  |     ----------- moved here
7 |     println(a.v)
  |             ^^^ used here after the move
  |
  = help: clone the value, or borrow it instead of moving it
```

`t4_init_mut_self.em` (`fn init(mut self)` on a class, and `fn open(self): self.angle = 90.0`):
**no output (accepted)**.

`t14_struct_self.em` (`fn bump(self): self.n += 1` on a struct):
```
error[E3023]: cannot mutate borrowed parameter `self`
 --> t14_struct_self.em:5:9
  |
4 |     fn bump(self):
  |             ---- `self` is borrowed here
5 |         self.n += 1
  |         ^^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut self: Counter`
```

`t2_let_field.em` (`let items: Array[int]` in a class, `self.items.push(v)` in a method): **no output
(accepted)**. `t2b_let_field_assign.em` (`self.items = []` in a method):
```
error[E2104]: cannot assign to `let` field `Log.items` outside its `init`
 --> t2b_let_field_assign.em:8:9
  |
8 |         self.items = []
  |         ^^^^^^^^^^
```

`t3_mutspan_no_mut.em` (`fn zero(xs: MutSpan[f32]): xs[0] = 0.0`):
```
error[E3023]: cannot mutate borrowed parameter `xs`
 --> t3_mutspan_no_mut.em:2:5
  |
1 | fn zero(xs: MutSpan[f32]):
  |         ---------------- `xs` is borrowed here
2 |     xs[0] = 0.0
  |     ^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut xs: MutSpan[f32]`
```

`t5_clone_class.em` (`b = a.clone(); b.health = 5` on a class) and `t15_clone_after_drop_advice.em`
(`b = a.clone(); mem.drop(a); b.v`): **no output (both accepted)**.

`t8_for_class.em` (`for p in players: p.health = 100.0` with `players: Array[Player]` borrowed) and
`t11_mut_handle_param.em` (`fn bump(h: Node): h.v += 1`; `fn repoint(mut h: Node): h = Node(0)`):
**no output (both accepted)**.

`t12_str_return.em` (`fn first(xs: Array[String]) -> str`, then `names.push` while `n` lives):
```
error[E3021]: `names` is borrowed here and mutably borrowed elsewhere
 --> t12_str_return.em:7:5
  |
6 |     n = first(names)
  |               ----- borrow of `names` starts here
7 |     names.push("bob")
  |     ^^^^^^^^^^^^^^^^^ conflicting access here
8 |     println(n)
  |             - borrow later used here
```

`t13_with_keepalive.em` (`with a:` on a class handle): **no output (accepted)**.

## 4. Error messages seen here that do not give the line to write

Reader 04 owns the full table; these are the four I met while checking the words above.

| code | where | today | proposed |
|---|---|---|---|
| E2104 | `t2b`, line 8 | "cannot assign to `let` field `Log.items` outside its `init`", no help | "`Log.items` is `let`: it is set once in `init`. To change it later, drop `let`: `items: Array[int]`" |
| E3040 | `t7` (class handle after `mem.drop`) | help "clone the value" (a clone of a handle keeps the object alive, undoing the drop) | "`a` was ended by `mem.drop(a)` on line 6; use it before that line, or remove the `mem.drop`" |
| E3040 | `t10` (capture moved by `owned fn`) | help "clone the value, or borrow it instead of moving it" | "`names` was handed to the function on line 4; to keep using it, write `fn() with copied names:` or drop `owned`" (wording follows the final capture words) |
| E3040 | `t6b` (moved into an `owned` parameter) | help "clone the value…" with no line | "`a` was handed to `take` on line 6; to keep it, write `take(copied a)`" (agreed row 9 word) |
Not counted: E3023 on a struct method (`t14`) does give a line, `mut self: Counter`; `t16_mut_self_typed.em`
(`fn bump(mut self: Counter):`) is accepted with no output, so the advice works, though the spec writes
the receiver as `mut self` (1633) and the help should match it. E3023's second and third helps
(`RefCell`, `class`) and its long note are noise for a beginner who only forgot `mut`.
