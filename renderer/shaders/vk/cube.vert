#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 pos;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    mat4 mvp_matrix;
} ubo;

layout (location = 0) out vec3 o_tex_coord;

void main() {
    o_tex_coord = pos;
    gl_Position = ubo.mvp_matrix * vec4(pos, 1.0);
}