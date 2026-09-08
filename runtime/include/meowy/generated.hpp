#pragma once

#include <cstdint>

struct MeowyCleanupToken final {
public:
    const void *owner;
    std::uint64_t index;
    std::uint64_t id;
};

struct MeowyCleanupMark final {
public:
    const void *owner;
    std::uint64_t depth;
    std::uint64_t anchor;
};

using MeowyCleanupDrop = void (*)(void *data, void *panic) noexcept;

extern "C" {
std::uint64_t meowy_cleanup_bytes_v0(std::uint64_t capacity) noexcept;
std::uint64_t meowy_cleanup_alignment_v0() noexcept;
std::uint32_t meowy_cleanup_open_v0(void *storage, std::uint64_t bytes,
                                    std::uint64_t capacity) noexcept;
std::uint32_t meowy_cleanup_mark_v0(void *frame, MeowyCleanupMark *mark) noexcept;
std::uint32_t meowy_cleanup_reserve_v0(void *frame, MeowyCleanupToken *token) noexcept;
std::uint32_t meowy_cleanup_arm_v0(void *frame, const MeowyCleanupToken *token,
                                   void *data, MeowyCleanupDrop drop,
                                   const char *name, std::uint64_t size) noexcept;
std::uint32_t meowy_cleanup_disarm_v0(void *frame, const MeowyCleanupToken *token) noexcept;
std::uint32_t meowy_cleanup_unwind_v0(void *frame, const MeowyCleanupMark *mark,
                                      std::uint32_t reason, const void *panic) noexcept;
std::uint32_t meowy_cleanup_finish_v0(void *frame) noexcept;
std::uint64_t meowy_cleanup_panic_bytes_v0() noexcept;
std::uint32_t meowy_cleanup_panic_init_v0(void *storage, std::uint64_t bytes,
                                          std::uint32_t code, const char *text,
                                          std::uint64_t size) noexcept;
std::uint64_t meowy_owned_bytes_v0() noexcept;
std::uint64_t meowy_owned_ops_bytes_v0() noexcept;
std::uint32_t meowy_owned_ops_init_v0(void *storage, std::uint64_t bytes,
                                      std::uint64_t size, std::uint64_t alignment,
                                      void (*move)(void *, void *) noexcept, MeowyCleanupDrop drop,
                                      const char *name, std::uint64_t length) noexcept;
std::uint32_t meowy_owned_open_v0(void *storage, std::uint64_t bytes,
                                  void *payload, std::uint64_t capacity) noexcept;
std::uint32_t meowy_owned_reserve_v0(void *owner, const void *ops) noexcept;
void *meowy_owned_data_v0(void *owner) noexcept;
std::uint32_t meowy_owned_commit_v0(void *owner) noexcept;
std::uint32_t meowy_owned_release_v0(void *owner) noexcept;
std::uint32_t meowy_owned_finish_v0(void *owner) noexcept;
std::uint32_t meowy_owned_arm_v0(void *frame, const MeowyCleanupToken *token, void *owner) noexcept;
std::uint32_t meowy_owned_transfer_v0(void *source_frame, const MeowyCleanupToken *source_token,
                                      void *source, void *destination_frame,
                                      const MeowyCleanupToken *destination_token, void *destination) noexcept;
}
