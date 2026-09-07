#include "meowy/generated.hpp"
#include "meowy/cleanup.hpp"

#include <array>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <limits>
#include <source_location>
#include <string_view>
#include <sys/resource.h>
#include <unistd.h>

namespace {
void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

struct Storage final {
public:
    alignas(16) std::array<std::byte, 8192> bytes{};
    void *data() noexcept { return bytes.data(); }
    void open(std::uint64_t capacity) noexcept {
        check(meowy_cleanup_alignment_v0() <= 16);
        check(meowy_cleanup_bytes_v0(capacity) <= bytes.size());
        check(meowy_cleanup_open_v0(data(), bytes.size(), capacity) == 0);
    }
};

struct Log final {
public:
    std::array<int, 8> values{};
    std::size_t size = 0;
};
struct Item final {
public:
    Log *log;
    int id;
};
void drop(void *data, void *) noexcept {
    auto &item = *static_cast<Item *>(data);
    item.log->values[item.log->size++] = item.id;
}
MeowyCleanupToken arm(Storage &storage, Item &item) {
    MeowyCleanupToken token{};
    check(meowy_cleanup_reserve_v0(storage.data(), &token) == 0);
    check(meowy_cleanup_arm_v0(storage.data(), &token, &item, drop, "item", 4) == 0);
    return token;
}
MeowyCleanupMark mark(Storage &storage) {
    MeowyCleanupMark value{};
    check(meowy_cleanup_mark_v0(storage.data(), &value) == 0);
    return value;
}
void order() {
    Storage storage;
    storage.open(4);
    const auto root = mark(storage);
    Log log;
    Item a{&log, 1}, b{&log, 2}, c{&log, 3};
    const auto first = arm(storage, a);
    const auto second = arm(storage, b);
    check(meowy_cleanup_disarm_v0(storage.data(), &second) == 0);
    MeowyCleanupToken empty{};
    check(meowy_cleanup_reserve_v0(storage.data(), &empty) == 0);
    const auto third = arm(storage, c);
    check(meowy_cleanup_finish_v0(storage.data()) == 2);
    check(meowy_cleanup_unwind_v0(storage.data(), &root, 0, nullptr) == 0);
    check(log.size == 2 && log.values[0] == 3 && log.values[1] == 1);
    check(meowy_cleanup_disarm_v0(storage.data(), &first) == 2);
    check(meowy_cleanup_disarm_v0(storage.data(), &third) == 2);
    check(meowy_cleanup_finish_v0(storage.data()) == 0);
}
void reasons() {
    for (std::uint32_t reason = 0; reason <= 4; ++reason) {
        Storage storage, panic;
        storage.open(1);
        const auto root = mark(storage);
        Log log;
        Item item{&log, 1};
        arm(storage, item);
        check(meowy_cleanup_panic_init_v0(panic.data(), panic.bytes.size(), 6, "first", 5) == 0);
        check(meowy_cleanup_unwind_v0(storage.data(), &root, 3, nullptr) == 2);
        check(meowy_cleanup_unwind_v0(storage.data(), &root, 5, nullptr) == 2);
        check(meowy_cleanup_unwind_v0(storage.data(), &root, 0, panic.data()) == 2);
        check(log.size == 0);
        check(meowy_cleanup_unwind_v0(storage.data(), &root, reason, reason == 3 ? panic.data() : nullptr) == 0);
        check(log.size == 1);
        check(meowy_cleanup_finish_v0(storage.data()) == 0);
    }
}
void tokens() {
    Storage a, b;
    a.open(1); b.open(1);
    const auto ma = mark(a), mb = mark(b);
    Log log;
    Item item{&log, 1};
    auto token = arm(a, item);
    const auto old = token;
    check(meowy_cleanup_reserve_v0(a.data(), &token) == 1);
    check(token.owner == old.owner && token.index == old.index && token.id == old.id);
    check(meowy_cleanup_arm_v0(a.data(), &token, &item, drop, "other", 5) == 2);
    check(meowy_cleanup_disarm_v0(b.data(), &token) == 2);
    check(meowy_cleanup_unwind_v0(b.data(), &ma, 0, nullptr) == 2);
    check(meowy_cleanup_unwind_v0(a.data(), &ma, 0, nullptr) == 0);
    const auto next = arm(a, item);
    check(next.id != old.id);
    check(meowy_cleanup_disarm_v0(a.data(), &old) == 2);
    check(meowy_cleanup_unwind_v0(a.data(), &ma, 0, nullptr) == 0);
    check(meowy_cleanup_unwind_v0(b.data(), &mb, 0, nullptr) == 0);
    check(meowy_cleanup_finish_v0(a.data()) == 0 && meowy_cleanup_finish_v0(b.data()) == 0);
}
void nested() {
    Storage storage;
    storage.open(2);
    const auto root = mark(storage);
    Log log;
    Item a{&log, 1}, b{&log, 2};
    arm(storage, a);
    const auto inner = mark(storage);
    arm(storage, b);
    check(meowy_cleanup_unwind_v0(storage.data(), &inner, 1, nullptr) == 0);
    check(log.size == 1 && log.values[0] == 2);
    check(meowy_cleanup_unwind_v0(storage.data(), &root, 2, nullptr) == 0);
    arm(storage, a);
    check(meowy_cleanup_unwind_v0(storage.data(), &inner, 0, nullptr) == 2);
    check(meowy_cleanup_unwind_v0(storage.data(), &root, 0, nullptr) == 0);
    check(meowy_cleanup_finish_v0(storage.data()) == 0);
}
void reenter(void *data, void *) noexcept {
    MeowyCleanupMark mark{};
    MeowyCleanupToken token{};
    check(meowy_cleanup_finish_v0(data) == 2);
    check(meowy_cleanup_mark_v0(data, &mark) == 2);
    check(meowy_cleanup_reserve_v0(data, &token) == 2);
    check(meowy_cleanup_unwind_v0(data, &mark, 0, nullptr) == 2);
}
void invalid() {
    Storage storage;
    check(meowy_cleanup_bytes_v0(std::numeric_limits<std::uint64_t>::max()) == 0);
    check(meowy_cleanup_open_v0(nullptr, 8192, 1) == 2);
    check(meowy_cleanup_open_v0(storage.bytes.data() + 1, 8191, 1) == 2);
    check(meowy_cleanup_open_v0(storage.data(), 1, 1) == 2);
    storage.open(1);
    const auto root = mark(storage);
    MeowyCleanupToken token{};
    check(meowy_cleanup_reserve_v0(storage.data(), nullptr) == 2);
    check(meowy_cleanup_reserve_v0(storage.data(), &token) == 0);
    check(meowy_cleanup_arm_v0(storage.data(), &token, nullptr, nullptr, "x", 1) == 2);
    check(meowy_cleanup_arm_v0(storage.data(), &token, storage.data(), reenter, "reentry", 7) == 0);
    check(meowy_cleanup_unwind_v0(storage.data(), &root, 0, nullptr) == 0);
    check(meowy_cleanup_finish_v0(storage.data()) == 0);
    storage.open(0);
    check(meowy_cleanup_reserve_v0(storage.data(), &token) == 1);
    check(meowy_cleanup_finish_v0(storage.data()) == 0);
}
void snapshots() {
    Storage storage;
    std::array<char, 300> text;
    text.fill('a');
    check(meowy_cleanup_panic_init_v0(storage.data(), 1, 6, text.data(), text.size()) == 2);
    check(meowy_cleanup_panic_init_v0(storage.data(), storage.bytes.size(), 6, text.data(), text.size()) == 0);
    text.fill('b');
    auto &panic = *static_cast<meowy::prototype::v0::Panic *>(storage.data());
    check(panic.code == 6 && panic.truncated() && panic.original_size() == 300);
    check(panic.message().size() == 256 && panic.message()[0] == 'a');
    const auto copy = panic;
    check(meowy_cleanup_panic_init_v0(storage.data(), storage.bytes.size(), 2, panic.message().data(), 10) == 0);
    check(panic.code == 2 && panic.message() == "aaaaaaaaaa");
    check(copy.code == 6 && copy.original_size() == 300);
}
void fail(void *, void *panic) noexcept {
    char text[] = "release failed";
    check(meowy_cleanup_panic_init_v0(panic, meowy_cleanup_panic_bytes_v0(), 6, text, sizeof(text) - 1) == 0);
    std::memset(text, '?', sizeof(text));
    constexpr char line[] = "release-trigger\n";
    check(::write(STDERR_FILENO, line, sizeof(line) - 1) == sizeof(line) - 1);
}
void fatal(bool panicked) {
    const rlimit limit{0, 0};
    check(setrlimit(RLIMIT_CORE, &limit) == 0);
    Storage storage, panic;
    storage.open(1);
    const auto root = mark(storage);
    MeowyCleanupToken token{};
    check(meowy_cleanup_reserve_v0(storage.data(), &token) == 0);
    check(meowy_cleanup_arm_v0(storage.data(), &token, nullptr, fail, "newer", 5) == 0);
    check(meowy_cleanup_panic_init_v0(panic.data(), panic.bytes.size(), 6, "body failed", 11) == 0);
    check(meowy_cleanup_unwind_v0(storage.data(), &root, panicked ? 3 : 0, panicked ? panic.data() : nullptr) == 0);
    std::abort();
}
}
int main(int argc, char **argv) {
    if (argc == 2) { fatal(std::string_view(argv[1]) == "--fatal-panic"); }
    for (const auto &[name, test] : std::array<std::pair<const char *, void (*)()>, 6>{{
             {"order", order}, {"reasons", reasons}, {"tokens", tokens},
             {"nested", nested}, {"invalid", invalid}, {"snapshots", snapshots}}}) {
        test();
        std::printf("PASS %s\n", name);
    }
    std::puts("6 generated cleanup cases passed");
}
