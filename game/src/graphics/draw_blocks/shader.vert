#version 450

layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec2 a_TexCoord;
layout(location = 2) in uint a_TexIndex;

layout(location = 0) out vec3 v_Pos;
layout(location = 1) out vec2 v_TexCoord;
layout(location = 2) out uint v_TexIndex;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_CorrProjView;
};

void main() {
    v_Pos = a_Pos;
    v_TexCoord = a_TexCoord;
    v_TexIndex = a_TexIndex;
    gl_Position = u_CorrProjView * vec4(a_Pos, 1.0);
    //gl_Position = vec4(a_Pos.xy, 0.5, 1.0);
}
