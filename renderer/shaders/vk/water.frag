#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 o_clip_space;

layout (set = 0, binding = 1) uniform sampler2D reflection_texture_sampler;

layout (location = 0) out vec4 uFragColor;

void main() {
    vec2 ndc = 0.5 * (o_clip_space.xy / o_clip_space.w) + 0.5;
    vec2 reflect_coords = vec2(ndc.x, 1.0 - ndc.y);
    vec4 reflect_color = texture(reflection_texture_sampler, reflect_coords);
    vec4 water_color = vec4(0.5, 0.5, 1.0, 1.0);
    uFragColor = mix(reflect_color, water_color, 0.5);
}