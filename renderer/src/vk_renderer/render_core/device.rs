
use ash::{
    vk,
    Instance,
    Device,
    extensions::khr::{Surface, Swapchain},
    version::{InstanceV1_0, DeviceV1_0}
};
use std::os::raw::c_char;
use defs::render::FeatureDeclaration;

#[derive(Copy, Clone)]
pub struct PhysicalDeviceProperties {
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue_family_index: u32,
    pub transfer_queue_family_index: u32,
    pub required_features: vk::PhysicalDeviceFeatures
}

pub unsafe fn make_device_resources(instance: &Instance, surface_fn: &Surface, surface: &vk::SurfaceKHR, features: &Vec<FeatureDeclaration>) -> Result<(Device, PhysicalDeviceProperties, vk::Queue, vk::Queue), String> {

    let physical_device_properties = select_physical_device(&instance, surface_fn, surface, features)?;

    // Find queue indices for graphics and transfer (ideally different but could be the same)
    let queue_family_properties = instance
        .get_physical_device_queue_family_properties(physical_device_properties.physical_device);
    let (graphics_queue_family_index, transfer_queue_family_index) = {
        let mut found_graphics_queue_index = None;
        let mut found_transfer_queue_index = None;
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                found_graphics_queue_index = Some(index as u32);
            }
            if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                if found_transfer_queue_index.is_none() || !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    found_transfer_queue_index = Some(index as u32);
                }
            }
        }
        (
            found_graphics_queue_index.unwrap(),
            found_transfer_queue_index.unwrap()
        )
    };

    // Device extensions required
    let device_extensions: Vec<*const c_char> = vec![ Swapchain::name().as_ptr() ];

    // Make the logical device
    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(transfer_queue_family_index)
            .queue_priorities(&priorities)
            .build()
    ];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extensions)
        .enabled_features(&physical_device_properties.required_features);
    let device = instance
        .create_device(physical_device_properties.physical_device, &device_create_info, None)
        .map_err(|e| format!("{:?}", e))?;

    // Get queues
    let graphics_queue = device.get_device_queue(graphics_queue_family_index, 0);
    let transfer_queue = device.get_device_queue(transfer_queue_family_index, 0);

    Ok((device, physical_device_properties, graphics_queue, transfer_queue))
}

unsafe fn select_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::extensions:: khr::Surface,
    surface: &vk::SurfaceKHR,
    features: &Vec<FeatureDeclaration>
) -> Result<PhysicalDeviceProperties, String> {

    let physical_devices = instance
        .enumerate_physical_devices()
        .map_err(|e| format!("{:?}", e))?;
    if physical_devices.is_empty() {
        return Err(String::from("No physical devices found"));
    }

    let unset_value: u32 = u32::MAX;
    for physical_device in physical_devices.iter() {
        let queue_family_properties = instance.get_physical_device_queue_family_properties(*physical_device);
        let mut graphics_index: u32 = unset_value;
        let mut transfer_index: u32 = unset_value;
        let mut feature_set_to_enable = vk::PhysicalDeviceFeatures::default();
        for (index, properties) in queue_family_properties.iter().enumerate() {

            let supports_graphics = properties.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            let supports_surface = surface_loader
                .get_physical_device_surface_support(*physical_device, index as u32, *surface)
                .unwrap();
            let supports_transfer = properties.queue_flags.contains(vk::QueueFlags::TRANSFER);

            let supported_features = instance.get_physical_device_features(*physical_device);
            feature_set_to_enable = match make_feature_set_to_enable(features, &supported_features) {
                Some(features) => features,
                None => continue
            };

            let graphics_and_surface = supports_graphics && supports_surface;
            if graphics_and_surface {
                graphics_index = index as u32;
            }
            if supports_transfer && (transfer_index == unset_value || !graphics_and_surface) {
                transfer_index = index as u32;
            }
        }
        if graphics_index != unset_value && transfer_index != unset_value {
            return Ok(PhysicalDeviceProperties {
                physical_device: *physical_device,
                graphics_queue_family_index: graphics_index,
                transfer_queue_family_index: transfer_index,
                required_features: feature_set_to_enable
            });
        }
    }

    Err(String::from("Could not find a suitable physical device"))
}

/// Return set of features to enable during device creation, knowing that all of those features
/// are supported by the physical device. If they are not all supported, this returns None.
fn make_feature_set_to_enable(features: &Vec<FeatureDeclaration>, supported_features: &vk::PhysicalDeviceFeatures) -> Option<vk::PhysicalDeviceFeatures> {
    let mut features_to_enable = vk::PhysicalDeviceFeatures::default();
    for feature in features.iter() {
        match feature {
            FeatureDeclaration::ClipPlanes => {
                if supported_features.shader_clip_distance == vk::TRUE {
                    features_to_enable.shader_clip_distance = vk::TRUE;
                } else {
                    return None;
                }
            }
        }
    }
    Some(features_to_enable)
}
