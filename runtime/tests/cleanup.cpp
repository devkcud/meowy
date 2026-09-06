#include "meowy/cleanup.hpp"

#include <array>
#include <cstdio>
#include <cstdlib>
#include <source_location>
#include <string_view>
#include <sys/resource.h>
#include <unistd.h>

using namespace meowy::prototype::v0;

namespace {

void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

struct Log final {
public:
    std::array<int, 32> values{};
    std::size_t size = 0;

    void add(int id) noexcept {
        check(size < values.size());
        values[size++] = id;
    }

    void expect(std::initializer_list<int> expected) const {
        check(size == expected.size());
        std::size_t index = 0;
        for (const int id : expected) {
            check(values[index++] == id);
        }
    }
};

struct Resource final {
public:
    Log &log;
    int id;
    bool alive = false;

    static Panic release(void *data) noexcept {
        auto &value = *static_cast<Resource *>(data);
        check(value.alive);
        value.alive = false;
        value.log.add(value.id);
        return {};
    }
};

Token init(Stack &stack, Resource &value) {
    const auto slot = stack.reserve();
    check(slot.status == Status::ok);
    check(!value.alive);
    value.alive = true;
    check(stack.arm(slot.token, &value, Resource::release, "resource") == Status::ok);
    return slot.token;
}

void reverse_initialization() {
    std::array<Entry, 3> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource first{log, 1};
    Resource second{log, 2};
    Resource third{log, 3};
    init(stack, first);
    init(stack, second);
    init(stack, third);
    const auto result = stack.unwind(root, Reason::complete);
    check(result.status == Status::ok && result.reason == Reason::complete);
    check(result.panic.code == 0 && stack.size() == 0);
    log.expect({3, 2, 1});
    check(!first.alive && !second.alive && !third.alive);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({3, 2, 1});
}

void partial_initialization() {
    std::array<Entry, 3> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource field{log, 1};
    init(stack, field);
    const auto failed = stack.reserve();
    check(failed.status == Status::ok);
    check(stack.disarm(failed.token) == Status::invalid);
    const auto result = stack.unwind(root, Reason::panic, {6, "constructor failed"});
    check(result.status == Status::ok && result.panic.message() == "constructor failed");
    log.expect({1});
}

void leave_preserves_result() {
    std::array<Entry, 4> entries;
    std::array<Entry, 1> result_entries;
    Stack stack(entries);
    Stack result_stack(result_entries);
    const auto root = stack.mark();
    const auto result_root = result_stack.mark();
    Log log;
    Resource outer{log, 1};
    Resource inner{log, 2};
    Resource emitted{log, 3};
    init(stack, outer);
    const auto scope = stack.mark();
    init(stack, inner);
    const auto result = init(stack, emitted);
    check(stack.transfer(result, result_stack).status == Status::ok);
    check(stack.unwind(scope, Reason::leave).status == Status::ok);
    log.expect({2});
    check(outer.alive && emitted.alive && !inner.alive);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    check(result_stack.unwind(result_root, Reason::complete).status == Status::ok);
    log.expect({2, 1, 3});
}

void restart_discards_iteration() {
    std::array<Entry, 4> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource outer{log, 1};
    Resource local{log, 2};
    Resource emitted{log, 3};
    init(stack, outer);
    const auto scope = stack.mark();
    init(stack, local);
    init(stack, emitted);
    check(stack.unwind(scope, Reason::restart).status == Status::ok);
    check(outer.alive && !local.alive && !emitted.alive);
    log.expect({3, 2});
    init(stack, local);
    check(stack.unwind(scope, Reason::leave).status == Status::ok);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({3, 2, 2, 1});
}

void disarm_after_close() {
    std::array<Entry, 1> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource value{log, 1};
    const auto token = init(stack, value);
    check(Resource::release(&value).code == 0);
    check(stack.disarm(token) == Status::ok);
    check(stack.disarm(token) == Status::invalid);
    check(stack.arm(token, &value, Resource::release, "again") == Status::invalid);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({1});
}

void transfer_failure_retains_owner() {
    std::array<Entry, 1> entries;
    std::array<Entry, 0> empty;
    Stack stack(entries);
    Stack full(empty);
    const auto root = stack.mark();
    Log log;
    Resource value{log, 1};
    const auto token = init(stack, value);
    check(stack.transfer(token, full).status == Status::full);
    check(stack.transfer(token, stack).status == Status::invalid);
    check(value.alive);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({1});
}

void panic_after_partial_transfer() {
    std::array<Entry, 2> entries;
    std::array<Entry, 2> result_entries;
    Stack stack(entries);
    Stack result_stack(result_entries);
    const auto root = stack.mark();
    const auto result_root = result_stack.mark();
    Log log;
    Resource existing{log, 1};
    Resource moved{log, 2};
    Resource remaining{log, 3};
    init(result_stack, existing);
    const auto token = init(stack, moved);
    init(stack, remaining);
    check(stack.transfer(token, result_stack).status == Status::ok);
    check(stack.transfer(token, result_stack).status == Status::invalid);
    check(stack.unwind(root, Reason::panic, {6, "after transfer"}).status == Status::ok);
    log.expect({3});
    check(existing.alive && moved.alive && !remaining.alive);
    check(result_stack.unwind(result_root, Reason::complete).status == Status::ok);
    log.expect({3, 2, 1});
}

void capacity_precedes_construction() {
    std::array<Entry, 1> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource value{log, 1};
    init(stack, value);
    check(stack.reserve().status == Status::full);
    check(stack.size() == 1 && value.alive);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({1});
}

void stale_tokens_and_marks() {
    std::array<Entry, 1> entries;
    std::array<Entry, 1> other_entries;
    Stack stack(entries);
    Stack other(other_entries);
    const auto root = stack.mark();
    Log log;
    Resource value{log, 1};
    const auto old = init(stack, value);
    const auto old_mark = stack.mark();
    check(other.disarm(old) == Status::invalid);
    check(other.unwind(old_mark, Reason::complete).status == Status::invalid);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    const auto current = init(stack, value);
    check(old.id != current.id);
    check(stack.disarm(old) == Status::invalid);
    check(stack.unwind(old_mark, Reason::complete).status == Status::invalid);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({1, 1});
}

void initialization_order_is_checked() {
    std::array<Entry, 2> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource value{log, 1};
    const auto first = stack.reserve();
    const auto second = stack.reserve();
    check(first.status == Status::ok && second.status == Status::ok);
    check(stack.arm(first.token, &value, Resource::release, "out of order") == Status::invalid);
    check(stack.arm(second.token, &value, nullptr, "missing callback") == Status::invalid);
    check(stack.arm(second.token, &value, Resource::release, {}) == Status::invalid);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({});
}

void recoverable_panic_preserves_cause() {
    std::array<Entry, 3> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource local{log, 1};
    Resource emitted{log, 2};
    init(stack, local);
    init(stack, emitted);
    const Panic panic{2, "body overflow"};
    const auto result = stack.unwind(root, Reason::panic, panic);
    check(result.status == Status::ok && result.reason == Reason::panic);
    check(result.panic.code == 2 && result.panic.message() == "body overflow");
    log.expect({2, 1});
}

void cancel_cleans_explicit_boundary() {
    std::array<Entry, 1> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource value{log, 1};
    init(stack, value);
    const auto result = stack.unwind(root, Reason::cancel);
    check(result.status == Status::ok && result.reason == Reason::cancel);
    log.expect({1});
}

void invalid_panic_protocol_retains_owner() {
    std::array<Entry, 1> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Log log;
    Resource value{log, 1};
    init(stack, value);
    check(stack.unwind(root, Reason::panic).status == Status::invalid);
    check(stack.unwind(root, Reason::complete, {6, "wrong edge"}).status == Status::invalid);
    check(stack.unwind(root, static_cast<Reason>(99)).status == Status::invalid);
    check(value.alive && stack.size() == 1);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    log.expect({1});
}

struct Reentrant final {
public:
    Stack &stack;
    Mark mark;
    Token token;
    bool called = false;

    static Panic release(void *data) noexcept {
        auto &value = *static_cast<Reentrant *>(data);
        check(value.stack.reserve().status == Status::invalid);
        check(value.stack.disarm(value.token) == Status::invalid);
        check(value.stack.unwind(value.mark, Reason::restart).status == Status::invalid);
        value.called = true;
        return {};
    }
};

void cleanup_cannot_mutate_active_stack() {
    std::array<Entry, 1> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    const auto slot = stack.reserve();
    Reentrant value{stack, root, slot.token};
    check(stack.arm(slot.token, &value, Reentrant::release, "reentrant") == Status::ok);
    check(stack.unwind(root, Reason::complete).status == Status::ok);
    check(value.called);
}

struct Fatal final {
public:
    std::string_view name;
    bool fail;

    static Panic release(void *data) noexcept {
        const auto &value = *static_cast<Fatal *>(data);
        check(::write(STDERR_FILENO, value.name.data(), value.name.size()) ==
              static_cast<ssize_t>(value.name.size()));
        return value.fail ? Panic{6, "release failed"} : Panic{};
    }
};

void fatal_cleanup(bool panicked) {
    const rlimit limit{0, 0};
    check(setrlimit(RLIMIT_CORE, &limit) == 0);
    std::array<Entry, 2> entries;
    Stack stack(entries);
    const auto root = stack.mark();
    Fatal older{"must-not-release\n", false};
    Fatal newer{"release-trigger\n", true};
    const auto first = stack.reserve();
    check(stack.arm(first.token, &older, Fatal::release, "older") == Status::ok);
    const auto second = stack.reserve();
    check(stack.arm(second.token, &newer, Fatal::release, "newer") == Status::ok);
    const auto result = stack.unwind(root, panicked ? Reason::panic : Reason::complete,
                                     panicked ? Panic{6, "body failed"} : Panic{});
    static_cast<void>(result);
    std::exit(3);
}

struct Case final {
public:
    std::string_view name;
    void (*run)();
};

constexpr std::array cases{
    Case{"reverse_initialization", reverse_initialization},
    Case{"partial_initialization", partial_initialization},
    Case{"leave_preserves_result", leave_preserves_result},
    Case{"restart_discards_iteration", restart_discards_iteration},
    Case{"disarm_after_close", disarm_after_close},
    Case{"transfer_failure_retains_owner", transfer_failure_retains_owner},
    Case{"panic_after_partial_transfer", panic_after_partial_transfer},
    Case{"capacity_precedes_construction", capacity_precedes_construction},
    Case{"stale_tokens_and_marks", stale_tokens_and_marks},
    Case{"initialization_order_is_checked", initialization_order_is_checked},
    Case{"recoverable_panic_preserves_cause", recoverable_panic_preserves_cause},
    Case{"cancel_cleans_explicit_boundary", cancel_cleans_explicit_boundary},
    Case{"invalid_panic_protocol_retains_owner", invalid_panic_protocol_retains_owner},
    Case{"cleanup_cannot_mutate_active_stack", cleanup_cannot_mutate_active_stack},
};

}

int main(int argc, char **argv) {
    if (argc == 2 && std::string_view(argv[1]) == "--fatal-normal") {
        fatal_cleanup(false);
    }
    if (argc == 2 && std::string_view(argv[1]) == "--fatal-panic") {
        fatal_cleanup(true);
    }
    if (argc != 1) {
        return 2;
    }
    for (const auto &test : cases) {
        test.run();
        std::printf("PASS %.*s\n", static_cast<int>(test.name.size()), test.name.data());
    }
    std::printf("%zu cleanup cases passed\n", cases.size());
    return 0;
}
