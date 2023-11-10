use crate::asset_management::{AssetLoader, ToUuid};
use crate::render_engine::buffer::{GpuUniformBuffer, GpuVertexBufferLayout};
use crate::render_engine::camera::CameraUniform;
use crate::render_engine::texture::GpuTexture;
use crate::render_engine::vertex::Vertex2;
use cgmath::Matrix4;
use wgpu::*;

pub struct SpriteRenderPipeline;

impl ToUuid for SpriteRenderPipeline {}

impl super::RenderPipelineInit for SpriteRenderPipeline {
    fn init(&self, device: &Device, format: TextureFormat) -> anyhow::Result<RenderPipeline> {
        Ok(init(device, format))
    }
}

pub fn init(device: &Device, format: TextureFormat) -> RenderPipeline {
    let raw_shader_source = AssetLoader::get_asset("sprite.wgsl").unwrap();
    let shader_source = String::from_utf8_lossy(&raw_shader_source);

    let shader = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("Sprite SM"),
        source: ShaderSource::Wgsl(shader_source),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Sprite RPL"),
        bind_group_layouts: &[
            &GpuUniformBuffer::<CameraUniform>::bind_group_static(
                &device,
                Some("Sprite RPL Camera BGL"),
            ),
            &GpuTexture::build_bind_group_layout(&device, "Sprite RPL Texture BGL"),
        ],
        push_constant_ranges: &[
            PushConstantRange {
                // 0..64 model matrix
                // 64..68 z-depth
                stages: ShaderStages::VERTEX,
                range: 0..68, // has to align to 4
            },
            //PushConstantRange {
            //    // model matrix
            //    stages: ShaderStages::VERTEX,
            //    range: 0..64,
            //},
            //PushConstantRange {
            //    // z-depth
            //    stages: ShaderStages::VERTEX,
            //    range: 64..65,
            //},
        ],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Sprite RP"),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex2::layout().to_owned()],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[ColorTargetState {
                format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            }],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Cw,
            // we're drawing sprites, which are rectangles with a texture, so no culling is needed
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub struct SpritePushConstant {
    model: Matrix4<f32>,
    z_layer: f32,
}

impl SpritePushConstant {
    pub fn new(model: Matrix4<f32>, z_layer: f32) -> Self {
        Self { model, z_layer }
    }

    pub fn as_bytes(&self) -> [u8; 68] {
        let mut bytes = [0u8; 68];
        let model_bytes: [u8; 64] =
            unsafe { std::mem::transmute(self.model * cgmath::Matrix4::from_scale(200.0)) };
        bytes[0..64].copy_from_slice(&model_bytes);
        bytes[64..68].copy_from_slice(&self.z_layer.to_ne_bytes());
        bytes
    }
}
