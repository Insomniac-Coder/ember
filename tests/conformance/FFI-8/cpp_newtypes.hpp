// The C++ side of accept_a_cpp_alias_is_a_newtype.em: a `using` alias and a `typedef` an overlay
// makes nominal (ODR-249).
#pragma once

using port_t = unsigned short;
typedef double ratio_t;

inline port_t next_port(port_t port) noexcept { return static_cast<port_t>(port + 1); }
inline ratio_t halve(ratio_t ratio) noexcept { return ratio / 2.0; }

class Socket {
public:
    explicit Socket(port_t port) noexcept : port_(port) {}
    ~Socket() {}
    port_t port() const noexcept { return port_; }

private:
    port_t port_;
};
