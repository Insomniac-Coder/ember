// The C++ side of the FFI-24 tests: functions whose exception policies are derived or declared.
#pragma once
#include <stdexcept>

struct Odd : std::runtime_error {
    Odd() : std::runtime_error("an odd number") {}
};

inline int halve(int x) {
    if (x % 2 != 0) throw Odd();
    return x / 2;
}
inline int risky(int x) {
    if (x < 0) throw std::runtime_error("negative");
    return x;
}
inline int quiet(int x) noexcept(sizeof(int) == 4) { return x + 1; }
inline int loud(int x) noexcept(sizeof(int) == 3) { return x + 2; }
inline int stated(int x) throw() { return x + 3; }
inline void fail() { throw 42; }
