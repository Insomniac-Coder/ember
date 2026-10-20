// The C++ side of the FFI-17b tests: a class template and function templates.
#pragma once

namespace geo {
template <class T>
struct Pair {
    T first;
    T second;
    T sum() const noexcept { return first + second; }
};
}  // namespace geo

template <class T>
T biggest(T a, T b) noexcept { return a < b ? b : a; }

template <class T>
T smallest(T a, T b) { return a < b ? a : b; }
