#$ test: parse-pass
#$ rules: GRM-35, FFI-11, LEX-15
# `[GRM-35]` — an overlay file: `unsafe overlay`, then entries with their
# attributes: a `type`, an `enum`, functions whose parameters take contract
# words in place of types (separated by spaces, a word with arguments, a
# parameter with none) and a result `-> status`, `rename … as …`, `hide …`, and
# an `extend` block of ordinary Ember wrappers (`[FFI-13]`). `overlay`,
# `rename` and `hide` are keywords only there (`[LEX-15]`).

unsafe overlay c "vulkan/vulkan.h":
    @ffi(handle)
    type VkBuffer
    @ffi(handle)
    type VkDevice
    @ffi(status, ok=VK_SUCCESS)
    enum VkResult

    @ffi(effects=[FFI, Alloc])
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status

    fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount,
                        pViewports: borrowed count(viewportCount))

    @grade(asserted)
    fn vkGetDeviceQueue(device: borrowed, queue: out one, flags: TODO(count) unsafe)

    rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction

    extend VkDevice:
        fn idle(self) -> bool:
            return true

overlay cpp "engine/Layer.hpp":
    rename rename_me as renamed
    hide hide

fn main():
    rename = 1
    hide = rename + 1
    overlay = hide
    println(overlay)
