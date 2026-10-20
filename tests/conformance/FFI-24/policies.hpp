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

// Computed specifications on members and on a template's instantiation.
struct Gauge {
    int level;
    Gauge(int level) noexcept(sizeof(int) == 4) : level(level) {}
    int read() const noexcept(sizeof(int) == 4) { return level; }
    int strain() const noexcept(sizeof(int) == 3) { return level * 2; }
};
template <class T>
T twice(T x) noexcept(noexcept(x + x)) { return x + x; }
