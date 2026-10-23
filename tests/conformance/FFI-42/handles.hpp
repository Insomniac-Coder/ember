// The C++ side of accept_a_smart_pointer_like_class_is_read_through_get.em: classes read through
// `operator*` and `operator->`, `const` and not (ODR-252).
#pragma once

struct Point {
    int x;
    int y;
};

class Handle {
public:
    Handle(int x, int y) noexcept : point_{x, y} {}
    const Point& operator*() const noexcept { return point_; }
    const Point* operator->() const noexcept { return &point_; }

private:
    Point point_;
};

class MutHandle {
public:
    MutHandle(int x, int y) noexcept : point_{x, y} {}
    Point& operator*() noexcept { return point_; }

private:
    Point point_;
};
