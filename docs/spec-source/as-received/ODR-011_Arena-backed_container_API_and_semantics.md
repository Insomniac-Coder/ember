# ODR-011 — Arena-backed container API and semantics

-[ARN-5] `ArenaArray[T]`, `ArenaMap[K,V]` are container variants whose backing
-storage is an arena view; they are view types (`@view`) and follow `[TYP-15]`.
+[ARN-5] `ArenaArray[T]` and `ArenaMap[K,V]` are fixed-capacity, arena-backed
+view containers. Their backing storage is allocated from the supplying
+`Arena`; the containers do not own that storage and MUST NOT outlive the
+Arena region from which their storage derives.
+
+Both types are `@view` types and follow `[TYP-15]`.
+
+Arena-backed container storage is allocated once at construction. Neither
+`ArenaArray` nor `ArenaMap` grows or reallocates its backing storage after
+construction. This is intentional: existing references/views into elements
+must never be invalidated by container growth.

+## [ARN-5a] Canonical construction and provenance
+
+The canonical constructors are:
+
+```ember
+@borrows(arena)
+fn ArenaArray.with_capacity[T](
+    arena: Arena,
+    capacity: usize
+) -> ArenaArray[T]
+
+@borrows(arena)
+fn ArenaMap.with_capacity[K: Eq + Hash, V](
+    arena: Arena,
+    capacity: usize
+) -> ArenaMap[K, V]
+```
+
+The `arena` parameter is a shared (`borrowed`) parameter, matching `[ARN-1]`.
+Construction does not require `mut arena` because the underlying Arena
+allocation itself follows the existing shared `alloc*` contract.
+
+The returned container carries the Arena's region. `@borrows(arena)` is
+required whenever the constructor exposes the Arena-backed view through a
+user-visible return type.
+
+The container's own methods use `self`/`mut self` according to whether they
+only observe or mutate the already-allocated container storage. A mutating
+operation does not mutate the Arena allocation cursor.
+
+## [ARN-5b] Capacity and growth
+
+`ArenaArray` and `ArenaMap` are fixed-capacity containers.
+
+The initial `capacity` determines the maximum number of elements that can be
+stored. No operation may allocate replacement storage, grow the backing
+buffer, or move existing elements.
+
+When an insertion would exceed capacity, the operation MUST use the existing
+explicit failure channel:
+
+```ember
+Result[void, CapacityError]
+```
+
+and MUST NOT silently allocate, reallocate, or invalidate existing element
+views.
+
+`CapacityError` is the canonical recoverable capacity-exhaustion error for
+these containers. Panic is not used for ordinary capacity exhaustion.
+
+Because backing storage never moves, an element reference/view returned from
+an `ArenaArray` or `ArenaMap` remains valid until the element is removed or
+the supplying Arena region is otherwise ended under the ordinary borrow rules.
+
+## [ARN-5c] `ArenaArray[T]` minimum API
+
+The canonical minimum API is:
+
+```ember
+len(self) -> usize
+capacity(self) -> usize
+is_empty(self) -> bool
+get(self, index: usize) -> Option[ref T]
+get_mut(mut self, index: usize) -> Option[ref mut T]
+push(mut self, owned value: T) -> Result[void, CapacityError]
+insert(mut self, index: usize, owned value: T) -> Result[void, CapacityError]
+remove(mut self, index: usize) -> Option[T]
+clear(mut self) -> void
+iter(self) -> Iterator[ref T]
+iter_mut(mut self) -> Iterator[ref mut T]
+```
+
+Indexing follows the ordinary bounds-checking rules of `Array`/`Span`.
+
+`push`, `insert`, `remove`, and `clear` require `mut self`. Read operations
+use ordinary shared borrowing.
+
+`remove` moves the removed value out of the container. The container remains
+valid and existing references to the removed element become invalid according
+to the ordinary borrow/liveness rules.
+
+## [ARN-5d] `ArenaMap[K,V]` minimum API
+
+The canonical minimum API is:
+
+```ember
+len(self) -> usize
+capacity(self) -> usize
+is_empty(self) -> bool
+get(self, key: K) -> Option[ref V]
+get_mut(mut self, key: K) -> Option[ref mut V]
+insert(mut self, owned key: K, owned value: V)
+    -> Result[Option[V], CapacityError]
+remove(mut self, key: K) -> Option[V]
+contains_key(self, key: K) -> bool
+iter(self) -> Iterator[(ref K, ref V)]
+clear(mut self) -> void
+```
+
+`ArenaMap[K,V]` requires:
+
+```text
+K: Eq + Hash
+```
+
+The hashing/equality contract is the same as the ordinary `Map[K,V]` contract.
+
+If `insert` receives a key already present, it replaces the old value and
+returns `Some(old_value)`. A new key consumes one capacity slot. A new-key
+insertion at full capacity returns `Err(CapacityError)` and does not modify
+the map.
+
+## [ARN-5e] Element destruction
+
+`ArenaArray[T]` and `ArenaMap[K,V]` inherit the Arena destruction rule:
+their stored element/value types MUST satisfy `!needs_drop(T)` and
+`!needs_drop(K)`/`!needs_drop(V)` as applicable.
+
+Elements are not individually destroyed when the Arena is rewound or dropped.
+
+If a future arena-backed container is intended to support `needs_drop`
+elements, it MUST be a separately specified container with an explicit
+destruction/ownership model. `ArenaArray` and `ArenaMap` do not provide such
+a model.
+
+`remove` may move an initialized value out before Arena reclamation; it does
+not cause the Arena itself to become responsible for running that value's
+destructor.
+
+## [ARN-5f] Iteration and ordering
+
+`ArenaArray` iteration is index order and therefore deterministic.
+
+`ArenaMap` does not promise hash-bucket iteration order. Its iteration order is
+**unspecified but deterministic for an unchanged map state and implementation
+configuration**; programs MUST NOT rely on a particular bucket order.
+
+If a future stable-insertion-order map is needed, that is a separate API
+revision rather than an implicit change to `ArenaMap`.
+
+## [ARN-5g] Borrowing and invalidation
+
+Because these containers never relocate their backing storage:
+
+1. reading an element creates the ordinary shared borrow;
+2. mutating an element creates the ordinary mutable borrow;
+3. removal is prohibited while a live borrow of the removed element exists;
+4. `clear` is prohibited while any element borrow remains live;
+5. Arena reset/drop remains prohibited while any container or element view
+   derived from that Arena remains live.
+
+The compiler MUST NOT invent a container-specific invalidation mechanism.
+All lifetime enforcement is through the ordinary `[BRW-*]` and `[LT-*]` rules.
+
+## [ARN-5h] No hidden Arena mutation
+
+Container mutation (`push`, `insert`, `remove`, map insertion/update, and
+`clear`) modifies only the already-allocated container storage. It MUST NOT
+implicitly call `Arena.alloc*`, change the Arena allocation cursor, or create
+a second hidden Arena region.
+
+The constructor is the only operation that obtains the backing storage from
+the Arena.
+
+## [TST-24] Arena-backed container conformance
+
+Conformance MUST cover:
+
+1. construction with `@borrows(arena)`;
+2. returned provenance tied to the supplying Arena;
+3. fixed-capacity insertion;
+4. `CapacityError` at exhaustion;
+5. proof that no growth/reallocation occurs;
+6. `ArenaArray` indexing, get/get_mut, push, insert, remove, clear, and
+   iteration;
+7. `ArenaMap` lookup, mutable lookup, insertion, replacement of duplicate
+   keys, removal, contains, clear, and iteration;
+8. rejection of `needs_drop` element/key/value types;
+9. rejection of removal/clear while an affected element borrow is live;
+10. rejection of container use after the supplying Arena region ends;
+11. deterministic `ArenaArray` iteration;
+12. preservation of the documented unspecified-but-deterministic map order;
+13. verification that no container mutation changes the Arena cursor;
+14. generic element/key/value substitution and return-provenance through
+    generic wrappers.