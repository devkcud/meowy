#include "meowy/owned.hpp"

#include <array>
#include <cstdint>
#include <cstdlib>

namespace meowy::prototype::v0 {

Owned::Owned(std::span<std::byte> storage) noexcept : bytes(storage) {}

OwnedStatus Owned::accepts(const ValueOps &value) const noexcept {
    if (busy || value.size == 0 || value.alignment == 0 ||
        (value.alignment & (value.alignment - 1)) != 0 || value.move == nullptr ||
        value.drop == nullptr || value.name.empty()) {
        return OwnedStatus::invalid;
    }
    if (phase != Phase::empty) {
        return OwnedStatus::occupied;
    }
    if (bytes.size() < value.size) {
        return OwnedStatus::full;
    }
    if (reinterpret_cast<std::uintptr_t>(bytes.data()) % value.alignment != 0) {
        return OwnedStatus::misaligned;
    }
    return OwnedStatus::ok;
}

OwnedStatus Owned::reserve(const ValueOps &value) noexcept {
    const auto status = accepts(value);
    if (status != OwnedStatus::ok) {
        return status;
    }
    ops = &value;
    phase = Phase::reserved;
    return OwnedStatus::ok;
}

OwnedStatus Owned::commit() noexcept {
    if (busy || phase != Phase::reserved) {
        return OwnedStatus::invalid;
    }
    phase = Phase::live;
    return OwnedStatus::ok;
}

OwnedStatus Owned::fits(const Owned &destination) const noexcept {
    if (!initialized() || this == &destination) {
        return OwnedStatus::invalid;
    }
    const auto status = destination.accepts(*ops);
    if (status != OwnedStatus::ok) {
        return status;
    }
    const auto source = reinterpret_cast<std::uintptr_t>(bytes.data());
    const auto target = reinterpret_cast<std::uintptr_t>(destination.bytes.data());
    if (source <= target ? target - source < bytes.size() : source - target < destination.bytes.size()) {
        return OwnedStatus::overlap;
    }
    return OwnedStatus::ok;
}

OwnedStatus Owned::move_to(Owned &destination) noexcept {
    const auto status = fits(destination);
    if (status != OwnedStatus::ok) {
        return status;
    }
    busy = true;
    destination.busy = true;
    ops->move(destination.bytes.data(), bytes.data());
    destination.ops = ops;
    destination.phase = Phase::live;
    ops = nullptr;
    phase = Phase::empty;
    destination.busy = false;
    busy = false;
    return OwnedStatus::ok;
}

OwnedStatus Owned::release() noexcept {
    if (busy) {
        return OwnedStatus::invalid;
    }
    if (phase != Phase::live) {
        phase = Phase::empty;
        ops = nullptr;
        return OwnedStatus::ok;
    }
    std::array<Entry, 1> entries;
    Stack cleanup(entries);
    const auto root = cleanup.mark();
    const auto slot = cleanup.reserve();
    if (slot.status != Status::ok || cleanup.arm(slot.token, this, drop, ops->name) != Status::ok ||
        cleanup.unwind(root, Reason::complete).status != Status::ok) {
        std::abort();
    }
    return OwnedStatus::ok;
}

Panic Owned::drop(void *data) noexcept {
    auto &value = *static_cast<Owned *>(data);
    if (value.busy) {
        std::abort();
    }
    if (value.phase != Phase::live) {
        return {};
    }
    value.busy = true;
    value.phase = Phase::empty;
    const auto panic = value.ops->drop(value.bytes.data());
    value.ops = nullptr;
    value.busy = false;
    return panic;
}

bool Owned::initialized() const noexcept {
    return !busy && phase == Phase::live;
}

bool Owned::empty() const noexcept {
    return !busy && phase == Phase::empty;
}

void *Owned::data() noexcept {
    return !busy && phase != Phase::empty ? bytes.data() : nullptr;
}

const void *Owned::data() const noexcept {
    return !busy && phase != Phase::empty ? bytes.data() : nullptr;
}

}
