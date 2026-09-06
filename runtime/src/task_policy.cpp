#include "meowy/scheduler.hpp"

namespace meowy::prototype::v0 {

std::size_t select_task(std::span<const TaskSlot> slots, std::size_t cursor) noexcept {
    if (slots.empty()) {
        return 0;
    }
    std::size_t index = cursor % slots.size();
    for (std::size_t count = 0; count < slots.size(); ++count) {
        if (slots[index].runnable()) {
            return index;
        }
        index = index + 1 == slots.size() ? 0 : index + 1;
    }
    return slots.size();
}

}
