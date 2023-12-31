use std::sync::Arc;

use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::image::view::ImageView;

use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo, Pipeline, PipelineBindPoint};
use vulkano::pipeline::graphics::vertex_input::Vertex;

use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo};
use vulkano::{VulkanLibrary, command_buffer};
use vulkano::instance::{InstanceCreateInfo, Instance};
use vulkano::device::{QueueFlags, Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::memory::allocator::{StandardMemoryAllocator, AllocationCreateInfo, MemoryTypeFilter};
use vulkano::buffer::{BufferContents, Buffer, BufferCreateInfo, BufferUsage};

use vulkano::command_buffer::{ClearColorImageInfo, AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo};

use vulkano::image::{Image, ImageType, ImageUsage, ImageCreateInfo};
use vulkano::format::{Format, ClearColorValue};

use vulkano::sync::{self, GpuFuture};

use image::{ImageBuffer, Rgba};


fn main() {
    let library = VulkanLibrary::new().expect("No local Vulkan library found");
    let instance = Instance::new(library, InstanceCreateInfo::default())
        .expect("Failed to create instance");

    let physical_device = instance
        .enumerate_physical_devices()
        .expect("Could not enumerate devices")
        .next()
        .expect("No devices available");

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_queue_family_index, queue_family_properties)| {
            queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
        })
    .expect("Couldn't find any") as u32;

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        })
    .expect("Failed to create device");

    let queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));


    #[derive(BufferContents, Vertex)]
    #[repr(C)]
    struct MyVertex {
        #[format(R32G32_SFLOAT)]
        position: [f32; 2],
    }


    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: r"
                #version 460

                layout(location = 0) in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: r"
                #version 460

                layout(location = 0) out vec4 f_color;

                void main() {
                    f_color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
        }
    }


}
