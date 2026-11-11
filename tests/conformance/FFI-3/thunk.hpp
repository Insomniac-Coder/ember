// The C++ side of the FFI-3 test: an inline function and a class member, neither of which has a
// C symbol a linker could name.
#pragma once

inline int triple(int x) noexcept { return x * 3; }

struct Counter {
    int value;
    explicit Counter(int start) noexcept : value(start) {}
    int bump() noexcept { return ++value; }
};
