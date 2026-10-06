#include <cstdint>
#include <cstdio>
#include <string>
#include <vector>

struct Player {
    std::vector<int64_t> a;
    std::vector<int64_t> b;
    std::vector<int64_t> c;
    std::vector<int64_t> d;
    std::vector<int64_t> e;
    std::vector<int64_t> f;
    int64_t hp;
    explicit Player(int64_t hp) : hp(hp) {}
};

int main() {
    std::vector<Player*> players;
    for (int64_t i = 0; i < 1000000; i++) players.push_back(new Player(i % 100));
    int64_t total = 0;
    for (int round = 0; round < 40; round++)
        for (Player* p : players) total += p->hp + (int64_t)p->f.size();
    std::printf("%lld\n", (long long)total);
    return 0;
}
