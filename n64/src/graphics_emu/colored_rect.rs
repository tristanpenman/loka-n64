use crate::graphics_emu::Vertex;
use std::mem;
use zerocopy::{AsBytes, FromBytes};

pub const MAX_COLORED_RECTS: u64 = 4096;

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub(crate) struct ColoredRectUniforms {
    pub color: [f32; 4],
    pub offset: [f32; 2],
    pub scale: [f32; 2],
}

pub(crate) struct ColoredRect {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: wgpu::ShaderModule,
    pub pipeline: wgpu::RenderPipeline,
    pub shader_storage_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl ColoredRect {
    pub(crate) fn new(device: &wgpu::Device, dst_tex_format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::StorageBuffer { dynamic: false, readonly: true },
            }],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let vs_bytes = wgpu::read_spirv(
            glsl_to_spirv::compile(
                include_str!("shaders/colored_rect.vert"),
                glsl_to_spirv::ShaderType::Vertex,
            )
            .map_err(|e| { println!("{}", e); "Unable to compile shaders/colored_rect.vert" }).unwrap(),
        )
        .map_err(|e| format!("{}", e)).unwrap();

        let fs_bytes = wgpu::read_spirv(
            glsl_to_spirv::compile(
                include_str!("shaders/colored_rect.frag"),
                glsl_to_spirv::ShaderType::Fragment,
            )
            .map_err(|e| { println!("{}", e); "Unable to compile shaders/colored_rect.frag" }).unwrap(),
        )
        .unwrap();

        let vs_module = device.create_shader_module(&vs_bytes);
        let fs_module = device.create_shader_module(&fs_bytes);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: dst_tex_format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let shader_storage_buffer =device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: MAX_COLORED_RECTS * mem::size_of::<ColoredRectUniforms>() as u64,
            usage: wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group =device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &shader_storage_buffer,
                    range: 0..(MAX_COLORED_RECTS * mem::size_of::<ColoredRectUniforms>() as u64),
                },
            }],
            label: None,
        });

        Self {
            bind_group_layout,
            pipeline_layout,
            vs_module,
            fs_module,
            pipeline,
            shader_storage_buffer,
            bind_group,
        }
    }
}
