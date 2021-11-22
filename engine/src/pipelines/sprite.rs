use crate::asset_management::{AssetLoader, ToUuid};
use crate::buffer::{GpuUniformBuffer, GpuVertexBufferLayout, Uniform};
use crate::camera::CameraUniform;
use crate::texture::GpuTexture;
use crate::vertex::Vertex3;
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
        push_constant_ranges: &[PushConstantRange {
            // model matrix
            stages: ShaderStages::VERTEX,
            range: 0..64,
        }],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Sprite RP"),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "main",
            buffers: &[Vertex3::layout().to_owned()],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "main",
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
            clamp_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    })
}
