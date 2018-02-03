#version 400

layout(location = 0) in vec4 a_Position;
layout(location = 1) in vec4 a_Normal;

out vec3 v_Normal;

void main()
{
    v_Normal = a_Normal.xyz;
    gl_Position = a_Position;
}
