--- ODR-014 — Span/MutSpan API completion
+++ ODR-014 — Owner-approved completion

+## [SPN-4] Canonical iterator types
+
+`std.collections` publicly exports:
+
+    SpanIter[T]
+    MutSpanIter[T]
+    SpanChunks[T]
+    MutSpanChunks[T]
+
+None of these types is a prelude name.
+
+They are named `@view` iterator types and implement the existing iterator
+interface using its associated-type form.
+
+    SpanIter[T]
+        implements Iterator[Item = ref T]
+
+    MutSpanIter[T]
+        implements Iterator[Item = ref mut T]
+
+    SpanChunks[T]
+        implements Iterator[Item = Span[T]]
+
+    MutSpanChunks[T]
+        implements Iterator[Item = MutSpan[T]]
+
+The iterator types are ordinary library types, not compiler intrinsics and not
+new ownership categories.
+
+## [SPN-5] Iterator construction
+
+The canonical APIs are:
+
+    Span.iter(self) -> SpanIter[T]
+    MutSpan.iter(self) -> SpanIter[T]
+    MutSpan.iter_mut(mut self) -> MutSpanIter[T]
+
+`MutSpan.iter()` creates a shared reborrow for the iteration and therefore
+prevents conflicting mutation for the duration of the resulting iterator.
+
+`MutSpan.iter_mut()` creates a mutable reborrow. It does not consume the
+underlying `MutSpan` ownership; after the iterator's borrow ends, the original
+`MutSpan` may be used again according to ordinary NLL/reborrow rules.
+
+The iterator cannot outlive the `Span`/`MutSpan` from which it was created.
+
+## [SPN-6] Chunk APIs
+
+The canonical APIs are:
+
+    Span.chunks(self, n: usize) -> SpanChunks[T]
+    MutSpan.chunks(self, n: usize) -> SpanChunks[T]
+    MutSpan.chunks_mut(mut self, n: usize) -> MutSpanChunks[T]
+
+For `n > 0`, iteration partitions the source span into consecutive,
+non-overlapping chunks:
+
+    [0 .. n]
+    [n .. 2n]
+    ...
+    [k .. len]
+
+The final chunk MAY contain fewer than `n` elements.
+
+`chunks_mut` produces disjoint `MutSpan[T]` values. A mutable chunk's region and
+storage identity cover only its own range. No two yielded mutable chunks may
+overlap.
+
+The implementation MUST preserve the existing `[BRW-5]` disjointness rules;
+`chunks_mut` is a sanctioned way to obtain multiple mutable borrows, not a new
+aliasing mechanism.
+
+## [SPN-7] Zero chunk size
+
+`chunks(0)` and `chunks_mut(0)` are invalid arguments and MUST panic through
+the existing bounds/argument failure mechanism.
+
+They MUST NOT:
+
+    return an empty iterator;
+    return a Result;
+    yield zero-width chunks;
+    loop indefinitely.
+
+This behavior is identical across supported profiles unless an explicit future
+language revision changes the failure policy.
+
+## [SPN-8] Raw pointer extraction
+
+The canonical raw-pointer APIs are:
+
+    Span.as_ptr(self) -> *const T
+    MutSpan.as_ptr(self) -> *const T
+    MutSpan.as_mut_ptr(mut self) -> *mut T
+
+Pointer extraction itself is safe because obtaining a raw pointer does not
+dereference or access the pointed-to memory.
+
+Use of the returned raw pointer for dereference, pointer arithmetic that is not
+otherwise proven safe, reads, writes, or construction of references requires
+the existing `unsafe` rules.
+
+`MutSpan.as_mut_ptr` requires a mutable reborrow of the `MutSpan`; it does not
+consume the underlying view.
+
+The returned pointer MUST NOT be interpreted as carrying a safe Ember borrow
+guarantee. Raw-pointer validity is governed by the existing `unsafe` contract.
+
+## [SPN-9] Pointer lifetime and provenance
+
+`as_ptr`/`as_mut_ptr` do not extend the lifetime of the source view.
+
+Obtaining a raw pointer does not retain, pin, or otherwise prolong the source
+storage lifetime.
+
+A program leaving the safe borrow model through a raw pointer is responsible
+for maintaining the pointer's validity under the existing `unsafe` rules.
+
+## [SPN-10] Public module boundary
+
+The iterator types are exported from:
+
+    std.collections
+
+They are public so that their concrete associated `Item` types are expressible
+in ordinary Ember type checking, but they are not prelude names.
+
+No opaque iterator-return mechanism is introduced by this completion.
+
+## [TST-25] Span API conformance
+
+Conformance MUST cover:
+
+1. `Span.iter()` yielding `ref T`;
+2. `MutSpan.iter()` yielding shared `ref T`;
+3. `MutSpan.iter_mut()` yielding `ref mut T`;
+4. reuse of the original `MutSpan` after the mutable iterator borrow ends;
+5. rejection of conflicting access while an iterator/reborrow remains live;
+6. `chunks(n)` for normal and final partial chunks;
+7. `chunks_mut(n)` producing non-overlapping mutable chunks;
+8. mutable-chunk alias rejection and interaction with `split_at_mut`;
+9. `chunks(0)` and `chunks_mut(0)` panicking through the existing failure path;
+10. safe extraction through `as_ptr()` and `as_mut_ptr()`;
+11. unsafe-only dereference/use of raw pointers;
+12. pointer extraction not extending the source lifetime;
+13. generic `T` substitution through all iterator and chunk types;
+14. ABI/layout erasure of region metadata.
+
+This completion does not introduce a new borrowing or lifetime mechanism.