// The C++ side of the FFI-41 tests: namespaces, a nested one, static members, nested types, an
// anonymous namespace.
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

// Nested types, and static data members: constants, and variables.
struct Grid {
    struct Cell {
        int row;
        int column;
    };
    enum class Fill { Empty, Wall };
    using Index = int;
    static constexpr int width = 8;
    static const int height = 4;
    static int made;
    static const double scale;
    static Cell corner() noexcept { ++made; return Cell{height - 1, width - 1}; }
    int cells;
};
inline int Grid::made = 0;
inline const double Grid::scale = 0.5;

inline int frames = 60;

// A variable holding a pointer, imported as any other (the owner's ruling of 2026-10-21).
inline Grid* last_grid = nullptr;
inline bool has_last_grid() noexcept { return last_grid != nullptr; }
}  // namespace engine

namespace {
inline int hidden() noexcept { return 1; }
}

inline int top() noexcept { return 7; }
