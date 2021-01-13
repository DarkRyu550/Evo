#version 450
#pragma shader_stage(fragment)

layout(set = 0, binding = 0, rgba32f) uniform readonly image2D AlbedoSpecular;
layout(set = 0, binding = 1, rgba32f) uniform readonly image2D NormalShadow;

layout(location = 0) in vec2 Position;
layout(location = 0) out vec4 Color;

vec4 ReadAlbedoSpecular() {
    ivec2 size  = imageSize(AlbedoSpecular);
    vec2  pos   = (Position + 1.0) / 2.0;
    ivec2 coord = ivec2(round(pos * vec2(size)));

    coord.x = clamp(coord.x, 0, size.x - 1);
    coord.y = clamp(coord.y, 0, size.y - 1);

    return imageLoad(AlbedoSpecular, coord);
}

vec4 ReadNormalShadow() {
    ivec2 size  = imageSize(NormalShadow);
    vec2  pos   = (Position + 1.0) / 2.0;
    ivec2 coord = ivec2(round(vec2(
        pos.x * float(size.x),
        pos.y * float(size.y)
    )));

    coord.x = clamp(coord.x, 0, size.x - 1);
    coord.y = clamp(coord.y, 0, size.y - 1);

    return imageLoad(NormalShadow, coord);
}

void main() {
    Color = vec4(ReadAlbedoSpecular().xyz, 1.0);
}
