#version 430

layout(location = 0) out vec4 f_color;

layout(location = 0) in vec3 m_normal_ws;

layout(set = 0, binding = 0) uniform ViewProjection_Data {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
} u_vp;

const vec3 light_direction = normalize(vec3(-1, -1, -1));

void main() {
    vec3 m_camera_direction = normalize(-u_vp.camera_position);

    vec3 k_diffuse = vec3(1.0);

    vec3 m_light_reflection_ws = reflect(light_direction, m_normal_ws);

    float cos_theta = clamp(dot(m_normal_ws, -light_direction), 0, 1);
    float cos_alpha = clamp(dot(m_camera_direction, m_light_reflection_ws), 0, 1);

    vec3 c_diffuse = k_diffuse * cos_theta;
    vec3 c_ambient = k_diffuse * 0.1;
    vec3 c_specular = k_diffuse * pow(cos_alpha, 5);

    f_color = vec4(clamp(c_diffuse + c_ambient + c_specular, 0, 1), 1.0);
}

