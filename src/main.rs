use std::sync::Arc;

use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
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


#[derive(BufferContents)]
#[repr(C)]
struct MyStruct {
    a: u32,
    b: u32
}

mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Data {
                uint data[];
            } buf;

            void main() {
                uint idx = gl_GlobalInvocationID.x;
                buf.data[idx] *= 12;
            }
        "
    }
}

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

    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [1024, 1024, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        }
        ).unwrap();

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
        );

    let mut builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator, 
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit
        ).unwrap();



    let buf = Buffer::from_iter(
        memory_allocator.clone(), 
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
        }, (0..1024 * 1024 * 4).map(|_| 0u8)
        )
        .expect("Failed to create buffer");

    builder
        .clear_color_image(ClearColorImageInfo {
            clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
            ..ClearColorImageInfo::image(image.clone())
        })
    .unwrap()
    .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                image.clone(), 
                buf.clone()
                ))
    .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();

    image.save("image.png").unwrap();

    println!("Image saved at image.png");

}
