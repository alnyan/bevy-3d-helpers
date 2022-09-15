#version 430

layout(location = 0) out vec4 f_color;

layout(location = 0) in vec3 m_normal_ws;
layout(location = 1) in vec2 m_tex_coords;

layout(set = 0, binding = 0) uniform ViewProjection_Data {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
} u_vp;

layout(set = 1, binding = 0) uniform Material_Data {
    vec4 k_diffuse;
} u_material;
layout(set = 1, binding = 1) uniform sampler2D u_diffuse_map;

const vec3 light_direction = normalize(vec3(-1, -1, -1));

void main() {
    vec3 m_camera_direction = normalize(-u_vp.camera_position);

    vec3 k_diffuse = u_material.k_diffuse.rgb;
    float alpha = u_material.k_diffuse.a;

    k_diffuse *= texture(u_diffuse_map, m_tex_coords).rgb;

    vec3 m_light_reflection_ws = reflect(light_direction, m_normal_ws);

    float cos_theta = clamp(dot(m_normal_ws, -light_direction), 0, 1);
    float cos_alpha = clamp(dot(m_camera_direction, m_light_reflection_ws), 0, 1);

    vec3 c_diffuse = k_diffuse * cos_theta;
    vec3 c_ambient = k_diffuse * 0.1;
    vec3 c_specular = k_diffuse * pow(cos_alpha, 5) * 0.0;

    f_color = vec4(clamp(c_diffuse + c_ambient + c_specular, 0, 1), alpha);
}

