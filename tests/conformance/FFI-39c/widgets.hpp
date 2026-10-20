// The C++ side of the FFI-39c tests: a widget C++ keeps once it is added, and a function that only
// borrows one for the call.
#pragma once

class Widget {
public:
    Widget() noexcept {}
    virtual ~Widget() {}
    virtual int draw() const noexcept { return 0; }
};

inline Widget* added_widget = nullptr;
inline void add_widget(Widget* widget) noexcept { added_widget = widget; }
inline int draw_added() noexcept { return added_widget != nullptr ? added_widget->draw() : -1; }
inline void remove_widget() noexcept { added_widget = nullptr; }
inline int draw_once(const Widget& widget) noexcept { return widget.draw(); }
