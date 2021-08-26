#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    mat4 mvp_matrix;
    float y_bias;
    float y_plane_normal;
    vec2 unused;
} ubo;

layout (location = 0) out vec2 o_tex_coord;

void main() {
    o_tex_coord = tex_coord;
    vec4 position = ubo.mvp_matrix * vec4(pos, 1.0);
    gl_Position = position;
    gl_ClipDistance[0] = ubo.y_bias + ubo.y_plane_normal * position.y;
}