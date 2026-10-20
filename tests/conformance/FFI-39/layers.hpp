// The C++ side of the FFI-39 tests: a class Ember classes derive from, and C++ that calls the
// virtuals of whatever it is given, as an engine calls its layers'.
#pragma once

class Layer {
public:
    Layer() noexcept : id_(1) {}
    explicit Layer(int id) noexcept : id_(id) {}
    explicit Layer(double scale) noexcept : id_(static_cast<int>(scale * 10)) {}
    virtual ~Layer() {}
    virtual void on_update(float dt) { total_ += static_cast<int>(dt); }
    virtual int name() const noexcept { return id_; }
    virtual int unnamed() noexcept { return 7; }
    int id() const noexcept { return id_; }
    int total() const noexcept { return total_; }

private:
    int id_;
    int total_ = 0;
};

inline int drive(Layer& layer) noexcept {
    layer.on_update(2.0f);
    layer.on_update(3.0f);
    return layer.name() * 100 + layer.total();
}
