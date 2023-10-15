attribute vec3 in_position; // [[attribute(0)]]
attribute vec2 in_uv; // [[attribute(1)]];
attribute vec3 in_normal; // [[attribute(2)]];

varying vec2 out_uv; // [[user(locn0)]];
varying vec3 out_pos; // [[user(locn1)]];
varying vec3 out_normal; // [[user(locn2)]];
varying vec4 out_shadow[4]; // [[user(locn3)]];
varying float out_clip_z; // [[user(locn4)]];

uniform mat4 Model;
uniform mat4 ModelInverse;
uniform mat4 Projection;
uniform mat4 ShadowProjection[4];

void main() {
    gl_Position = Projection * Model * vec4(in_position, 1);
    out_clip_z = gl_Position.z;
    out_uv = in_uv;
    out_normal = transpose(mat3(ModelInverse)) * in_normal;
    out_pos = vec3(Model * vec4(in_position, 1.0));
    for (int i = 0; i < 4; i++) {
        out_shadow[i] = ShadowProjection[i] * Model * vec4(in_position, 1.0) * vec4(0.5) + vec4(0.5);
    }

}
