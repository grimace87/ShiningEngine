
use crate::vk_renderer::{
    RenderCore,
    buffers::BufferWrapper,
    renderpass::RenderpassWrapper
};

use defs::{DrawingStep, Shader, VertexFormat};

use ash::{
    vk,
    version::DeviceV1_0
};
use std::ffi::CString;

pub struct PipelineWrapper {
    vertex_shader_module: vk::ShaderModule,
    fragment_shader_module: vk::ShaderModule,
    vertex_buffer: vk::Buffer,
    vertex_count: usize,
    uniform_buffer: BufferWrapper,
    texture_image_view: vk::ImageView,
    sampler: vk::Sampler,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline
}

impl PipelineWrapper {

    pub fn new() -> Result<PipelineWrapper, String> {
        Ok(PipelineWrapper {
            vertex_shader_module: vk::ShaderModule::null(),
            fragment_shader_module: vk::ShaderModule::null(),
            vertex_buffer: vk::Buffer::null(),
            vertex_count: 0,
            uniform_buffer: BufferWrapper::empty(),
            texture_image_view: vk::ImageView::null(),
            sampler: vk::Sampler::null(),
            descriptor_set_layout: vk::DescriptorSetLayout::null(),
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_sets: vec![],
            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null()
        })
    }

    pub fn destroy_resources(&self, render_core: &RenderCore) {
        let allocator = render_core.get_mem_allocator();
        unsafe {
            render_core.device.destroy_pipeline(self.pipeline, None);
            render_core.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.uniform_buffer.destroy(allocator).unwrap();
            render_core.device.destroy_descriptor_pool(self.descriptor_pool, None);
            render_core.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            render_core.device.destroy_sampler(self.sampler, None);
            render_core.device.destroy_shader_module(self.fragment_shader_module, None);
            render_core.device.destroy_shader_module(self.vertex_shader_module, None);
        }
    }

    pub unsafe fn create_resources(&mut self, render_core: &RenderCore, renderpass_wrapper: &RenderpassWrapper, description: &DrawingStep) -> Result<(), String> {

        // Make shader modules
        let vertex_shader_create_info = vk::ShaderModuleCreateInfo::builder()
            .code(match description.shader {
                Shader::PlainPnt => vk_shader_macros::include_glsl!("shaders/vk/triangle.vert"),
                Shader::Text => vk_shader_macros::include_glsl!("shaders/vk/text.vert")
            });
        let vertex_shader_module = render_core.device
            .create_shader_module(&vertex_shader_create_info, None)
            .map_err(|e| format!("{:?}", e))?;
        let fragment_shader_create_info = vk::ShaderModuleCreateInfo::builder()
            .code(match description.shader {
                Shader::PlainPnt => vk_shader_macros::include_glsl!("shaders/vk/triangle.frag"),
                Shader::Text => vk_shader_macros::include_glsl!("shaders/vk/text.frag")
            });
        let fragment_shader_module = render_core.device
            .create_shader_module(&fragment_shader_create_info, None)
            .map_err(|e| format!("{:?}", e))?;
        let main_function_name = CString::new("main").unwrap();
        let vertex_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(&main_function_name);
        let fragment_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(&main_function_name);
        let shader_stages = vec![vertex_shader_stage.build(), fragment_shader_stage.build()];

        // Vertex buffer
        let (vbo_vertex_count, vbo_handle) = render_core.query_vbo(description.vbo_index)?;
        let vertex_size_bytes: usize = match description.vbo_format {
            VertexFormat::PositionNormalTexture => 32
        };

        // Vertex input configuration
        let vertex_attrib_descriptions = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                offset: 12,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                offset: 24,
                format: vk::Format::R32G32_SFLOAT
            }
        ];
        let vertex_binding_descriptions = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: vertex_size_bytes as u32,
                input_rate: vk::VertexInputRate::VERTEX
            }
        ];
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attrib_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions);
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        // Create uniform buffer
        let ubo_size_bytes: usize = match description.shader {
            Shader::PlainPnt => 4 * 16,
            Shader::Text => 4 * 20
        };
        let uniform_buffer = {
            let uniform_buffer_data: Vec<f32> = vec![0.0; ubo_size_bytes];
            let allocator = render_core.get_mem_allocator();
            let mut buffer = BufferWrapper::new_uniform_buffer(
                allocator,
                ubo_size_bytes)?;
            buffer.update_from_vec::<f32>(allocator, &uniform_buffer_data)?;
            buffer
        };

        // Texture image
        let texture_image_view = render_core.query_texture(description.texture_index)?;

        // Sampler
        let sampler_info = vk::SamplerCreateInfo::builder()
            .min_filter(vk::Filter::LINEAR)
            .mag_filter(vk::Filter::LINEAR);
        let sampler = render_core.device
            .create_sampler(&sampler_info, None)
            .map_err(|e| format!("Error creating sampler: {:?}", e))?;

        // All the stuff around descriptors
        let ubo_stage_flags = match description.shader {
            Shader::PlainPnt => vk::ShaderStageFlags::VERTEX,
            Shader::Text => vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT
        };
        let descriptor_set_layout_binding_infos = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(ubo_stage_flags)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];
        let descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&descriptor_set_layout_binding_infos);
        let descriptor_set_layout = render_core.device
            .create_descriptor_set_layout(&descriptor_set_layout_info, None)
            .map_err(|e| format!("Error creating descriptor set layout: {:?}", e))?;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: render_core.image_views.len() as u32
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: render_core.image_views.len() as u32
            }
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(render_core.image_views.len() as u32)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = render_core.device
            .create_descriptor_pool(&descriptor_pool_info, None)
            .map_err(|e| format!("Error creating descriptor pool: {:?}", e))?;
        let descriptor_layouts = vec![descriptor_set_layout; render_core.image_views.len()];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_layouts);
        let descriptor_sets = render_core.device
            .allocate_descriptor_sets(&descriptor_set_alloc_info)
            .map_err(|e| format!("Failed allocating descriptor sets: {:?}", e))?;

        // Descriptor bindings
        for (_i, descriptor_set) in descriptor_sets.iter().enumerate() {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffer(),
                offset: 0,
                range: ubo_size_bytes as u64
            }];
            let image_infos = [vk::DescriptorImageInfo {
                image_view: texture_image_view,
                sampler,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            }];
            let descriptor_set_writes = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(*descriptor_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&buffer_infos)
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(*descriptor_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .buffer_info(&buffer_infos)
                    .image_info(&image_infos)
                    .build()
            ];
            render_core.device.update_descriptor_sets(&descriptor_set_writes, &[]);
        }

        // Viewport
        let extent = render_core.get_extent()?;
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent
        }];
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        // Random pipeline configurations
        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::NONE)
            .polygon_mode(vk::PolygonMode::FILL);
        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
        let colour_blend_attachments = [
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::all())
                .build()
        ];
        let colour_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&colour_blend_attachments);
        let pipeline_descriptor_layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipeline_descriptor_layouts);
        let pipeline_layout = render_core.device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .map_err(|e| format!("{:?}", e))?;

        // Make pipeline
        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&colour_blend_info)
            .layout(pipeline_layout)
            .render_pass(renderpass_wrapper.renderpass)
            .subpass(0);
        let graphics_pipeline = render_core.device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info.build()], None)
            .map_err(|e| format!("{:?}", e))?;

        self.vertex_shader_module = vertex_shader_module;
        self.fragment_shader_module = fragment_shader_module;
        self.vertex_buffer = vbo_handle;
        self.vertex_count = vbo_vertex_count;
        self.uniform_buffer = uniform_buffer;
        self.texture_image_view = texture_image_view;
        self.sampler = sampler;
        self.descriptor_set_layout = descriptor_set_layout;
        self.descriptor_pool = descriptor_pool;
        self.descriptor_sets.clear();
        for set in descriptor_sets.iter() {
            self.descriptor_sets.push(*set);
        }
        self.pipeline_layout = pipeline_layout;
        self.pipeline = graphics_pipeline[0];

        Ok(())
    }

    pub unsafe fn record_commands(&self, command_buffer_index: usize, command_buffer: vk::CommandBuffer, render_core: &RenderCore) -> Result<(), String> {
        render_core.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
        render_core.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);
        render_core.device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[self.descriptor_sets[command_buffer_index]], &[]);
        render_core.device.cmd_draw(command_buffer, self.vertex_count as u32, 1, 0, 0);
        Ok(())
    }

    pub unsafe fn update_uniform_buffer(&mut self, render_core: &mut RenderCore, data_ptr: *const u8, size_bytes: usize) -> Result<(), String> {
        self.uniform_buffer.update::<u8>(render_core.get_mem_allocator(), data_ptr, size_bytes)
    }
}
