#include "meowy/generated.hpp"

#include <array>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <source_location>
#include <string_view>
#include <sys/resource.h>
#include <unistd.h>

namespace {
void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) { std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line()); std::abort(); }
}
struct Buffer final {
public:
    alignas(16) std::array<std::byte, 4096> bytes{};
    void *data() noexcept { return bytes.data(); }
};
struct Counts final {
public:
    int moves = 0;
    int drops = 0;
};
struct Payload final {
public:
    Counts *counts;
    void *self;
    int value;
};
void move(void *to, void *from) noexcept {
    auto &source = *static_cast<Payload *>(from);
    check(source.self == from);
    ++source.counts->moves;
    std::construct_at(static_cast<Payload *>(to), Payload{source.counts, to, source.value});
    source.self = nullptr;
    std::destroy_at(&source);
}
void drop(void *data, void *) noexcept {
    auto &value = *static_cast<Payload *>(data);
    check(value.self == data);
    ++value.counts->drops;
    value.self = nullptr;
    std::destroy_at(&value);
}
void *ops(MeowyCleanupDrop capture = drop) {
    static Buffer normal, failing;
    static bool ready = false;
    if (!ready) {
        check(meowy_owned_ops_bytes_v0() <= normal.bytes.size());
        check(meowy_owned_ops_init_v0(normal.data(), normal.bytes.size(), sizeof(Payload), alignof(Payload), move, drop, "owned fixture", 13) == 0);
        ready = true;
    }
    if (capture == drop) { return normal.data(); }
    check(meowy_owned_ops_init_v0(failing.data(), failing.bytes.size(), sizeof(Payload), alignof(Payload), move, capture, "owned fixture", 13) == 0);
    return failing.data();
}
struct Owner final {
public:
    Buffer token;
    Buffer payload;
    Owner() {
        check(meowy_owned_bytes_v0() <= token.bytes.size());
        check(meowy_owned_open_v0(token.data(), token.bytes.size(), payload.data(), payload.bytes.size()) == 0);
    }
    void *data() noexcept { return token.data(); }
    void construct(Counts &counts, MeowyCleanupDrop capture = drop) {
        check(meowy_owned_reserve_v0(data(), ops(capture)) == 0);
        auto *ptr = meowy_owned_data_v0(data());
        std::construct_at(static_cast<Payload *>(ptr), Payload{&counts, ptr, 42});
        check(meowy_owned_commit_v0(data()) == 0);
    }
    void finish() { check(meowy_owned_finish_v0(data()) == 0); }
};
struct Frame final {
public:
    Buffer bytes;
    MeowyCleanupMark root{};
    explicit Frame(std::uint64_t capacity = 4) {
        check(meowy_cleanup_bytes_v0(capacity) <= bytes.bytes.size());
        check(meowy_cleanup_open_v0(data(), bytes.bytes.size(), capacity) == 0);
        check(meowy_cleanup_mark_v0(data(), &root) == 0);
    }
    void *data() noexcept { return bytes.data(); }
    MeowyCleanupToken reserve() {
        MeowyCleanupToken token{};
        check(meowy_cleanup_reserve_v0(data(), &token) == 0);
        return token;
    }
    MeowyCleanupToken arm(Owner &owner) {
        const auto token = reserve();
        check(meowy_owned_arm_v0(data(), &token, owner.data()) == 0);
        return token;
    }
    void close() {
        check(meowy_cleanup_unwind_v0(data(), &root, 0, nullptr) == 0);
        check(meowy_cleanup_finish_v0(data()) == 0);
    }
};
void relocate() {
    Counts counts;
    Owner source, target;
    Frame parent, child;
    source.construct(counts);
    const auto old = child.arm(source), next = parent.reserve();
    const auto before = meowy_owned_data_v0(source.data());
    check(meowy_owned_transfer_v0(child.data(), &old, source.data(), parent.data(), &next, target.data()) == 0);
    check(meowy_owned_data_v0(source.data()) == nullptr);
    const auto *value = static_cast<Payload *>(meowy_owned_data_v0(target.data()));
    check(value != before && value->self == value && value->value == 42);
    check(counts.moves == 1 && counts.drops == 0);
    child.close(); source.finish();
    check(counts.drops == 0);
    parent.close(); target.finish();
    check(counts.drops == 1);
}
void rejected_payloads() {
    for (const auto expected : {2U, 3U, 4U, 5U}) {
        Counts counts;
        Owner source, target;
        Frame a, b;
        source.construct(counts);
        const auto old = a.arm(source), next = b.reserve();
        if (expected == 2 || expected == 3 || expected == 5) {
            target.finish();
            void *ptr = expected == 5 ? source.payload.data() : target.payload.bytes.data() + (expected == 3 ? 1 : 0);
            check(meowy_owned_open_v0(target.data(), target.token.bytes.size(), ptr, expected == 2 ? 1 : sizeof(Payload)) == 0);
        } else {
            check(meowy_owned_reserve_v0(target.data(), ops()) == 0);
        }
        check(meowy_owned_transfer_v0(a.data(), &old, source.data(), b.data(), &next, target.data()) == expected);
        check(meowy_owned_data_v0(source.data()) != nullptr && counts.moves == 0 && counts.drops == 0);
        check(meowy_owned_release_v0(target.data()) == 0);
        target.finish();
        check(meowy_owned_open_v0(target.data(), target.token.bytes.size(), target.payload.data(), target.payload.bytes.size()) == 0);
        check(meowy_owned_transfer_v0(a.data(), &old, source.data(), b.data(), &next, target.data()) == 0);
        a.close(); b.close(); source.finish(); target.finish();
        check(counts.moves == 1 && counts.drops == 1);
    }
}
void rejected_tokens() {
    Counts counts;
    Owner source, target, wrong;
    Frame a, b, full(0);
    source.construct(counts);
    const auto old = a.arm(source), next = b.reserve();
    auto stale = old; ++stale.id;
    check(meowy_owned_transfer_v0(a.data(), &stale, source.data(), b.data(), &next, target.data()) == 1);
    check(meowy_owned_transfer_v0(a.data(), &old, wrong.data(), b.data(), &next, target.data()) == 1);
    check(meowy_owned_transfer_v0(a.data(), &old, source.data(), a.data(), &next, target.data()) == 1);
    auto missing = next;
    check(meowy_cleanup_reserve_v0(full.data(), &missing) == 1);
    check(meowy_owned_transfer_v0(a.data(), &old, source.data(), full.data(), &missing, target.data()) == 1);
    check(counts.moves == 0 && counts.drops == 0);
    check(meowy_owned_transfer_v0(a.data(), &old, source.data(), b.data(), &next, target.data()) == 0);
    check(meowy_owned_transfer_v0(a.data(), &old, source.data(), b.data(), &next, wrong.data()) == 1);
    a.close(); b.close(); full.close(); source.finish(); target.finish(); wrong.finish();
    check(counts.drops == 1);
}
void same_frame() {
    Counts counts;
    Owner source, target;
    Frame frame;
    source.construct(counts);
    const auto old = frame.arm(source), next = frame.reserve();
    check(meowy_owned_transfer_v0(frame.data(), &old, source.data(), frame.data(), &next, target.data()) == 0);
    source.finish();
    frame.close(); target.finish();
    check(counts.moves == 1 && counts.drops == 1);
}
void partial() {
    Owner owner;
    Frame frame;
    const auto token = frame.reserve();
    check(meowy_owned_commit_v0(owner.data()) == 1);
    check(meowy_owned_reserve_v0(owner.data(), ops()) == 0);
    check(meowy_owned_arm_v0(frame.data(), &token, owner.data()) == 1);
    check(meowy_owned_finish_v0(owner.data()) == 1);
    check(meowy_owned_release_v0(owner.data()) == 0);
    owner.finish(); frame.close();
}
void descriptors() {
    Buffer buffer;
    check(meowy_owned_ops_init_v0(buffer.data(), 1, sizeof(Payload), alignof(Payload), move, drop, "x", 1) == 1);
    check(meowy_owned_ops_init_v0(buffer.data(), buffer.bytes.size(), 0, 8, move, drop, "x", 1) == 1);
    check(meowy_owned_ops_init_v0(buffer.data(), buffer.bytes.size(), 8, 3, move, drop, "x", 1) == 1);
    check(meowy_owned_ops_init_v0(buffer.data(), buffer.bytes.size(), 8, 8, nullptr, drop, "x", 1) == 1);
    check(meowy_owned_ops_init_v0(buffer.data(), buffer.bytes.size(), 8, 8, move, nullptr, "x", 1) == 1);
    check(meowy_owned_open_v0(buffer.data(), 1, nullptr, 0) == 1);
    check(meowy_owned_open_v0(buffer.data(), buffer.bytes.size(), nullptr, 1) == 1);
}
Frame *active_a = nullptr;
Frame *active_b = nullptr;
Owner *active_source = nullptr;
Owner *active_target = nullptr;
void guarded_move(void *to, void *from) noexcept {
    MeowyCleanupToken token{};
    check(meowy_cleanup_reserve_v0(active_a->data(), &token) == 2);
    check(meowy_cleanup_reserve_v0(active_b->data(), &token) == 2);
    check(meowy_cleanup_finish_v0(active_a->data()) == 2);
    check(meowy_owned_data_v0(active_source->data()) == nullptr);
    check(meowy_owned_data_v0(active_target->data()) == nullptr);
    check(meowy_owned_release_v0(active_source->data()) == 1);
    check(meowy_owned_finish_v0(active_target->data()) == 1);
    move(to, from);
}
void reentry() {
    static Buffer descriptor;
    check(meowy_owned_ops_init_v0(descriptor.data(), descriptor.bytes.size(), sizeof(Payload), alignof(Payload), guarded_move, drop, "guarded", 7) == 0);
    Counts counts;
    Owner source, target;
    Frame a, b;
    active_a = &a; active_b = &b; active_source = &source; active_target = &target;
    check(meowy_owned_reserve_v0(source.data(), descriptor.data()) == 0);
    auto *ptr = meowy_owned_data_v0(source.data());
    std::construct_at(static_cast<Payload *>(ptr), Payload{&counts, ptr, 42});
    check(meowy_owned_commit_v0(source.data()) == 0);
    const auto old = a.arm(source), next = b.reserve();
    check(meowy_owned_transfer_v0(a.data(), &old, source.data(), b.data(), &next, target.data()) == 0);
    a.close(); b.close(); source.finish(); target.finish();
    check(counts.moves == 1 && counts.drops == 1);
}
void fail(void *data, void *panic) noexcept {
    char text[] = "release failed";
    check(meowy_cleanup_panic_init_v0(panic, meowy_cleanup_panic_bytes_v0(), 6, text, sizeof(text) - 1) == 0);
    std::memset(text, '?', sizeof(text));
    drop(data, nullptr);
    constexpr char line[] = "release-trigger\n";
    check(::write(STDERR_FILENO, line, sizeof(line) - 1) == sizeof(line) - 1);
}
void fatal(bool panicked) {
    const rlimit limit{0, 0}; check(setrlimit(RLIMIT_CORE, &limit) == 0);
    Counts counts;
    Owner source, target;
    Frame a, b;
    source.construct(counts, fail);
    const auto old = a.arm(source), next = b.reserve();
    check(meowy_owned_transfer_v0(a.data(), &old, source.data(), b.data(), &next, target.data()) == 0);
    a.close(); source.finish();
    Buffer panic;
    check(meowy_cleanup_panic_init_v0(panic.data(), panic.bytes.size(), 6, "body failed", 11) == 0);
    check(meowy_cleanup_unwind_v0(b.data(), &b.root, panicked ? 3 : 0, panicked ? panic.data() : nullptr) == 0);
    std::abort();
}
}
int main(int argc, char **argv) {
    if (argc == 2) { fatal(std::string_view(argv[1]) == "--fatal-panic"); }
    for (const auto &[name, test] : std::array<std::pair<const char *, void (*)()>, 7>{{
             {"relocate", relocate}, {"payload failures", rejected_payloads}, {"token failures", rejected_tokens},
             {"same frame", same_frame}, {"partial", partial}, {"descriptors", descriptors}, {"reentry", reentry}}}) {
        test(); std::printf("PASS %s\n", name);
    }
    std::puts("7 generated ownership cases passed");
}
