// The C++ side of the FFI-32 tests: a class by value, and an opaque one.
#pragma once
#include <stdexcept>
#include <vector>

// Standard-layout and trivially copyable: a struct by value, with a constructor and methods.
struct Point {
    int x;
    int y;
    Point(int x, int y) noexcept : x(x), y(y) {}
    int sum() const noexcept { return x + y; }
    void shift(int by) noexcept { x += by; y += by; }
    static Point origin() noexcept { return Point(0, 0); }
};

// Private state and a std::vector: opaque, made only by its constructor.
class Counter {
public:
    explicit Counter(int start) : values_{start} {}
    ~Counter() { ++destroyed(); }
    int total() const noexcept {
        int sum = 0;
        for (int value : values_) sum += value;
        return sum;
    }
    void add(int value) {
        if (value < 0) throw std::out_of_range("negative");
        values_.push_back(value);
    }
    int count() const noexcept { return static_cast<int>(values_.size()); }
    static int& destroyed() noexcept { static int count = 0; return count; }
    void consume() && {}
private:
    std::vector<int> values_;
};
