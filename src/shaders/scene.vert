#version 430

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec3 normal;

layout(set = 0, binding = 0) uniform ViewProjection_Data {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
} u_vp;

layout(set = 2, binding = 0) uniform Model_Data {
    mat4 model;
} u_model;

layout(location = 0) out vec3 m_normal_ws;
layout(location = 1) out vec2 m_tex_coords;

void main() {
    vec4 pos = vec4(position, 1.0);
    gl_Position = u_vp.projection * u_vp.view * u_model.model * pos;

    m_normal_ws = (u_model.model * vec4(normal, 0.0)).xyz;
    m_tex_coords = tex_coords;
}
