#include "meowy/strings.hpp"

#include <cstdlib>
#include <cstring>
#include <new>

namespace meowy::prototype::v0 {
namespace {

void *allocate(void *, std::size_t size) noexcept {
    return std::malloc(size);
}

void release(void *, void *data, std::size_t) noexcept {
    std::free(data);
}

StringStatus storage_status(OwnedStatus status) noexcept {
    switch (status) {
    case OwnedStatus::ok: return StringStatus::ok;
    case OwnedStatus::full: return StringStatus::full;
    case OwnedStatus::misaligned: return StringStatus::misaligned;
    case OwnedStatus::occupied: return StringStatus::occupied;
    default: return StringStatus::invalid;
    }
}

}

const Allocator &Strings::heap() noexcept {
    static const Allocator allocator{nullptr, allocate, release};
    return allocator;
}

const ValueOps &Strings::ops() noexcept {
    static const ValueOps descriptor{sizeof(Value), alignof(Value), move, drop, "strings.Owned"};
    return descriptor;
}

StringStatus Strings::copy(Owned &owner, std::string_view text, const Allocator &allocator,
                           AllocationFailure &failure) noexcept {
    if (allocator.allocate == nullptr || allocator.release == nullptr ||
        (text.data() == nullptr && !text.empty())) {
        return StringStatus::invalid;
    }
    const auto status = owner.reserve(ops());
    if (status != OwnedStatus::ok) {
        return storage_status(status);
    }
    void *const payload = owner.data();
    owner.busy = true;
    char *const data = text.empty() ? nullptr : static_cast<char *>(allocator.allocate(allocator.context, text.size()));
    if (data == nullptr && !text.empty()) {
        owner.busy = false;
        if (owner.release() != OwnedStatus::ok) {
            std::abort();
        }
        failure = {AllocationCause::exhausted, text.size(), 1};
        return StringStatus::allocation_failed;
    }
    if (!text.empty()) {
        std::memcpy(data, text.data(), text.size());
    }
    new (payload) Value{&allocator, data, text.size()};
    owner.busy = false;
    if (owner.commit() != OwnedStatus::ok) {
        std::abort();
    }
    failure = {};
    return StringStatus::ok;
}

bool Strings::view(const Owned &owner, std::string_view &text) noexcept {
    if (!owner.initialized() || owner.ops != &ops()) {
        return false;
    }
    const auto &value = *static_cast<const Value *>(owner.data());
    text = value.size == 0 ? std::string_view{} : std::string_view{value.data, value.size};
    return true;
}

void Strings::move(void *destination, void *source) noexcept {
    auto *const value = static_cast<Value *>(source);
    new (destination) Value{*value};
    value->~Value();
}

Panic Strings::drop(void *data) noexcept {
    auto *const value = static_cast<Value *>(data);
    if (value->data != nullptr) {
        value->allocator->release(value->allocator->context, value->data, value->size);
    }
    value->~Value();
    return {};
}

}

using namespace meowy::prototype::v0;

static_assert(sizeof(std::size_t) == sizeof(std::uint64_t));
static_assert(sizeof(AllocationFailure) == 24 && alignof(AllocationFailure) == 8);
static_assert(offsetof(AllocationFailure, bytes) == 8 && offsetof(AllocationFailure, alignment) == 16);

extern "C" const void *meowy_string_heap_v0() noexcept {
    return &Strings::heap();
}

extern "C" const void *meowy_string_ops_v0() noexcept {
    return &Strings::ops();
}

extern "C" std::uint64_t meowy_string_bytes_v0() noexcept {
    return Strings::ops().size;
}

extern "C" std::uint64_t meowy_string_alignment_v0() noexcept {
    return Strings::ops().alignment;
}

extern "C" std::uint32_t meowy_string_copy_v0(void *owner, const char *text, std::uint64_t size,
                                              const void *allocator, AllocationFailure *failure) noexcept {
    if (owner == nullptr || allocator == nullptr || failure == nullptr || (text == nullptr && size != 0)) {
        return static_cast<std::uint32_t>(StringStatus::invalid);
    }
    return static_cast<std::uint32_t>(Strings::copy(*static_cast<Owned *>(owner), {text, size},
                                                   *static_cast<const Allocator *>(allocator), *failure));
}

extern "C" std::uint32_t meowy_string_view_v0(const void *owner, const char **text, std::uint64_t *size) noexcept {
    if (owner == nullptr || text == nullptr || size == nullptr) {
        return static_cast<std::uint32_t>(StringStatus::invalid);
    }
    std::string_view value;
    if (!Strings::view(*static_cast<const Owned *>(owner), value)) {
        return static_cast<std::uint32_t>(StringStatus::invalid);
    }
    *text = value.data();
    *size = value.size();
    return static_cast<std::uint32_t>(StringStatus::ok);
}
