// The C++ side of the FFI-48 tests of a coroutine, a cast across a hierarchy and overloads that
// differ only in their return type.
#pragma once

struct Task {
    struct promise_type {};
    void* frame = nullptr;
};
Task fetch(int id);

class Shape {
public:
    virtual ~Shape() {}
    virtual int sides() const noexcept { return 0; }
};
class Square : public Shape {
public:
    int sides() const noexcept override { return 4; }
};
inline int sides_of(const Shape& shape) noexcept { return shape.sides(); }

class Holder {
public:
    Holder() noexcept {}
    int& value() noexcept { return value_; }
    const int& value() const noexcept { return value_; }
    int plain() const noexcept { return value_; }

private:
    int value_ = 7;
};

inline int ready() noexcept { return 1; }
