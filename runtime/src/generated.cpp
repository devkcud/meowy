#include "meowy/generated.hpp"
#include "meowy/cleanup.hpp"
#include "meowy/owned.hpp"

#include <cstdlib>

#include <limits>
#include <memory>
#include <type_traits>

namespace {
using namespace meowy::prototype::v0;

struct Record final {
public:
    void *data = nullptr;
    MeowyCleanupDrop drop = nullptr;
};

class Frame final {
public:
    explicit Frame(std::uint64_t count) noexcept
        : capacity(count), records(reinterpret_cast<Record *>(reinterpret_cast<std::byte *>(this) +
              sizeof(Frame) + count * sizeof(Entry))),
          stack({reinterpret_cast<Entry *>(reinterpret_cast<std::byte *>(this) + sizeof(Frame)), count}) {
        auto *entries = reinterpret_cast<Entry *>(reinterpret_cast<std::byte *>(this) + sizeof(Frame));
        for (std::uint64_t i = 0; i < count; ++i) {
            std::construct_at(entries + i);
            std::construct_at(records + i);
        }
    }

    static Panic drop(void *data) noexcept {
        const auto &record = *static_cast<Record *>(data);
        Panic panic;
        record.drop(record.data, &panic);
        return panic;
    }

    const std::uint64_t capacity;
    Record *const records;
    Stack stack;
    bool busy = false;
};

static_assert(sizeof(void *) == 8 && sizeof(std::size_t) == 8);
static_assert(sizeof(MeowyCleanupToken) == 24 && alignof(MeowyCleanupToken) == 8);
static_assert(sizeof(MeowyCleanupMark) == 24 && alignof(MeowyCleanupMark) == 8);
static_assert(offsetof(MeowyCleanupToken, index) == 8 && offsetof(MeowyCleanupToken, id) == 16);
static_assert(offsetof(MeowyCleanupMark, depth) == 8 && offsetof(MeowyCleanupMark, anchor) == 16);
static_assert(std::is_standard_layout_v<MeowyCleanupToken> && std::is_standard_layout_v<MeowyCleanupMark>);
static_assert(alignof(Frame) >= alignof(Entry) && alignof(Frame) >= alignof(Record));
static_assert(sizeof(Frame) % alignof(Entry) == 0 && sizeof(Entry) % alignof(Record) == 0);
static_assert(alignof(Frame) >= alignof(Panic) && alignof(Frame) <= 16);
static_assert(std::is_trivially_destructible_v<Entry> && std::is_trivially_destructible_v<Record>);
static_assert(std::is_trivially_destructible_v<Panic>);
static_assert(alignof(Frame) >= alignof(Owned) && alignof(Frame) >= alignof(ValueOps));

bool aligned(const void *data, std::size_t alignment) noexcept {
    return data != nullptr && reinterpret_cast<std::uintptr_t>(data) % alignment == 0;
}

Frame *frame(void *data) noexcept {
    if (!aligned(data, alignof(Frame))) {
        return nullptr;
    }
    auto *value = static_cast<Frame *>(data);
    return value->busy ? nullptr : value;
}

std::uint32_t status(Status value) noexcept {
    return static_cast<std::uint32_t>(value);
}
}

extern "C" std::uint64_t meowy_cleanup_bytes_v0(std::uint64_t capacity) noexcept {
    constexpr auto limit = (std::numeric_limits<std::uint64_t>::max() - sizeof(Frame)) /
                           (sizeof(Entry) + sizeof(Record));
    return capacity > limit ? 0 : sizeof(Frame) + capacity * (sizeof(Entry) + sizeof(Record));
}

extern "C" std::uint64_t meowy_cleanup_alignment_v0() noexcept { return alignof(Frame); }

extern "C" std::uint32_t meowy_cleanup_open_v0(void *storage, std::uint64_t bytes,
                                               std::uint64_t capacity) noexcept {
    const auto required = meowy_cleanup_bytes_v0(capacity);
    if (!aligned(storage, alignof(Frame)) || required == 0 || bytes < required) {
        return status(Status::invalid);
    }
    std::construct_at(static_cast<Frame *>(storage), capacity);
    return status(Status::ok);
}

extern "C" std::uint32_t meowy_cleanup_mark_v0(void *data, MeowyCleanupMark *output) noexcept {
    const auto *value = frame(data);
    if (value == nullptr || output == nullptr) { return status(Status::invalid); }
    const auto mark = value->stack.mark();
    *output = {mark.stack, mark.depth, mark.anchor};
    return status(Status::ok);
}

extern "C" std::uint32_t meowy_cleanup_reserve_v0(void *data, MeowyCleanupToken *output) noexcept {
    auto *value = frame(data);
    if (value == nullptr || output == nullptr) { return status(Status::invalid); }
    const auto slot = value->stack.reserve();
    if (slot.status == Status::ok) {
        *output = {slot.token.stack, slot.token.index, slot.token.id};
    }
    return status(slot.status);
}

extern "C" std::uint32_t meowy_cleanup_arm_v0(void *data, const MeowyCleanupToken *token,
                                              void *payload, MeowyCleanupDrop drop,
                                              const char *name, std::uint64_t size) noexcept {
    auto *value = frame(data);
    if (value == nullptr || token == nullptr || token->index >= value->capacity ||
        drop == nullptr || name == nullptr || size == 0) { return status(Status::invalid); }
    auto &record = value->records[token->index];
    const auto result = value->stack.arm({static_cast<const Stack *>(token->owner), token->index, token->id},
                                          &record, Frame::drop, {name, size});
    if (result == Status::ok) { record = {payload, drop}; }
    return status(result);
}

extern "C" std::uint32_t meowy_cleanup_disarm_v0(void *data, const MeowyCleanupToken *token) noexcept {
    auto *value = frame(data);
    if (value == nullptr || token == nullptr) { return status(Status::invalid); }
    return status(value->stack.disarm({static_cast<const Stack *>(token->owner), token->index, token->id}));
}

extern "C" std::uint32_t meowy_cleanup_unwind_v0(void *data, const MeowyCleanupMark *mark,
                                                 std::uint32_t reason, const void *panic) noexcept {
    auto *value = frame(data);
    if (value == nullptr || mark == nullptr || reason > static_cast<std::uint32_t>(Reason::cancel) ||
        (reason == static_cast<std::uint32_t>(Reason::panic) && panic == nullptr) ||
        (panic != nullptr && !aligned(panic, alignof(Panic)))) { return status(Status::invalid); }
    const auto first = panic == nullptr ? Panic{} : *static_cast<const Panic *>(panic);
    value->busy = true;
    const auto result = value->stack.unwind({static_cast<const Stack *>(mark->owner), mark->depth, mark->anchor},
                                            static_cast<Reason>(reason), first);
    value->busy = false;
    return status(result.status);
}

extern "C" std::uint32_t meowy_cleanup_finish_v0(void *data) noexcept {
    auto *value = frame(data);
    if (value == nullptr || value->stack.size() != 0) { return status(Status::invalid); }
    std::destroy_at(value);
    return status(Status::ok);
}

extern "C" std::uint64_t meowy_cleanup_panic_bytes_v0() noexcept { return sizeof(Panic); }

extern "C" std::uint32_t meowy_cleanup_panic_init_v0(void *storage, std::uint64_t bytes,
                                                     std::uint32_t code, const char *text,
                                                     std::uint64_t size) noexcept {
    if (!aligned(storage, alignof(Panic)) || bytes < sizeof(Panic) || (text == nullptr && size != 0)) {
        return status(Status::invalid);
    }
    const Panic snapshot(code, {text == nullptr ? "" : text, size});
    std::construct_at(static_cast<Panic *>(storage), snapshot);
    return status(Status::ok);
}

namespace meowy::prototype::v0 {
class Generated final {
public:
    static void drop(void *data, void *panic) noexcept {
        *static_cast<Panic *>(panic) = Owned::drop(data);
    }

    static std::uint32_t arm(void *data, const MeowyCleanupToken *token, void *owner) noexcept {
        if (!aligned(owner, alignof(Owned))) { return static_cast<std::uint32_t>(OwnedStatus::invalid); }
        const auto &value = *static_cast<Owned *>(owner);
        if (!value.initialized()) { return static_cast<std::uint32_t>(OwnedStatus::invalid); }
        const auto result = meowy_cleanup_arm_v0(data, token, owner, drop, value.ops->name.data(), value.ops->name.size());
        return static_cast<std::uint32_t>(result == 0 ? OwnedStatus::ok : OwnedStatus::invalid);
    }

    static std::uint32_t transfer(void *from, const MeowyCleanupToken *old, void *source,
                                  void *to, const MeowyCleanupToken *next, void *destination) noexcept {
        auto *a = frame(from);
        auto *b = frame(to);
        if (a == nullptr || b == nullptr || old == nullptr || next == nullptr ||
            !aligned(source, alignof(Owned)) || !aligned(destination, alignof(Owned)) ||
            old->index >= a->capacity || next->index >= b->capacity) {
            return static_cast<std::uint32_t>(OwnedStatus::invalid);
        }
        const Token before{static_cast<const Stack *>(old->owner), old->index, old->id};
        const Token after{static_cast<const Stack *>(next->owner), next->index, next->id};
        if (a->stack.can_rebind(before, b->stack, after) != Status::ok ||
            a->records[old->index].data != source || a->records[old->index].drop != drop) {
            return static_cast<std::uint32_t>(OwnedStatus::invalid);
        }
        auto &src = *static_cast<Owned *>(source);
        auto &dst = *static_cast<Owned *>(destination);
        const auto ready = src.fits(dst);
        if (ready != OwnedStatus::ok) { return static_cast<std::uint32_t>(ready); }
        a->busy = true;
        b->busy = true;
        const auto moved = src.move_to(dst);
        if (moved != OwnedStatus::ok) { std::abort(); }
        auto &record = b->records[next->index];
        record = {destination, drop};
        if (b->stack.arm(after, &record, Frame::drop, dst.ops->name) != Status::ok ||
            a->stack.disarm(before) != Status::ok) { std::abort(); }
        a->busy = false;
        b->busy = false;
        return static_cast<std::uint32_t>(OwnedStatus::ok);
    }
};
}

extern "C" std::uint64_t meowy_owned_bytes_v0() noexcept { return sizeof(Owned); }
extern "C" std::uint64_t meowy_owned_ops_bytes_v0() noexcept { return sizeof(ValueOps); }

extern "C" std::uint32_t meowy_owned_ops_init_v0(void *storage, std::uint64_t bytes,
                                                std::uint64_t size, std::uint64_t alignment,
                                                void (*move)(void *, void *) noexcept, MeowyCleanupDrop drop,
                                                const char *name, std::uint64_t length) noexcept {
    if (!aligned(storage, alignof(ValueOps)) || bytes < sizeof(ValueOps) || size == 0 || alignment == 0 ||
        (alignment & (alignment - 1)) != 0 || move == nullptr || drop == nullptr || name == nullptr || length == 0) {
        return static_cast<std::uint32_t>(OwnedStatus::invalid);
    }
    std::construct_at(static_cast<ValueOps *>(storage), ValueOps{size, alignment, move, nullptr, {name, length}, drop});
    return static_cast<std::uint32_t>(OwnedStatus::ok);
}

extern "C" std::uint32_t meowy_owned_open_v0(void *storage, std::uint64_t bytes,
                                            void *payload, std::uint64_t capacity) noexcept {
    if (!aligned(storage, alignof(Owned)) || bytes < sizeof(Owned) || (payload == nullptr && capacity != 0)) {
        return static_cast<std::uint32_t>(OwnedStatus::invalid);
    }
    std::construct_at(static_cast<Owned *>(storage), std::span<std::byte>(static_cast<std::byte *>(payload), capacity));
    return static_cast<std::uint32_t>(OwnedStatus::ok);
}

extern "C" std::uint32_t meowy_owned_reserve_v0(void *owner, const void *ops) noexcept {
    if (!aligned(owner, alignof(Owned)) || !aligned(ops, alignof(ValueOps))) {
        return static_cast<std::uint32_t>(OwnedStatus::invalid);
    }
    return static_cast<std::uint32_t>(static_cast<Owned *>(owner)->reserve(*static_cast<const ValueOps *>(ops)));
}

extern "C" void *meowy_owned_data_v0(void *owner) noexcept {
    return aligned(owner, alignof(Owned)) ? static_cast<Owned *>(owner)->data() : nullptr;
}

extern "C" std::uint32_t meowy_owned_commit_v0(void *owner) noexcept {
    return static_cast<std::uint32_t>(aligned(owner, alignof(Owned)) ?
        static_cast<Owned *>(owner)->commit() : OwnedStatus::invalid);
}

extern "C" std::uint32_t meowy_owned_release_v0(void *owner) noexcept {
    return static_cast<std::uint32_t>(aligned(owner, alignof(Owned)) ?
        static_cast<Owned *>(owner)->release() : OwnedStatus::invalid);
}

extern "C" std::uint32_t meowy_owned_finish_v0(void *owner) noexcept {
    if (!aligned(owner, alignof(Owned)) || !static_cast<Owned *>(owner)->empty()) {
        return static_cast<std::uint32_t>(OwnedStatus::invalid);
    }
    std::destroy_at(static_cast<Owned *>(owner));
    return static_cast<std::uint32_t>(OwnedStatus::ok);
}

extern "C" std::uint32_t meowy_owned_arm_v0(void *data, const MeowyCleanupToken *token, void *owner) noexcept {
    return Generated::arm(data, token, owner);
}

extern "C" std::uint32_t meowy_owned_transfer_v0(void *from, const MeowyCleanupToken *old, void *source,
                                                void *to, const MeowyCleanupToken *next, void *destination) noexcept {
    return Generated::transfer(from, old, source, to, next, destination);
}
