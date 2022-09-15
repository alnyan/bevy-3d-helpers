use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position, tex_coords, normal);
