#include <cstdint>
#include <cstdio>
#include <string>
int main() {
    std::string s;
    for (int64_t i = 0; i < 8000000; i++) { s += "hello "; s += "world "; }
    int64_t count = 0;
    for (unsigned char b : s) if (b == 111) count++;
    std::printf("%lld %lld\n", (long long)s.size(), (long long)count);
    return 0;
}
