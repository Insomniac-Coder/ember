// The C++ side of the FFI-11f tests: results whose lifetimes `[[clang::lifetimebound]]` states,
// written through a macro as portable headers write it.
#pragma once
#include <vector>
#if defined(__clang__)
#define LIFETIMEBOUND [[clang::lifetimebound]]
#else
#define LIFETIMEBOUND
#endif

inline std::vector<int> numbers() { return {4, 5, 6}; }
inline const int& pick(const std::vector<int>& values LIFETIMEBOUND, int i) noexcept { return values[static_cast<std::size_t>(i)]; }
inline int& larger(int& a LIFETIMEBOUND, int& b LIFETIMEBOUND) noexcept { return a > b ? a : b; }

class Shelf {
public:
    Shelf() : items{7, 8, 9} {}
    // The result borrows the other shelf, not this one.
    const int& last_of(const Shelf& other LIFETIMEBOUND) const noexcept { return other.items.back(); }
private:
    std::vector<int> items;
};
