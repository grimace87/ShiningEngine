
use ash::vk;

pub struct BufferWrapper {
    pub buffer: ash::vk::Buffer,
    allocation: vk_mem::Allocation
}

impl BufferWrapper {

    unsafe fn new_buffer(
        allocator: &vk_mem::Allocator,
        size_bytes: usize,
        buffer_usage: vk::BufferUsageFlags,
        mem_usage: vk_mem::MemoryUsage
    ) -> Result<BufferWrapper, String> {
        let buffer_create_info = ash::vk::BufferCreateInfo::builder()
            .size(size_bytes as u64)
            .usage(buffer_usage)
            .build();
        let memory_create_info = vk_mem::AllocationCreateInfo {
            usage: mem_usage,
            ..Default::default()
        };
        let (buffer, allocation, _) = allocator.create_buffer(&buffer_create_info, &memory_create_info)
            .map_err(|e| format!("Failed to create buffer: {:?}", e))?;

        Ok(BufferWrapper {
            buffer,
            allocation
        })
    }

    pub fn empty() -> BufferWrapper {
        BufferWrapper {
            buffer: vk::Buffer::null(),
            allocation: vk_mem::Allocation::null()
        }
    }

    pub unsafe fn destroy(&self, allocator: &vk_mem::Allocator) -> Result<(), String> {
        allocator.destroy_buffer(self.buffer, &self.allocation)
            .map_err(|e| format!("Error freeing buffer: {:?}", e))
    }

    pub unsafe fn new_vertex_buffer(
        allocator: &vk_mem::Allocator,
        size_bytes: usize
    ) -> Result<BufferWrapper, String> {
        Self::new_buffer(
            allocator,
            size_bytes,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu
        )
    }

    pub unsafe fn new_uniform_buffer(
        allocator: &vk_mem::Allocator,
        size_bytes: usize
    ) -> Result<BufferWrapper, String> {
        Self::new_buffer(
            allocator,
            size_bytes,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu
        )
    }

    pub unsafe fn new_staging_buffer(
        allocator: &vk_mem::Allocator,
        size_bytes: usize
    ) -> Result<BufferWrapper, String> {
        Self::new_buffer(
            allocator,
            size_bytes,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuToGpu
        )
    }

    pub unsafe fn update<T: Sized>(&mut self, allocator: &vk_mem::Allocator, buffer_data: *const T, element_count: usize) -> Result<(), String> {
        let dst_ptr = allocator.map_memory(&self.allocation)
            .map_err(|e| format!("Failed to map buffer memory: {:?}", e))? as *mut T;
        dst_ptr.copy_from_nonoverlapping(buffer_data as *const T, element_count);
        allocator.unmap_memory(&self.allocation).unwrap();
        Ok(())
    }

    pub unsafe fn update_from_vec<E: Sized>(&mut self, allocator: &vk_mem::Allocator, buffer_data: &Vec<E>) -> Result<(), String> {
        let dst_ptr = allocator.map_memory(&self.allocation)
            .map_err(|e| format!("Failed to map buffer memory: {:?}", e))? as *mut E;
        dst_ptr.copy_from_nonoverlapping(buffer_data.as_ptr() as *const E, buffer_data.len());
        allocator.unmap_memory(&self.allocation).unwrap();
        Ok(())
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }
}

