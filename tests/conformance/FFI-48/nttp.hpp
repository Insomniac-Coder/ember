// The C++ side of the FFI-48 test of a non-type template parameter of class type (C++20).
#pragma once

struct Point { int x; int y; };
template <Point P> struct Fixed {
    int sum() const noexcept { return P.x + P.y; }
};
inline Fixed<Point{1, 2}> origin_fixed() noexcept { return {}; }
inline int plain_sum(int a, int b) noexcept { return a + b; }
