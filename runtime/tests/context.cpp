#include "meowy/cleanup.hpp"
#include "meowy/context.hpp"

#include <array>
#include <cfenv>
#include <cstdio>
#include <cstdlib>
#include <cstdint>
#include <source_location>
#include <string_view>
#include <sys/resource.h>
#include <type_traits>
#include <unistd.h>

using namespace meowy::prototype::v0;

extern "C" int meowy_test_registers_v0(void *, int (*)(void *));

namespace {

constexpr std::size_t stack_bytes = 262144;

void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

void empty(Context &, void *data) noexcept {
    if (data != nullptr) {
        ++*static_cast<int *>(data);
    }
}

void invalid_lifecycle_does_not_run_body() {
    static_assert(!std::is_copy_constructible_v<Context>);
    static_assert(!std::is_move_constructible_v<Context>);
    Context context;
    int calls = 0;
    check(context.resume() == ContextStatus::invalid);
    check(context.yield() == ContextStatus::invalid);
    check(context.release().status == ContextStatus::ok);
    check(context.initialize(0, empty, &calls).status == ContextStatus::invalid);
    check(context.initialize(Context::minimum_bytes - 1, empty, &calls).status == ContextStatus::invalid);
    check(context.initialize(stack_bytes, nullptr, nullptr).status == ContextStatus::invalid);
    check(context.initialize(stack_bytes, empty, &calls).status == ContextStatus::ok);
    check(context.initialize(stack_bytes, empty, &calls).status == ContextStatus::invalid);
    check(context.state() == ContextState::ready && calls == 0);
    check(context.release().status == ContextStatus::ok);
    check(context.state() == ContextState::empty && calls == 0 && context.bottom() == nullptr);
}

void completed_context_can_be_reinitialized() {
    Context context;
    int calls = 0;
    for (int index = 0; index < 8; ++index) {
        check(context.initialize(stack_bytes, empty, &calls).status == ContextStatus::ok);
        check(context.resume() == ContextStatus::ok);
        check(context.state() == ContextState::completed && calls == index + 1);
        check(context.resume() == ContextStatus::invalid);
        check(context.release().status == ContextStatus::ok);
    }
}

struct Progress final {
public:
    int count = 0;
    bool on_stack = false;

    [[gnu::noinline]] static void nested(Context &context, int depth, Progress &progress) noexcept {
        std::array<int, 32> values{};
        for (std::size_t index = 0; index < values.size(); ++index) {
            values[index] = depth * 100 + static_cast<int>(index);
        }
        const auto *const address = values.data();
        if (depth != 0) {
            nested(context, depth - 1, progress);
        } else {
            std::uintptr_t stack = 0;
            asm volatile("movq %%rsp, %0" : "=r"(stack));
            const auto begin = reinterpret_cast<std::uintptr_t>(context.bottom());
            progress.on_stack = stack >= begin && stack < begin + context.size().usable;
            for (int step = 0; step < 64; ++step) {
                ++progress.count;
                check(context.yield() == ContextStatus::ok);
                check(values.data() == address);
            }
        }
        for (std::size_t index = 0; index < values.size(); ++index) {
            check(values[index] == depth * 100 + static_cast<int>(index));
        }
    }

    static void run(Context &context, void *data) noexcept {
        nested(context, 12, *static_cast<Progress *>(data));
    }
};

void nested_stack_locals_survive_repeated_yields() {
    Context context;
    Progress progress;
    check(context.initialize(stack_bytes, Progress::run, &progress).status == ContextStatus::ok);
    for (int step = 0; step < 64; ++step) {
        check(context.resume() == ContextStatus::ok);
        check(context.state() == ContextState::suspended && progress.count == step + 1);
        check(context.release().status == ContextStatus::invalid);
    }
    check(context.resume() == ContextStatus::ok);
    check(context.state() == ContextState::completed && progress.on_stack);
    check(context.release().status == ContextStatus::ok);
}

int suspend(void *data) {
    return static_cast<Context *>(data)->yield() == ContextStatus::ok ? 0 : 1;
}

void registers(Context &context, void *data) noexcept {
    *static_cast<bool *>(data) = meowy_test_registers_v0(&context, suspend) == 0;
}

void callee_saved_registers_survive_switches() {
    Context context;
    bool preserved = false;
    check(context.initialize(stack_bytes, registers, &preserved).status == ContextStatus::ok);
    check(context.resume() == ContextStatus::ok && !preserved);
    check(context.resume() == ContextStatus::ok && preserved);
    check(context.release().status == ContextStatus::ok);
}

void rounding(Context &context, void *) noexcept {
    check(std::fegetround() == FE_UPWARD);
    check(std::fesetround(FE_DOWNWARD) == 0);
    check(context.yield() == ContextStatus::ok);
    check(std::fegetround() == FE_DOWNWARD);
    check(std::fesetround(FE_TONEAREST) == 0);
}

void floating_control_is_preserved() {
    const int previous = std::fegetround();
    check(std::fesetround(FE_UPWARD) == 0);
    Context context;
    check(context.initialize(stack_bytes, rounding, nullptr).status == ContextStatus::ok);
    check(context.resume() == ContextStatus::ok);
    check(std::fegetround() == FE_UPWARD);
    check(std::fesetround(FE_TOWARDZERO) == 0);
    check(context.resume() == ContextStatus::ok);
    check(std::fegetround() == FE_TOWARDZERO);
    check(context.release().status == ContextStatus::ok);
    check(std::fesetround(previous) == 0);
}

void once(Context &context, void *data) noexcept {
    auto &calls = *static_cast<int *>(data);
    ++calls;
    check(context.yield() == ContextStatus::ok);
    ++calls;
}

struct ThreadCall final {
public:
    Context &context;
    ContextStatus result = ContextStatus::invalid;

    static void *resume(void *data) noexcept {
        auto &call = *static_cast<ThreadCall *>(data);
        call.result = call.context.resume();
        check(call.context.yield() == ContextStatus::wrong_thread);
        return nullptr;
    }

    static void *finish(void *data) noexcept {
        auto &call = *static_cast<ThreadCall *>(data);
        call.result = call.context.resume();
        check(call.context.resume() == ContextStatus::ok);
        check(call.context.release().status == ContextStatus::ok);
        return nullptr;
    }
};

void resumed_context_stays_on_first_worker() {
    Context context;
    int calls = 0;
    check(context.initialize(stack_bytes, once, &calls).status == ContextStatus::ok);
    check(context.resume() == ContextStatus::ok);
    ThreadCall call{context};
    pthread_t thread{};
    check(pthread_create(&thread, nullptr, ThreadCall::resume, &call) == 0);
    check(pthread_join(thread, nullptr) == 0);
    check(call.result == ContextStatus::wrong_thread && calls == 1);
    check(context.resume() == ContextStatus::ok && calls == 2);
    check(context.release().status == ContextStatus::ok);
}

void first_execution_selects_worker() {
    Context context;
    int calls = 0;
    check(context.initialize(stack_bytes, once, &calls).status == ContextStatus::ok);
    ThreadCall call{context};
    pthread_t thread{};
    check(pthread_create(&thread, nullptr, ThreadCall::finish, &call) == 0);
    check(pthread_join(thread, nullptr) == 0);
    check(call.result == ContextStatus::ok && calls == 2 && context.state() == ContextState::empty);
}

void host_can_alternate_contexts() {
    Context first;
    Context second;
    int first_calls = 0;
    int second_calls = 0;
    check(first.initialize(stack_bytes, once, &first_calls).status == ContextStatus::ok);
    check(second.initialize(stack_bytes, once, &second_calls).status == ContextStatus::ok);
    check(first.resume() == ContextStatus::ok && first_calls == 1);
    check(second.resume() == ContextStatus::ok && second_calls == 1);
    check(first.resume() == ContextStatus::ok && first_calls == 2);
    check(second.resume() == ContextStatus::ok && second_calls == 2);
    check(first.release().status == ContextStatus::ok);
    check(second.release().status == ContextStatus::ok);
}

void invalid_operations(Context &context, void *data) noexcept {
    auto &other = *static_cast<Context *>(data);
    check(context.resume() == ContextStatus::invalid);
    check(context.release().status == ContextStatus::invalid);
    check(other.resume() == ContextStatus::invalid);
    check(other.yield() == ContextStatus::invalid);
    check(context.yield() == ContextStatus::ok);
}

void active_context_rejects_nested_resume_and_release() {
    Context first;
    Context second;
    check(second.initialize(stack_bytes, empty, nullptr).status == ContextStatus::ok);
    check(first.initialize(stack_bytes, invalid_operations, &second).status == ContextStatus::ok);
    check(first.resume() == ContextStatus::ok);
    check(first.resume() == ContextStatus::ok);
    check(first.release().status == ContextStatus::ok);
    check(second.release().status == ContextStatus::ok);
}

struct Borrowed final {
public:
    int &parent;
    bool cancel = false;
    bool cleaned = false;
    int cleanup_phase = 0;
    Context *context = nullptr;

    static Panic release(void *data) noexcept {
        auto &value = *static_cast<Borrowed *>(data);
        check(value.parent == 42);
        value.cleanup_phase = 1;
        check(value.context->yield() == ContextStatus::ok);
        check(value.parent == 42);
        value.cleanup_phase = 2;
        value.cleaned = true;
        return {};
    }

    static void run(Context &context, void *data) noexcept {
        auto &value = *static_cast<Borrowed *>(data);
        value.context = &context;
        std::array<Entry, 1> entries;
        Stack cleanup(entries);
        const auto root = cleanup.mark();
        const auto slot = cleanup.reserve();
        check(slot.status == Status::ok);
        check(cleanup.arm(slot.token, &value, release, "borrowed resource") == Status::ok);
        ++value.parent;
        check(context.yield() == ContextStatus::ok);
        check(value.cancel);
        check(cleanup.unwind(root, Reason::cancel).status == Status::ok);
    }
};

void explicit_cancel_cleanup_can_suspend() {
    Context context;
    int parent = 41;
    Borrowed value{parent};
    check(context.initialize(stack_bytes, Borrowed::run, &value).status == ContextStatus::ok);
    check(context.resume() == ContextStatus::ok && parent == 42 && !value.cleaned);
    value.cancel = true;
    check(context.resume() == ContextStatus::ok && value.cleanup_phase == 1 && !value.cleaned);
    check(context.release().status == ContextStatus::invalid);
    check(context.resume() == ContextStatus::ok && value.cleanup_phase == 2 && value.cleaned);
    check(context.state() == ContextState::completed && parent == 42);
    check(context.release().status == ContextStatus::ok);
}

Panic fail_release(void *) noexcept {
    constexpr std::string_view text = "release-trigger\n";
    check(::write(STDERR_FILENO, text.data(), text.size()) == static_cast<ssize_t>(text.size()));
    return {6, "release failed"};
}

void fatal_cleanup(Context &context, void *) noexcept {
    check(context.yield() == ContextStatus::ok);
    std::array<Entry, 1> entries;
    Stack cleanup(entries);
    const auto root = cleanup.mark();
    const auto slot = cleanup.reserve();
    check(cleanup.arm(slot.token, nullptr, fail_release, "newer") == Status::ok);
    static_cast<void>(cleanup.unwind(root, Reason::panic, {6, "body failed"}));
}

[[gnu::noinline]] void expired(Context &context, volatile int *&pointer) noexcept {
    volatile int value = 42;
    pointer = &value;
    check(context.yield() == ContextStatus::ok);
}

void lifetime(Context &context, void *data) noexcept {
    auto &pointer = *static_cast<volatile int **>(data);
    expired(context, pointer);
    check(context.yield() == ContextStatus::ok);
}

void sanitizer_lifetime_probe() {
    Context context;
    volatile int *pointer = nullptr;
    check(context.initialize(stack_bytes, lifetime, &pointer).status == ContextStatus::ok);
    check(context.resume() == ContextStatus::ok && *pointer == 42);
    check(context.resume() == ContextStatus::ok);
    constexpr std::string_view evidence = "asan-context: read returned fiber local\n";
    check(::write(STDERR_FILENO, evidence.data(), evidence.size()) == static_cast<ssize_t>(evidence.size()));
    const int invalid = *pointer;
    std::_Exit(invalid == 42 ? 3 : 4);
}

struct Case final {
public:
    std::string_view name;
    void (*run)();
};

constexpr std::array cases{
    Case{"invalid_lifecycle_does_not_run_body", invalid_lifecycle_does_not_run_body},
    Case{"completed_context_can_be_reinitialized", completed_context_can_be_reinitialized},
    Case{"nested_stack_locals_survive_repeated_yields", nested_stack_locals_survive_repeated_yields},
    Case{"callee_saved_registers_survive_switches", callee_saved_registers_survive_switches},
    Case{"floating_control_is_preserved", floating_control_is_preserved},
    Case{"resumed_context_stays_on_first_worker", resumed_context_stays_on_first_worker},
    Case{"first_execution_selects_worker", first_execution_selects_worker},
    Case{"host_can_alternate_contexts", host_can_alternate_contexts},
    Case{"active_context_rejects_nested_resume_and_release", active_context_rejects_nested_resume_and_release},
    Case{"explicit_cancel_cleanup_can_suspend", explicit_cancel_cleanup_can_suspend},
};

}

int main(int argc, char **argv) {
    if (argc == 2) {
        const rlimit limit{0, 0};
        check(setrlimit(RLIMIT_CORE, &limit) == 0);
        if (std::string_view(argv[1]) == "--fatal-context-cleanup") {
            Context context;
            check(context.initialize(stack_bytes, fatal_cleanup, nullptr).status == ContextStatus::ok);
            check(context.resume() == ContextStatus::ok);
            check(context.resume() == ContextStatus::ok);
            return 3;
        }
        if (std::string_view(argv[1]) == "--asan-use-after-return") {
            sanitizer_lifetime_probe();
        }
    }
    if (argc != 1) {
        return 2;
    }
    for (const auto &test : cases) {
        test.run();
        std::printf("PASS %.*s\n", static_cast<int>(test.name.size()), test.name.data());
    }
    std::printf("%zu context cases passed\n", cases.size());
    return 0;
}
