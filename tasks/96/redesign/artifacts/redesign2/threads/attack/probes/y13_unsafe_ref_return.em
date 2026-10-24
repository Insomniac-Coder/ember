# PROPOSED rule (8.5): "a reference made from a raw pointer gets the region of the enclosing unsafe block".
# Then a safe wrapper cannot return a reference into the memory it owns; Rust ties it to &self
# (r06_raw_ref_borrows_self.rs compiles).
struct Wrapper:
    raw: *const int

    fn get(self) -> ref int:
        unsafe:
            # SAFETY: raw points at memory this Wrapper owns for as long as it lives
            return ref *self.raw
