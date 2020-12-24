#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in uint Vertex_Tile_Index;

layout(location = 0) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform TileAtlas_size {
    vec2 AtlasSize;
};

struct Rect {
    // Upper-left coordinate
    vec2 begin;
    // Bottom-right coordinate
    vec2 end;
};

layout(set = 1, binding = 1) buffer TileAtlas_textures {
    Rect[] textures;
};

struct Tile {
    uint sprite_index;
    bool is_horizontally_flipped;
};

layout(set = 1, binding = 2) buffer TileAtlas_tiles {
    Tile[] tiles;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};


void main() {
    Tile tile = tiles[Vertex_Tile_Index];
    Rect sprite_rect = textures[tile.sprite_index];

    vec2 sprite_dimensions = sprite_rect.end - sprite_rect.begin;
    vec3 vertex_position = vec3(
        Vertex_Position.xy * sprite_dimensions,
        0.0
    );

    vec2 atlas_positions[4] = vec2[](
        sprite_rect.begin,
        vec2(sprite_rect.begin.x, sprite_rect.end.y),
        sprite_rect.end,
        vec2(sprite_rect.end.x, sprite_rect.begin.y)
    );

    vec2 midpoint = vec2(
        (sprite_rect.begin.x + sprite_rect.end.x) / 2.0,
        (sprite_rect.begin.y + sprite_rect.end.y) / 2.0
    );

    for (int i = 0; i < 4; i++) {
        atlas_positions[i] -= midpoint;
        if (tile.is_horizontally_flipped) {
            atlas_positions[i] *= vec2(-1.0, 1.0);
        }
        atlas_positions[i] += midpoint;
    } 

    v_Uv = atlas_positions[gl_VertexIndex % 4] / AtlasSize;
    gl_Position = ViewProj * Model * vec4(ceil(vertex_position), 1.0);  
}
