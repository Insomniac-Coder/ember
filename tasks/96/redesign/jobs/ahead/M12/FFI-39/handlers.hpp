// The C++ side of the M12 FFI-39 test: a base with two virtual methods of one name.
#pragma once

namespace ui {
struct Event {
    int code;
};

struct KeyEvent : Event {
    int key;
};

class Handler {
public:
    Handler() noexcept {}
    virtual ~Handler() {}
    virtual void on(const Event& e) noexcept { (void)e; }
    virtual void on(int timer_id) noexcept { (void)timer_id; }
};
}  // namespace ui
