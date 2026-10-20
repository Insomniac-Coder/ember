// The C++ side of the FFI-44 tests: one declaration of each row of the import table.
#pragma once
#include <functional>
#include <stdexcept>
#include <vector>

// `extern "C"` functions and POD structs: Part XVI's, automatic.
extern "C" {
struct Pair { int a; int b; };
inline int pair_sum(Pair p) { return p.a + p.b; }
}

// A standard-layout, trivially copyable class: automatic, by value.
struct Point {
    int x;
    int y;
    int manhattan() const noexcept { return x + y; }
};

// Other classes: automatic, opaque behind a pointer, their methods through thunks; a class with
// one C++ base has the base's methods too.
class Counter {
public:
    Counter() {}
    ~Counter() {}
    int bump() noexcept { return ++count; }
private:
    int count = 0;
};
class Named {
public:
    int id() const noexcept { return 7; }
};
class Tagged : public Named {
public:
    Tagged() {}
    int tag() const noexcept { return 3; }
};

// The standard library's types of [FFI-17a]: automatic.
inline std::vector<int> evens(int n) {
    std::vector<int> out;
    for (int i = 0; i < n; ++i) out.push_back(2 * i);
    return out;
}

// Explicit template instantiations: automatic; other templates: a native island.
template <class T> T largest(T a, T b) noexcept { return a > b ? a : b; }
template <class T> T smallest(T a, T b) noexcept { return a < b ? a : b; }

// Function-like macros: an overlay; `std::function`: never a parameter type.
#define SQUARE(x) ((x) * (x))
inline int call_with_one(std::function<int(int)> f) { return f(1); }

// Multiple inheritance: a native island.
class Both : public Named, public Counter {};

// Exceptions: automatic, with a derived policy.
inline int checked(int x) {
    if (x < 0) throw std::invalid_argument("negative");
    return x;
}

// Ownership the header does not state: unsafe until `adopt` or a contract.
inline int* made() { return new int(5); }
