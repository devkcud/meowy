#include "meowy/stack_memory.hpp"

#include <cerrno>
#include <cstdint>
#include <limits>
#include <sys/mman.h>
#include <unistd.h>

namespace meowy::prototype::v0 {

StackPlan StackMemory::plan(std::size_t bytes, std::size_t page) noexcept {
    if (bytes == 0 || page == 0 || bytes % page != 0) {
        return {{MemoryStatus::invalid}, {}};
    }
    constexpr auto limit = static_cast<std::size_t>(std::numeric_limits<std::ptrdiff_t>::max());
    if (page > limit / 2 || bytes > limit - page * 2) {
        return {{MemoryStatus::overflow}, {}};
    }
    return {{}, {bytes, page, bytes + page * 2}};
}

MemoryResult StackMemory::allocate(std::size_t bytes) noexcept {
    if (owns()) {
        return {MemoryStatus::invalid};
    }
    errno = 0;
    const auto page = sysconf(_SC_PAGESIZE);
    if (page <= 0) {
        return {MemoryStatus::system_failed, errno == 0 ? EINVAL : errno};
    }
    const auto request = plan(bytes, static_cast<std::size_t>(page));
    if (request.result.status != MemoryStatus::ok) {
        return request.result;
    }
    void *const area = mmap(nullptr, request.size.total, PROT_NONE,
                            MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (area == MAP_FAILED) {
        return {MemoryStatus::allocation_failed, errno};
    }
    base = area;
    layout = request.size;
    owned = true;
    if (mprotect(reinterpret_cast<void *>(reinterpret_cast<std::uintptr_t>(base) + layout.guard), layout.usable,
                 PROT_READ | PROT_WRITE) != 0) {
        const int error = errno;
        const auto rollback = release();
        return {MemoryStatus::protection_failed, error, rollback.error};
    }
    ready = true;
    return {};
}

MemoryResult StackMemory::release() noexcept {
    if (!owns()) {
        return {};
    }
    if (munmap(base, layout.total) != 0) {
        return {MemoryStatus::release_failed, errno};
    }
    base = nullptr;
    layout = {};
    owned = false;
    ready = false;
    return {};
}

bool StackMemory::owns() const noexcept {
    return owned;
}

std::byte *StackMemory::data() noexcept {
    return ready ? reinterpret_cast<std::byte *>(reinterpret_cast<std::uintptr_t>(base) + layout.guard) : nullptr;
}

const std::byte *StackMemory::data() const noexcept {
    return ready ? reinterpret_cast<const std::byte *>(reinterpret_cast<std::uintptr_t>(base) + layout.guard) : nullptr;
}

StackSize StackMemory::size() const noexcept {
    return layout;
}

}
