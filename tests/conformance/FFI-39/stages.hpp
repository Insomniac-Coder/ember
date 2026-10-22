// The C++ side of the FFI-39 tests of classes Ember derives from with no overlay entry (ODR-221):
// a class whose virtuals C++ calls, and an abstract one.
#pragma once

class Stage {
public:
    Stage() noexcept : id_(1) {}
    explicit Stage(int id) noexcept : id_(id) {}
    virtual ~Stage() {}
    virtual void tick(int dt) { total_ += dt; }
    virtual int label() const noexcept { return id_; }
    virtual int weight() const noexcept { return 1; }
    int id() const noexcept { return id_; }
    int total() const noexcept { return total_; }

private:
    int id_;
    int total_ = 0;
};

class Shape {
public:
    virtual ~Shape() {}
    virtual int area() const noexcept = 0;
    virtual int sides() const noexcept { return 0; }
};

inline int run(Stage& stage) noexcept {
    stage.tick(2);
    stage.tick(3);
    return stage.label() * 100 + stage.total() * 10 + stage.weight();
}

inline int measure(const Shape& shape) noexcept { return shape.area() * 10 + shape.sides(); }
