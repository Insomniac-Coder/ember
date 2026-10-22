// The C++ side of accept_fixed_spans_cross_as_arrays.em: spans of fixed extent (ODR-200), C++20.
#pragma once
#include <span>

inline int sum3(std::span<const int, 3> values) noexcept { return values[0] + values[1] + values[2]; }
inline void zero2(std::span<int, 2> values) noexcept { values[0] = 0; values[1] = 0; }
inline int first_of(std::span<const int, 2> values) noexcept { return values[0]; }

class Grid {
public:
    Grid() noexcept : cells_{1, 2, 3, 4} {}
    ~Grid() {}
    std::span<const int, 4> cells() const noexcept { return std::span<const int, 4>(cells_); }
    std::span<int, 4> cells_mut() noexcept { return std::span<int, 4>(cells_); }
    std::span<const int, 4> raw_cells() const noexcept { return std::span<const int, 4>(cells_); }

private:
    int cells_[4];
};
