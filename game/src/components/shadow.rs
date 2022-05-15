use super::{mesh_drawable::MeshDrawable, movable::Movable};
use crate::{camera::Camera, ecs::world::World};
use core::f32::consts::PI;
use n64::{
    gfx::{
        color_combiner_mode::{ColorCombinerMode, DSrc},
        CommandBuffer, Pipeline,
    },
    VideoMode,
};
use n64_math::{vec3, Mat4};

pub struct Shadow;

static SHADOW_PIPELINE: Pipeline = Pipeline {
    color_combiner_mode: ColorCombinerMode::single(DSrc::Primitive),
    prim_color: Some(0x10101060),
    blend: true,
    z_update: false,
    z_compare: true,
    ..Pipeline::default()
};

pub fn draw(world: &mut World, cb: &mut CommandBuffer, video_mode: VideoMode, camera: &Camera) {
    n64_profiler::scope!("shadow::draw");

    let (mesh_drawable, shadow, movable) = world.components.get3::<MeshDrawable, Shadow, Movable>();

    let half_width = 0.5 * video_mode.width() as f32;
    let half_height = 0.5 * video_mode.height() as f32;

    let post_transform = Mat4::from_cols_array_2d(&[
        [half_width, 0.0, 0.0, 0.0],
        [0.0, half_height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [half_width, half_height, 0.0, 1.0],
    ]);

    let proj = Mat4::perspective_rh_gl(PI / 2.0, 1.0, 0.01, 1000.0);

    let pre_transform = Mat4::from_cols_array_2d(&[
        [2.0, 0.0, 0.0, 0.0],
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, -1.0, 0.0, 1.0],
    ]);

    let proj = post_transform * proj * pre_transform;

    cb.set_pipeline(&SHADOW_PIPELINE);

    for entity in shadow.entities() {
        if let (Some(mesh_drawable), Some(movable)) =
            (mesh_drawable.lookup(*entity), movable.lookup(*entity))
        {
            let transform = proj
                * Mat4::from_rotation_translation(
                    mesh_drawable.rot,
                    vec3(
                        movable.pos.x - camera.pos.x - 0.06,
                        movable.pos.y - camera.pos.y + 0.12,
                        -1.1,
                    ),
                );

            cb.add_mesh_indexed(
                &mesh_drawable.model.verts,
                &mesh_drawable.model.uvs,
                &mesh_drawable.model.colors,
                &mesh_drawable.model.indices,
                &transform.to_cols_array_2d(),
            );
        }
    }
}
