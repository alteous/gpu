#version 400

in vec3 v_Normal;

layout(location = 0) out vec3 g_Normal;

void main()
{
    g_Normal = normalize(v_Normal);
}
