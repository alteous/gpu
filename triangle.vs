#version 400

layout(location = 0) in vec4 a_local_position;

void main()
{
    gl_Position = a_local_position;
}
