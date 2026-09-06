#include "meowy/cleanup.hpp"
#include "meowy/stack_memory.hpp"

#include <array>
#include <atomic>
#include <cerrno>
#include <csignal>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <limits>
#include <source_location>
#include <string_view>
#include <sys/mman.h>
#include <sys/resource.h>
#include <type_traits>
#include <unistd.h>

using namespace meowy::prototype::v0;

namespace {

bool fail_map = false;
bool fail_protect = false;
bool fail_release = false;
int releases = 0;

void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

std::size_t page_size() {
    const auto size = sysconf(_SC_PAGESIZE);
    check(size > 0);
    return static_cast<std::size_t>(size);
}

void layout_accounts_for_guards() {
    const auto page = page_size();
    const auto plan = StackMemory::plan(page * 4, page);
    check(plan.result.status == MemoryStatus::ok);
    check(plan.size.usable == page * 4 && plan.size.guard == page);
    check(plan.size.total == page * 6);
}

void invalid_sizes_do_not_allocate() {
    const auto page = page_size();
    StackMemory memory;
    for (const auto size : {std::size_t(0), page - 1, page + 1}) {
        check(memory.allocate(size).status == MemoryStatus::invalid);
        check(!memory.owns() && memory.data() == nullptr && memory.size().total == 0);
    }
    check(StackMemory::plan(page, 0).result.status == MemoryStatus::invalid);
}

void overflow_is_checked() {
    const auto limit = std::numeric_limits<std::size_t>::max();
    const auto page = page_size();
    check(StackMemory::plan(limit - limit % page, page).result.status == MemoryStatus::overflow);
    check(StackMemory::plan(limit, limit).result.status == MemoryStatus::overflow);
    const auto max = static_cast<std::size_t>(std::numeric_limits<std::ptrdiff_t>::max());
    check(StackMemory::plan(max - max % page, page).result.status == MemoryStatus::overflow);
}

void payload_is_writable_and_zeroed() {
    const auto page = page_size();
    StackMemory memory;
    check(memory.allocate(page * 3).status == MemoryStatus::ok);
    check(memory.owns() && memory.data() != nullptr);
    check(reinterpret_cast<std::uintptr_t>(memory.data()) % page == 0);
    for (std::size_t index = 0; index < memory.size().usable; ++index) {
        check(memory.data()[index] == std::byte{0});
        memory.data()[index] = std::byte{0x5a};
    }
    check(memory.data()[0] == std::byte{0x5a});
    check(memory.data()[memory.size().usable - 1] == std::byte{0x5a});
    check(memory.release().status == MemoryStatus::ok);
    check(!memory.owns() && memory.data() == nullptr && memory.size().total == 0);
}

void ownership_is_exclusive_and_reusable() {
    static_assert(!std::is_copy_constructible_v<StackMemory>);
    static_assert(!std::is_move_constructible_v<StackMemory>);
    const auto page = page_size();
    StackMemory memory;
    check(memory.release().status == MemoryStatus::ok);
    check(memory.allocate(page).status == MemoryStatus::ok);
    auto *const data = memory.data();
    check(memory.allocate(page * 2).status == MemoryStatus::invalid);
    check(memory.data() == data && memory.size().usable == page);
    check(memory.release().status == MemoryStatus::ok);
    check(memory.release().status == MemoryStatus::ok);
    check(memory.allocate(page * 2).status == MemoryStatus::ok);
    check(memory.release().status == MemoryStatus::ok);
}

void allocation_failure_has_no_owner() {
    StackMemory memory;
    fail_map = true;
    const auto result = memory.allocate(page_size());
    fail_map = false;
    check(result.status == MemoryStatus::allocation_failed && result.error == ENOMEM);
    check(!memory.owns() && memory.data() == nullptr);
}

void failed_protection_rolls_back() {
    StackMemory memory;
    const int before = releases;
    fail_protect = true;
    const auto result = memory.allocate(page_size());
    fail_protect = false;
    check(result.status == MemoryStatus::protection_failed && result.error == EACCES);
    check(result.rollback_error == 0 && releases == before + 1);
    check(!memory.owns() && memory.data() == nullptr);
}

void failed_rollback_retains_mapping() {
    StackMemory memory;
    const auto page = page_size();
    fail_protect = true;
    fail_release = true;
    const auto result = memory.allocate(page);
    fail_protect = false;
    fail_release = false;
    check(result.status == MemoryStatus::protection_failed && result.error == EACCES);
    check(result.rollback_error == EIO);
    check(memory.owns() && memory.data() == nullptr && memory.size().total == page * 3);
    check(memory.allocate(page).status == MemoryStatus::invalid);
    check(memory.release().status == MemoryStatus::ok);
    check(!memory.owns());
}

void failed_release_preserves_owner_and_payload() {
    StackMemory memory;
    check(memory.allocate(page_size()).status == MemoryStatus::ok);
    auto *const data = memory.data();
    data[0] = std::byte{0x3c};
    fail_release = true;
    const auto result = memory.release();
    fail_release = false;
    check(result.status == MemoryStatus::release_failed && result.error == EIO);
    check(memory.owns() && memory.data() == data && data[0] == std::byte{0x3c});
    check(memory.release().status == MemoryStatus::ok);
}

struct Owner final {
public:
    StackMemory memory;
    bool released = false;

    static Panic release(void *data) noexcept {
        auto &owner = *static_cast<Owner *>(data);
        const auto result = owner.memory.release();
        owner.released = result.status == MemoryStatus::ok;
        return owner.released ? Panic{} : Panic{6, "stack release failed"};
    }
};

void explicit_cleanup_releases_mapping() {
    std::array<Entry, 1> entries;
    Stack cleanup(entries);
    const auto root = cleanup.mark();
    const auto slot = cleanup.reserve();
    check(slot.status == Status::ok);
    Owner owner;
    check(owner.memory.allocate(page_size()).status == MemoryStatus::ok);
    check(cleanup.arm(slot.token, &owner, Owner::release, "stack allocation") == Status::ok);
    check(cleanup.unwind(root, Reason::cancel).status == Status::ok);
    check(owner.released && !owner.memory.owns());
}

void os_admission_failure() {
    const auto page = page_size();
    rlimit previous{};
    check(getrlimit(RLIMIT_AS, &previous) == 0);
    const rlimit limited{1, previous.rlim_max};
    check(setrlimit(RLIMIT_AS, &limited) == 0);
    StackMemory memory;
    const auto result = memory.allocate(page);
    check(setrlimit(RLIMIT_AS, &previous) == 0);
    check(result.status == MemoryStatus::allocation_failed && result.error == ENOMEM);
    check(!memory.owns() && memory.data() == nullptr);
    std::printf("PASS kernel admission refusal: ENOMEM without owner\n");
}

alignas(16) std::array<std::byte, 131072> signal_stack;
std::atomic<std::uintptr_t> expected{0};
volatile sig_atomic_t high_guard = 0;
static_assert(std::atomic<std::uintptr_t>::is_always_lock_free);

void fault(int signal, siginfo_t *info, void *) noexcept {
    const char marker = 0;
    const auto position = reinterpret_cast<std::uintptr_t>(&marker);
    const auto begin = reinterpret_cast<std::uintptr_t>(signal_stack.data());
    if (signal != SIGSEGV || info->si_code != SEGV_ACCERR ||
        reinterpret_cast<std::uintptr_t>(info->si_addr) != expected.load(std::memory_order_relaxed) ||
        position < begin || position >= begin + signal_stack.size()) {
        std::_Exit(5);
    }
    constexpr std::string_view low = "guard-low: SEGV_ACCERR at expected address on alternate stack\n";
    constexpr std::string_view high = "guard-high: SEGV_ACCERR at expected address on alternate stack\n";
    const auto message = high_guard ? high : low;
    if (::write(STDERR_FILENO, message.data(), message.size()) != static_cast<ssize_t>(message.size())) {
        std::_Exit(6);
    }
}

[[gnu::no_sanitize("address", "undefined")]] void touch(std::uintptr_t address) {
    *reinterpret_cast<volatile unsigned char *>(address) = 0xaa;
}

void guard_fault(bool high) {
    const rlimit limit{0, 0};
    check(setrlimit(RLIMIT_CORE, &limit) == 0);
    StackMemory memory;
    check(memory.allocate(page_size() * 2).status == MemoryStatus::ok);
    stack_t alternate{};
    alternate.ss_sp = signal_stack.data();
    alternate.ss_size = signal_stack.size();
    check(sigaltstack(&alternate, nullptr) == 0);
    struct sigaction action{};
    action.sa_sigaction = fault;
    action.sa_flags = SA_SIGINFO | SA_ONSTACK | SA_RESETHAND;
    check(sigemptyset(&action.sa_mask) == 0);
    check(sigaction(SIGSEGV, &action, nullptr) == 0);
    const auto start = reinterpret_cast<std::uintptr_t>(memory.data());
    const auto address = high ? start + memory.size().usable : start - 1;
    expected.store(address, std::memory_order_relaxed);
    high_guard = high;
    touch(address);
    std::_Exit(3);
}

struct Case final {
public:
    std::string_view name;
    void (*run)();
};

constexpr std::array cases{
    Case{"layout_accounts_for_guards", layout_accounts_for_guards},
    Case{"invalid_sizes_do_not_allocate", invalid_sizes_do_not_allocate},
    Case{"overflow_is_checked", overflow_is_checked},
    Case{"payload_is_writable_and_zeroed", payload_is_writable_and_zeroed},
    Case{"ownership_is_exclusive_and_reusable", ownership_is_exclusive_and_reusable},
    Case{"allocation_failure_has_no_owner", allocation_failure_has_no_owner},
    Case{"failed_protection_rolls_back", failed_protection_rolls_back},
    Case{"failed_rollback_retains_mapping", failed_rollback_retains_mapping},
    Case{"failed_release_preserves_owner_and_payload", failed_release_preserves_owner_and_payload},
    Case{"explicit_cleanup_releases_mapping", explicit_cleanup_releases_mapping},
};

}

extern "C" void *__real_mmap(void *, std::size_t, int, int, int, off_t);
extern "C" int __real_mprotect(void *, std::size_t, int);
extern "C" int __real_munmap(void *, std::size_t);

extern "C" void *__wrap_mmap(void *address, std::size_t size, int protection,
                             int flags, int descriptor, off_t offset) {
    if (fail_map) {
        errno = ENOMEM;
        return MAP_FAILED;
    }
    return __real_mmap(address, size, protection, flags, descriptor, offset);
}

extern "C" int __wrap_mprotect(void *address, std::size_t size, int protection) {
    if (fail_protect) {
        errno = EACCES;
        return -1;
    }
    return __real_mprotect(address, size, protection);
}

extern "C" int __wrap_munmap(void *address, std::size_t size) {
    ++releases;
    if (fail_release) {
        errno = EIO;
        return -1;
    }
    return __real_munmap(address, size);
}

int main(int argc, char **argv) {
    if (argc == 2 && std::string_view(argv[1]) == "--guard-low") {
        guard_fault(false);
    }
    if (argc == 2 && std::string_view(argv[1]) == "--guard-high") {
        guard_fault(true);
    }
    if (argc == 2 && std::string_view(argv[1]) == "--os-admission-failure") {
        os_admission_failure();
        return 0;
    }
    if (argc != 1) {
        return 2;
    }
    for (const auto &test : cases) {
        test.run();
        std::printf("PASS %.*s\n", static_cast<int>(test.name.size()), test.name.data());
    }
    std::printf("%zu stack allocation cases passed\n", cases.size());
    return 0;
}
