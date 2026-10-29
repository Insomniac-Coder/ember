// The C++ side of accept_ffi_const_asserts_a_method_only_reads_its_object.em: a `const` method of a
// class with `mutable` state whose body calls through a function pointer, which the importer cannot
// follow, so it takes the method as changing its object.
#pragma once

inline int same(int value) noexcept { return value; }

class Ledger {
public:
    Ledger() noexcept {}
    int balance() const noexcept { return read_(total_); }

private:
    mutable int reads_ = 0;
    int total_ = 5;
    int (*read_)(int) noexcept = same;
};
