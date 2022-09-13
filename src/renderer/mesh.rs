use std::sync::Arc;

use bevy::prelude::Component;
use vulkano::{
    buffer::{BufferUsage, ImmutableBuffer},
    device::Queue, sync::GpuFuture,
};

use crate::data::Vertex;

#[derive(Component)]
pub struct DisplayMesh {
    vertices: Arc<ImmutableBuffer<[Vertex]>>,
    indices: Arc<ImmutableBuffer<[u32]>>,
}

impl DisplayMesh {
    pub fn new<V: IntoIterator<Item = Vertex>, I: IntoIterator<Item = u32>>(
        vertices: V,
        indices: I,
        queue: Arc<Queue>,
    ) -> Self
    where
        V::IntoIter: ExactSizeIterator,
        I::IntoIter: ExactSizeIterator,
    {
        let (vertices, init) =
            ImmutableBuffer::from_iter(vertices, BufferUsage::vertex_buffer(), queue.clone())
                .unwrap();

        init.then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let (indices, init) =
            ImmutableBuffer::from_iter(indices, BufferUsage::index_buffer(), queue).unwrap();

        init.then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        Self { vertices, indices }
    }

    pub const fn indices(&self) -> &Arc<ImmutableBuffer<[u32]>> {
        &self.indices
    }

    pub const fn vertices(&self) -> &Arc<ImmutableBuffer<[Vertex]>> {
        &self.vertices
    }
}
