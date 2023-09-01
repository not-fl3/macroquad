varying vec2 out_uv;
varying vec3 out_pos;
varying vec3 out_normal;

uniform sampler2D Albedo;
uniform sampler2D Emissive;
uniform sampler2D Occlusion;
uniform sampler2D Normal;
uniform sampler2D MetallicRoughness;
uniform vec3 CameraPosition;
uniform vec4 Material;
uniform vec4 Color;
uniform samplerCube Environment;

#if HAS_NORMAL_MAP
vec3 extractNormal(vec2 uv, vec3 pos, vec3 normal, vec3 rgb) {
    vec2 uv_dx = dFdx(uv);
    vec2 uv_dy = dFdy(uv);
    if (length(uv_dx) <= 1e-2) {
        uv_dx = vec2(1.0, 0.0);
    }
    if (length(uv_dy) <= 1e-2) {
        uv_dy = vec2(0.0, 1.0);
    }
    vec3 t_ = (uv_dy.y * dFdx(pos) - uv_dx.y * dFdy(pos)) /
        (uv_dx.x * uv_dy.y - uv_dy.x * uv_dx.y);
    vec3 t, b, ng;
    ng = normalize(normal);
    t = normalize(t_ - ng * dot(ng, t_));
    b = cross(ng, t);
    vec3 res = rgb * 2.0 - vec3(1.0);
    res = normalize(res);
    res = normalize(mat3(t, b, ng) * res);
    return res;
}
#endif

void main() {
    vec3 I = normalize(out_pos - CameraPosition);

#if HAS_NORMAL_MAP
    vec3 N = extractNormal(out_uv, out_pos, out_normal, texture2D(Normal, out_uv).rgb);
    vec3 R = reflect(I, N);
#else
    vec3 N = normalize(out_normal);
    vec3 R = reflect(I, N);
#endif

#if HAS_METALLIC_ROUGHNESS_MAP
    float roughness = texture2D(MetallicRoughness, out_uv).g * Material.y;
#else
    float roughness = Material.y;
#endif

#if HAS_METALLIC_ROUGHNESS_MAP
    float metallic = texture2D(MetallicRoughness, out_uv).b * Material.x;
#else
    float metallic = Material.x;
#endif

    vec4 occlusion = texture2D(Occlusion, out_uv);
    vec4 base_color = texture2D(Albedo, out_uv) * Color;

    vec4 environment = textureCubeLod(Environment, R, sm_level((1.0 - metallic) * 5.0));
    vec4 reflection = vec4(roughness) + environment * vec4(1.0 - roughness);

    gl_FragColor = reflection * occlusion * base_color + texture2D(Emissive, out_uv);
}
