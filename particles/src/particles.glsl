#ifdef DEF_VERTEX_ATTRIBUTES
attribute vec3 in_attr_pos;
attribute vec2 in_attr_uv;
attribute vec4 in_attr_color;
attribute vec4 in_attr_inst_pos;
attribute vec4 in_attr_inst_uv;
attribute vec4 in_attr_inst_data;
attribute vec4 in_attr_inst_color;
uniform mat4 _mvp;
uniform float _local_coords;
uniform vec3 _emitter_position;

vec4 particle_transform_vertex() {
     vec4 transformed = vec4(0.0, 0.0, 0.0, 0.0);

     if (_local_coords == 0.0) {
        transformed = vec4(in_attr_pos * in_attr_inst_pos.w + in_attr_inst_pos.xyz, 1.0);
     } else {
        transformed = vec4(in_attr_pos * in_attr_inst_pos.w + in_attr_inst_pos.xyz +
                        _emitter_position.xyz, 1.0);
     }

     return _mvp * transformed;
}

vec2 particle_transform_uv() {
    return in_attr_uv * in_attr_inst_uv.zw + in_attr_inst_uv.xy;
}
#endif

highp float rand(lowp vec2 co) {
    highp float a = 12.9898;
    highp float b = 78.233;
    highp float c = 43758.5453;
    highp float dt= dot(co.xy ,vec2(a,b));
    highp float sn= mod(dt,3.14);
    return fract(sin(sn) * c);
}

lowp float particle_ix(lowp vec4 particle_data) {
    return particle_data.x;
}

lowp float particle_lifetime(lowp vec4 particle_data) {
    return particle_data.y;
}
