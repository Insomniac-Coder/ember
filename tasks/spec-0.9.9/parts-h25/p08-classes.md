---

# Part VIII — Classes and Reference Counting

Classes give Ember the object graphs Python programmers expect — shared objects, parent pointers,
observers — with deterministic destruction and no garbage collector.

```ember
class Node:
    name: String
    children: Array[Node] = []
    parent: Weak[Node] = Weak.empty()

    fn add(self, child: Node):
        child.parent = Weak(self)
        self.children.push(child)

fn main():
    root = Node("root")
    root.add(Node("leaf"))
    for c in root.children:
        match c.parent.upgrade():
            Some(p) => println(f"{c.name} -> {p.name}")
            None => println("orphan")
```

## VIII.1 Object representation

Every counted allocation — a class instance or a `Shared[T]` payload — is one heap block:

```text
offset  size  field
0       4     strong   u32   atomic iff the class is Sync or the block is a SyncShared
4       4     weak     u32   same atomicity
8       4     access   u32   dynamic exclusivity of a Shared payload (bit 31 writer, bits 0..30 reader count);
                             unused by class objects, whose fields carry their own (`[EXC-19]`)
12      4     flags    u32   bit 0 deinitialising, bit 1 pinned by foreign code
16      8     type     *const TypeInfo
24      …     base-class fields, then own fields (or the Shared payload), naturally aligned;
              each non-`Copy` field of a class that is not `@sync` is preceded by its u32 access word
```

* `[OBJ-1]` *(changed in 0.9.9)* The header is 24 bytes on 64-bit targets and is part of the runtime ABI
  (`[VER-4]`). Foreign code never reads it; it calls `ember_rt`. The per-field access words of
  `[EXC-19]` belong to the class's layout, not the header.
* `[OBJ-2]` A handle points at offset 0. An interface-typed handle is the same single pointer: the
  interface table is reached through the header's type information, so `Option[Handle]` is one pointer
  wide.
* `[OBJ-3]` `weak` starts at 1 on behalf of all strong handles. When `strong` reaches 0 the object is
  deinitialised (its `drop` chain and field drops run) and `weak` is decremented; the block is freed
  when `weak` reaches 0.
* `[OBJ-4]` Objects are allocated through the runtime allocator (`[RT-1]`).
* `[OBJ-5]` The runtime sets the deinitialising flag before the `drop` chain and checks `strong`
  after it and after the field drops; a nonzero count means the object was resurrected, which panics
  in every profile.

## VIII.2 Reference counting and its elision

* `[RC-1]` Copying a handle retains; dropping one releases; the release that reaches zero
  deinitialises. A strong count that would exceed its maximum panics (`[RT-7]`).
* `[RC-2]` **Guaranteed elisions.** No retain or release is emitted in the cases `[RC-2a]`–`[RC-2e]`;
  each is tested by counting retains in the emitted C.
* `[RC-2a]` Passing a handle to a borrowed parameter.
* `[RC-2b]` *(new in 0.9.9)* A handle read from a place and used only within one expression while that place is not
  written.
* `[RC-2c]` *(new in 0.9.9)* A retain immediately followed by a release of the same handle with no call or store between.
* `[RC-2d]` A handle stored into a field from a temporary (a move, not a retain).
* `[RC-2e]` Handles yielded by a `for` over a borrowed collection, unless the body stores, returns,
  consumes or `owned fn`-captures them.
* `[RC-3]` *(changed in 0.9.9)* Further elisions are allowed only when semantics are preserved (`[PHIL-5]`). An object is
  deinitialised when its last strong owner ends as the source says — a handle's scope ending, its
  reassignment, `mem.drop` — and never earlier: an elision removes a retain and a release only when it
  moves no `drop`, no `Weak.upgrade` outcome and no foreign release across that point. A surviving
  handle in a field, collection or capture keeps the object alive. The last use of a borrow ends the
  loan, not the owner (`[BRW-2]`) (ODR-063). `unsafe` code that keeps a raw pointer into an object
  keeps a handle to it alive for as long as it uses the pointer; `with h:` and `mem.keep_alive(h)`
  state that where no Safe use does.
* `[RC-4]` *(changed in 0.9.9)* A non-`Sync` class's counts, and a `Shared`'s, use plain loads and
  stores; a `Sync` class's, and a `SyncShared`'s, use a relaxed increment and an acquire-release
  decrement (`[RT-8]`).
* `[RC-5]` A borrow whose place goes through a class handle or `Shared` (`ref h.f`, a `Span` of an
  object's `Array` field, a `RefCell` guard obtained from one, a view built from these) is also a
  shared loan of the handle: the object stays alive until the borrow's last use, and a handle whose
  storage ends first is `E3060`. A handle stored where another handle can overwrite it — a field or
  element of a class object (`p.child`, `p.nodes[i]`) — cannot keep its object alive by its
  storage, so a borrow of it or through it goes through a retained copy that lasts to the end of the
  statement (`[EXP-4]`): `ref p.child.v` kept longer is `E3060`, whose help is to bind the handle to
  a local first (`c = p.child`). A call made on such a handle, or passed one, is given that copy for
  the call, so a method's `self` is the object it was called on until it returns, whatever the
  callee overwrites; a `mut` handle parameter is given the copy and it is stored back into the place
  after the call (`[FN-9]`), so a callee never holds anyone else's storage (ODR-065).
* `[RC-6]` `ember inspect` lists every retain and release that survives inside a loop, with the reason
  it could not be removed, because a surviving count operation blocks vectorisation.

## VIII.3 Exclusivity

Handles alias: two handles can reach one object. Ember enforces Swift's **law of exclusivity** for
the accesses that could otherwise invalidate memory, checking at run time what it cannot prove.

**Instantaneous accesses** — reading a field of a `Copy` type, or assigning a value to a field of a
`Copy` type (`h.x = 1.0`, `y = h.x`, `h.count += 1`) — complete in one step, cannot leave a dangling
reference, and are never checked.

**Long-term accesses** begin and end at run time and are checked. They are:

| Access | Kind | Duration |
|---|---|---|
| calling a method on a field (`h.items.push(x)`, `h.name.len()`) | write if the method takes `mut self`, else read | the call |
| calling an owned callable value stored in a field (`b.on_click(e)`, `[CLO-11]`) | write | the call |
| passing a field to a parameter (`f(h.items)`) | write for `mut`, read for borrowed, `E3012` for `owned` | the call; when the result borrows that parameter (`[LT-1]`), until the result's loan dies (`[EXC-18]`) |
| `ref h.f` / `ref mut h.f`, or a view of a field | read / write | until the loan dies (`[EXC-18]`) |
| iterating a field (`for x in h.items`) | read (write for `iter_mut`) | the loop |
| **assigning a field of a non-`Copy` type** (`h.items = []`, `h.name = other`) | write | the store and the drop of the old value |
| calling a `mut self` class method (`[CLS-7]`) | write, every field | the call |

Each access is to one field of the class's own (`h.inner.items` is an access to `inner`), and only
accesses to the same field conflict (`[EXC-19]`).

* `[EXC-1]` *(changed in 0.9.9)* Beginning a **write** access to a field while any access to the same
  field is active panics: `exclusivity violation: write access to Node.children while a read access
  to Node.children is active`, in every profile. There is no setting that removes this check
  (`[PHIL-13]`).
* `[EXC-2]` *(changed in 0.9.9)* Beginning a **read** access to a field while a write access to it is
  active panics likewise.
* `[EXC-19]` *(new in 0.9.9)* **Access state is per field.** Each non-`Copy` field of a class that is
  not `@sync` has its own access word (bit 31 writer, bits 0..30 reader count), stored in front of the
  field. A long-term access to a field checks and updates that word only, so reading one field while
  writing another never conflicts: `for c in self.children: self.log.push(c.name)` is accepted. A
  whole-object write (a `mut self` method, `[EXC-15]`) checks every field word of the object's
  **dynamic** class at entry — for an `open` class, through the list of access-word offsets in its type
  information, so a derived override reached from a base method is covered too — marks each as
  written, and clears them at return; a `mut self` call on `self` inside it is a reborrow (`[EXC-5]`)
  and marks nothing again. The cost is one word per non-`Copy` field, one check per such field on
  entry to a `mut self` method, and nothing per access. `Copy` fields have no word
  (`[EXC-17]`), and the fields of a `@sync` class need none (`[THR-1]`).
* `[EXC-16]` *(new in 0.9.9)* Assigning a value to a class field whose type is not `Copy` is a write access (table
  above): it conflicts with, for example, a `for` loop over the same field, which would otherwise read
  a buffer freed by the assignment.
* `[EXC-17]` *(new in 0.9.9)* Instantaneous writes are not checked against long-term accesses, so a
  `ref` or view of a `Copy` field (including a fixed-array field) observes writes made through other
  handles while it is live, as a C pointer would. It never dangles: a `Copy` field's storage lasts as
  long as the object, which the loan keeps alive (`[RC-5]`), and a borrow *through* a handle held in
  such a field holds its own copy of that handle (`[RC-5]`, ODR-065). For the same reason such a view gets no
  alias fact the other handles could break (`[SIMD-3]`).
* `[EXC-18]` *(new in 0.9.9)* The dynamic access that a borrow through a class handle begins — a
  `ref h.f`, a view of a field, a guard obtained from one — ends where the borrow's loan dies
  (`[BCK-2]`), in whichever function that is. A view of a field returned from a method carries its
  access to the caller and ends at the caller's last use of the view, on every path.
* `[EXC-3]` The compiler MAY remove a check only when it proves no conflicting access can occur: every
  access to the object in the interval goes through one handle local that is not reassigned, their
  intervals are ordered, and either the interval contains no call, no virtual or interface dispatch and
  no call through a callable value, or escape analysis proves that local is the only handle to the
  object. Conflicting accesses through one local that the compiler can see are rejected at compile
  time instead (`E3080`, shape X1).
* `[EXC-3a]` Every removed check is recorded, with the condition that justified it, and reported by
  `ember inspect --safety --elided-only`.
* `[EXC-4]` A `let` field is subject to the same rules: `let` fixes the binding, not the value
  (`[CLS-9a]`).
* `[EXC-5]` Accesses nested inside the same `mut self` method are reborrows and are not checked again.
* `[EXC-6]` *(changed in 0.9.9)* A panic from `[EXC-1]`/`[EXC-2]` names both the offending access and
  the active one it conflicts with (file, line, column, field). The `debug` runtime keeps a per-thread
  stack of active accesses for this; other profiles always name the offending access and its field,
  and name the active one where they can without any per-access cost.
* `[EXC-7]` *(changed in 0.9.9)* The opt-in lint `L3013` reports a long-term access held across a
  virtual or `dyn` call within one `open` class hierarchy, or across a call into C++ that may call back
  into Ember (`[FFI-39d]`).
* `[EXC-8]` When a loop makes repeated long-term accesses to one object whose identity, access kind and
  conflict set are loop-invariant, the compiler performs one check in the loop preheader and holds the
  access for the whole loop.
* `[EXC-9]` `[EXC-8]` applies only when the receiver's identity is loop-invariant, the access does not
  escape the loop, nothing in the loop can replace or publish the receiver, no call the compiler cannot
  see through can begin a conflicting access, and the hoisted access starts and ends where the original
  accesses did. If any condition is unknown, the per-access checks stay.
* `[EXC-10]` An inner loop reuses an outer loop's hoisted access when its accesses are a subset of it.
* `[EXC-11]` A hoisted access is invisible to programs: it cannot be named, stored or observed.
* `[EXC-12]` `ember inspect --safety` reports each check as `STATIC`, `DYNAMIC_PER_ACCESS` or
  `DYNAMIC_HOISTED_LOOP`, naming the loop and the proof for a hoisted one.
* `[EXC-15]` *(new in 0.9.9)* A `mut self` class method holds a write access to every field of the
  object (`[EXC-19]`) from entry to return; accesses to `self`'s fields inside it are covered by it and
  not checked individually.
* `[EXC-14]` *(changed in 0.9.9)* The 0.9.8 `exclusivity = "unchecked"` setting is removed. Code that
  cannot afford a check per access uses value types, `mut self` methods (`[EXC-15]`), or a query
  yielding `ref mut` (Part XII).

## VIII.4 Inheritance and dispatch

* `[DSP-1]` A call is dispatched statically when the receiver's static type is a final class or the
  method is not `virtual`.
* `[DSP-2]` A virtual call loads its slot from the object's type table; slots are assigned in
  declaration order, base first, and an `override` reuses the base's slot.
* `[DSP-3]` A call through an interface-typed handle finds the interface's table through the type
  information and caches the lookup for repeated calls on one handle.
* `[DSP-4]` `h as? D` walks the base chain; `a is b` compares addresses.
* `[DSP-5]` With the whole program visible, a virtual call with exactly one reachable implementation
  may become a direct call; `--emit-optimization-report` lists each.

## VIII.5 Weak handles and cycles

* `[WK-1]` A cycle of strong handles among class instances and `Shared` payloads is never collected;
  it leaks. This is a documented property. `Weak` breaks cycles; the compiler warns about the cycles it
  can see (`[WK-6]`); the debug runtime reports the ones it finds (`[WK-15]`).
* `[WK-11]` *(changed in 0.9.9)* `Weak(h)` creates a weak handle to a class object or a `Shared` or
  `SyncShared` payload without retaining it strongly; `Weak[O].empty()` (or `Weak.empty()` where the
  type is known) creates one that points nowhere. `Weak[O]` is `Copy`: copying increments the weak
  count, dropping decrements it.
* `[WK-2]` An object is deinitialised when its strong count reaches zero, whatever weak handles remain;
  those weak handles then fail to upgrade.
* `[WK-3]` `upgrade` returns `None` while the object's deinitialising flag is set, so a `drop` body
  cannot resurrect its object through a weak handle.
* `[WK-12]` *(changed in 0.9.9)* `w.upgrade() -> Option[O]` returns a retained strong handle while the
  object is alive and not deinitialising, and `None` otherwise. On a `@sync` object (or a
  `SyncShared`) it is a compare-exchange loop that fails once the strong count has reached zero, so an
  object another thread is releasing is never resurrected.
* `[WK-13]` A `Weak[Shared[T]]` refers to the same block as its `Shared[T]`.
* `[WK-14]` Ember's `Weak` and `Shared` never convert to or from C++'s `std::weak_ptr`/`std::shared_ptr`
  (`[SEL-2]`).
* `[WK-5]` The compiler builds a graph of strong ownership among class fields (class handles,
  `Shared`, and collections of them, after generic substitution); `Weak`, raw pointers, views and
  foreign handles are not strong edges; a foreign edge whose ownership is unknown is marked unknown.
* `[WK-6]` A cycle of strong edges in that graph is warning `L3001` at the field that closes the
  shortest cycle, naming the whole cycle and the field to make `Weak`, with a machine-applicable fix-it
  where the replacement changes no declared ownership contract. It never changes whether a program is
  accepted.
* `[WK-7]` Cycle analysis is conservative: a possible cycle suffices for `L3001`, it is never reported
  as certain without run-time evidence, generic types are analysed after substitution, and an opaque
  foreign edge is `unknown`, never assumed weak or strong.
* `[WK-8]` The run-time report (`[WK-15]`) lists each leaked strongly connected component with its
  class and field edges, whether static analysis predicted it, and the edge to weaken.
* `[WK-4]` The leak report names, for each leaked object on a cycle, the shortest strong cycle through
  it as a path of `Type.field` edges and the edge to weaken (`L3017`).
* `[WK-15]` *(new in 0.9.9)* `ember run` and `ember test` in the `debug` profile report leaked objects
  and their cycles when the program exits, by default; `--no-leak-check` turns the report off.
* `[WK-9]` `ember explain --cycle <path> <Class[.field]>` explains one cycle-capable class or field;
  `ember inspect --cycle <path>` prints the whole graph. A cycle through a generic container is shown with
  its instantiated types, and where no static cycle can be established the command says the edge is
  only dynamically cycle-capable.
* `[CLI-18]` For both commands `<path>` is a package directory (its manifest and imports) or a single
  `.em` file (`[CLI-4]`), and both analyse exactly the same ownership graph (`[WK-5]`), classifying
  each edge strong, weak or unknown and showing the shortest static cycle when there is one.

## VIII.6 Stack promotion

* `[OPT-1]` When escape analysis proves that no handle to an object outlives the function that
  created it, the compiler MAY place the object in the function's frame with the same header and run
  its `drop` at scope end. This cannot be observed.
