#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 3) uniform texture2D TileAtlas_texture;
layout(set = 1, binding = 4) uniform sampler TileAtlas_texture_sampler;

void main() {
    o_Target = texture(
        sampler2D(TileAtlas_texture, TileAtlas_texture_sampler),
        v_Uv
    );
}
