// The C++ side of the FFI-42 tests: a value class and a container with overloaded operators.
#pragma once
#include <stdexcept>
#include <vector>

struct Vec2 {
    int x;
    int y;
    Vec2 operator+(const Vec2& other) const noexcept { return Vec2{x + other.x, y + other.y}; }
    Vec2 operator-(const Vec2& other) const noexcept { return Vec2{x - other.x, y - other.y}; }
    Vec2 operator-() const noexcept { return Vec2{-x, -y}; }
    Vec2& operator+=(const Vec2& other) noexcept {
        x += other.x;
        y += other.y;
        return *this;
    }
    bool operator==(const Vec2& other) const noexcept { return x == other.x && y == other.y; }
    bool operator!=(const Vec2& other) const noexcept { return !(*this == other); }
    bool operator<(const Vec2& other) const noexcept { return x < other.x; }
    Vec2 operator*(int k) const {
        if (k < 0) throw std::invalid_argument("a negative scale");
        return Vec2{x * k, y * k};
    }
};

class Grid {
public:
    Grid() : cells(4, 0) {}
    int& operator[](int i) noexcept { return cells[static_cast<std::size_t>(i)]; }
    const int& operator[](int i) const noexcept { return cells[static_cast<std::size_t>(i)]; }
    int total() const noexcept {
        int sum = 0;
        for (int cell : cells) sum += cell;
        return sum;
    }
private:
    std::vector<int> cells;
};

namespace bank {
// Operators declared outside the class, as C++ code often writes them.
struct Money {
    long long cents;
};
inline Money operator+(const Money& a, const Money& b) noexcept { return Money{a.cents + b.cents}; }
inline Money& operator+=(Money& a, const Money& b) noexcept {
    a.cents += b.cents;
    return a;
}
inline bool operator==(const Money& a, const Money& b) noexcept { return a.cents == b.cents; }

// An operator defined as a friend in its class, which only argument-dependent lookup finds.
struct Points {
    int value;
    friend Points operator*(const Points& a, int k) noexcept { return Points{a.value * k}; }
};
}
