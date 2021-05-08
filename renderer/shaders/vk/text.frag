#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 o_tex_coord;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    mat4 mvp_matrix;
    vec4 paintColor;
} ubo;
layout (set = 0, binding = 1) uniform sampler2D textureSampler;

layout (location = 0) out vec4 uFragColor;

void main() {
    float sampleColor = texture(textureSampler, o_tex_coord).r;
    uFragColor = vec4(ubo.paintColor.rgb, sampleColor * ubo.paintColor.a);
}