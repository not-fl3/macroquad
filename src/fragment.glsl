varying vec2 out_uv;
varying vec3 out_pos;
varying vec3 out_normal;
varying vec4 out_shadow[4];
varying float out_clip_z;

uniform sampler2D Albedo;
uniform sampler2D Emissive;
uniform sampler2D Occlusion;
uniform sampler2D Normal;
uniform sampler2D MetallicRoughness;
uniform sampler2D ShadowMap0;
uniform sampler2D ShadowMap1;
uniform sampler2D ShadowMap2;
uniform sampler2D ShadowMap3;
uniform vec3 CameraPosition;
uniform vec4 Material;
uniform vec4 Color;
uniform vec4 ShadowCascades;
uniform samplerCube Environment;
uniform ivec4 ShadowCasters;

float ShadowMap(int ix, vec2 uv, vec2 offset, float z) {
    float d = 0.0002;
    if (ix == 0) return ((texture2D(ShadowMap0, uv + offset * d).z < z - 0.001) ? 1.0 : 0.0);
    if (ix == 1) return ((texture2D(ShadowMap1, uv + offset * d).z < z - 0.001) ? 1.0 : 0.0);
    if (ix == 2) return ((texture2D(ShadowMap2, uv + offset * d).z < z - 0.001) ? 1.0 : 0.0);
    if (ix == 3) return ((texture2D(ShadowMap3, uv + offset * d).z < z - 0.001) ? 1.0 : 0.0);
}

float ShadowCascade(int ix) {
    if (ix == 0) return ShadowCascades.x;
    if (ix == 1) return ShadowCascades.y;
    if (ix == 2) return ShadowCascades.z;
    if (ix == 3) return ShadowCascades.a;
}

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

    vec4 o = texture2D(Occlusion, out_uv);
    vec4 occlusion = vec4(o.r, o.r, o.r, 1.0);
    vec4 base_color = texture2D(Albedo, out_uv) * Color;

    float specular = 0.0;
    // vec3 light_dir = vec3(1.0, -2.0, -1.0);

    // if (dot(N, light_dir) > 0.0) {
    //     specular = pow(dot(reflect(-light_dir, N), I), 5.0) * 0.05;
    // }

    vec4 environment = textureCubeLod(Environment, R, sm_level((1.0 - metallic) * 5.0));
    vec4 reflection = vec4(roughness) + environment * vec4((1.0 - roughness) + specular * (1.0 - roughness + metallic * 0.2));

    float visibility = 1.0;

    for (int n = 0; n < ShadowCasters.x; n++) {
        for (int i = 0; i < ShadowCasters.y; i++) {
            if (out_clip_z <= ShadowCascade(i)) {
                float s = ShadowMap(i, out_shadow[i].xy, vec2(0.0, 0.0), out_shadow[i].z)
                    + ShadowMap(i, out_shadow[i].xy, vec2(1, 0.0), out_shadow[i].z)
                    + ShadowMap(i, out_shadow[i].xy, vec2(1, 1), out_shadow[i].z)
                    + ShadowMap(i, out_shadow[i].xy, vec2(0.0, 1), out_shadow[i].z)
                    + ShadowMap(i, out_shadow[i].xy, vec2(-1, 1), out_shadow[i].z)
                    + ShadowMap(i, out_shadow[i].xy, vec2(-1, 0.0), out_shadow[i].z)
                    + ShadowMap(i, out_shadow[i].xy, vec2(-1, -1), out_shadow[i].z)
                    + ShadowMap(i, out_shadow[i].xy, vec2(0.0, -1), out_shadow[i].z);
                s /= 8;
                visibility = 1.0 - 0.5 * s;
                break;
            }
        }
    }

    gl_FragColor = (reflection * occlusion * base_color  + texture2D(Emissive, out_uv)) * vec4(visibility, visibility, visibility, 1.);
    //gl_FragColor = (reflection * occlusion * base_color  + texture2D(Emissive, out_uv)) * vec4(visibility, visibility, visibility, 1.);
    //gl_FragColor = metallic;
    //gl_FragColor = vec4(metallic, 0.0, 0.0, 1.0);
}
