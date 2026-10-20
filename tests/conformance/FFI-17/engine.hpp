// The C++ side of the FFI-17 tests: free functions, a class by value, references and exceptions.
#pragma once
#include <stdexcept>

struct Vec2 {
    float x;
    float y;
};

inline float dot(Vec2 a, Vec2 b) noexcept { return a.x * b.x + a.y * b.y; }
inline int scale(int value) noexcept { return value * ENGINE_SCALE; }
inline int checked_div(int a, int b) {
    if (b == 0) throw std::invalid_argument("division by zero");
    return a / b;
}
inline void bump(int& value) noexcept { value += 1; }
inline int twice(const int& value) noexcept { return value * 2; }
inline double sum(const Vec2& v) noexcept { return v.x + v.y; }
extern "C" inline int plain_c(int x) { return x + 100; }
