#version 450

layout(location = 0) in vec3 v_Pos;
layout(location = 1) in vec2 v_TexCoord;
layout(location = 2) flat in uint v_TexIndex;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 1) uniform texture2DArray u_BlockTextureArray;
layout(set = 0, binding = 2) uniform sampler u_BlockSamplerArray;

void main() {
    o_Target = texture(
        sampler2DArray(u_BlockTextureArray, u_BlockSamplerArray),
        vec3(v_TexCoord, v_TexIndex)
    );
}