#![allow(clippy::needless_question_mark)]

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/scene.vert",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Pod, Zeroable)]
        }
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/scene.frag",
    }
}
