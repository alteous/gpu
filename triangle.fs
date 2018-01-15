#version 140

layout(std140) uniform UniformBlock {
    vec4 u_Color;
};

uniform sampler2D u_Sampler;

void main()
{
    gl_FragColor = u_Color * texture(u_Sampler, vec2(0.0, 0.0));
}
