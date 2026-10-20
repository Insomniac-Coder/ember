// The C++ side of the FFI-41 tests: namespaces, a nested one, static members, an anonymous one.
#pragma once

namespace engine {
struct Vec2 {
    float x;
    float y;
};
inline float length2(Vec2 v) noexcept { return v.x * v.x + v.y * v.y; }

namespace math {
inline int square(int x) noexcept { return x * x; }
inline Vec2 scale(Vec2 v, float by) noexcept { return Vec2{v.x * by, v.y * by}; }
}  // namespace math

struct Clock {
    static int ticks() noexcept { return 42; }
};
}  // namespace engine

namespace {
inline int hidden() noexcept { return 1; }
}

inline int top() noexcept { return 7; }
