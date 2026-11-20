// The C++ side of the M12 FFI-39d test: C++ keeps the widget it is given and sends it events.
#pragma once

class Widget {
public:
    Widget() noexcept {}
    virtual ~Widget() {}
    virtual void on_event(int code) noexcept { (void)code; }
};

inline Widget* held_widget = nullptr;
inline void hold(Widget* widget) noexcept { held_widget = widget; }
inline void dispatch_all() noexcept {
    if (held_widget != nullptr) {
        held_widget->on_event(1);
    }
}
