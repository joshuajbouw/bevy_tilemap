#version 300 es

precision highp float;

in vec2 v_Uv;
in vec4 v_Color;

out vec4 o_Target;

uniform sampler2D TextureAtlas_texture;  // set = 1, binding = 2
// layout(std140) uniform sampler TextureAtlas_texture_sampler;  // set = 1, binding = 3

void main() {
    if (v_Color.a == 0.0) {
        discard;
    }
    o_Target = v_Color * texture(
        TextureAtlas_texture,
        v_Uv
    );
}
