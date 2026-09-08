use bevy_asset::{load_embedded_asset, AssetServer, Handle};
use bevy_ecs::prelude::*;
use bevy_mesh::VertexBufferLayout;
use bevy_render::{
    render_resource::{
        binding_types::{sampler, texture_2d, uniform_buffer},
        *,
    },
    view::ViewUniform,
};
use bevy_shader::Shader;
use bevy_utils::default;

#[derive(Resource)]
pub struct UiPipeline {
    pub view_layout: BindGroupLayoutDescriptor,
    pub image_layout: BindGroupLayoutDescriptor,
    pub shader: Handle<Shader>,
}

pub fn init_ui_pipeline(mut commands: Commands, asset_server: Res<AssetServer>) {
    let view_layout = BindGroupLayoutDescriptor::new(
        "ui_view_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::VERTEX_FRAGMENT,
            uniform_buffer::<ViewUniform>(true),
        ),
    );

    let image_layout = BindGroupLayoutDescriptor::new(
        "ui_image_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );

    commands.insert_resource(UiPipeline {
        view_layout,
        image_layout,
        shader: load_embedded_asset!(asset_server.as_ref(), "ui.wgsl"),
    });
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UiPipelineKey {
    pub target_format: TextureFormat,
    pub anti_alias: bool,
}

/// TICKET 22 PATCH -- the UI vertex layout, as `(format, byte offset, shader
/// location)`. Exported so the `offset_of!` test in `lib.rs` checks these exact
/// numbers against the `UiVertex` struct, instead of a second copy of them.
pub(crate) const UI_VERTEX_STRIDE: u64 = 96;
pub(crate) const UI_VERTEX_ATTRS: [(VertexFormat, u64, u32); 8] = [
    (VertexFormat::Float32x3, 48, 0), // position
    (VertexFormat::Float32x2, 64, 1), // uv
    (VertexFormat::Float32x4, 0, 2),  // color
    (VertexFormat::Uint32, 60, 3),    // mode
    (VertexFormat::Float32x4, 16, 4), // border radius
    (VertexFormat::Float32x4, 32, 5), // border thickness
    (VertexFormat::Float32x2, 72, 6), // border size
    (VertexFormat::Float32x2, 80, 7), // position relative to the center
];

impl SpecializedRenderPipeline for UiPipeline {
    type Key = UiPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // TICKET 22 PATCH -- hand-written instead of `from_vertex_formats`.
        //
        // `from_vertex_formats` packs tightly (`offset += format.size()`), which
        // for this attribute list yields color @ 20, radius @ 40, border @ 56 and
        // an 88-byte stride: not one vec4 lands on a 16-byte boundary, and the
        // stride (88 % 16 == 8) shifts the misalignment on every vertex. The
        // sprite pipeline, which renders correctly on the same device, hand-writes
        // its layout with every attribute at a multiple of 16 and an 80-byte
        // stride -- which is the only structural difference between the layer that
        // works and the layer that does not.
        //
        // Shader locations are unchanged, so `ui.wgsl` needs no edit. Only the
        // byte offsets and the stride move, matching the reordered `UiVertex`.
        let vertex_layout = VertexBufferLayout {
            array_stride: UI_VERTEX_STRIDE,
            step_mode: VertexStepMode::Vertex,
            attributes: UI_VERTEX_ATTRS
                .iter()
                .map(|&(format, offset, shader_location)| VertexAttribute {
                    format,
                    offset,
                    shader_location,
                })
                .collect(),
        };
        let shader_defs = if key.anti_alias {
            vec!["ANTI_ALIAS".into()]
        } else {
            Vec::new()
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: shader_defs.clone(),
                buffers: vec![vertex_layout],
                ..default()
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                targets: vec![Some(ColorTargetState {
                    format: key.target_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            layout: vec![self.view_layout.clone(), self.image_layout.clone()],
            label: Some("ui_pipeline".into()),
            ..default()
        }
    }
}
