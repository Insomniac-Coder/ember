// The C++ side of the FFI-39d tests: a widget whose events C++ delivers back to it.
#pragma once

class Widget {
public:
    Widget() noexcept {}
    virtual ~Widget() {}
    virtual void on_event(int code) noexcept { (void)code; }
};

inline void fire(Widget& widget, int code) noexcept { widget.on_event(code); }
