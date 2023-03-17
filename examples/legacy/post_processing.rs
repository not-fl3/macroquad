use macroquad::prelude::*;

#[macroquad::main("Post processing")]
async fn main() {
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);

    let material = load_material(
        ShaderSource {
            glsl_vertex: Some(CRT_VERTEX_SHADER),
            glsl_fragment: Some(CRT_FRAGMENT_SHADER),
            metal_shader: None,
        },
        Default::default(),
    )
    .unwrap();

    loop {
        // drawing to the texture

        // 0..100, 0..100 camera
        set_camera(&Camera2D {
            zoom: vec2(0.01, 0.01),
            target: vec2(0.0, 0.0),
            render_target: Some(render_target.clone()),
            ..Default::default()
        });

        clear_background(BLACK);
        draw_line(-30.0, 45.0, 30.0, 45.0, 3.0, BLUE);
        draw_circle(-45.0, -35.0, 20.0, YELLOW);
        draw_circle(45.0, -35.0, 20.0, GREEN);

        // drawing to the screen

        set_default_camera();

        clear_background(WHITE);
        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        next_frame().await;
    }
}

const CRT_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;

varying vec4 color;
varying vec2 uv;
    
uniform sampler2D Texture;

// https://www.shadertoy.com/view/XtlSD7

vec2 CRTCurveUV(vec2 uv)
{
    uv = uv * 2.0 - 1.0;
    vec2 offset = abs( uv.yx ) / vec2( 6.0, 4.0 );
    uv = uv + uv * offset * offset;
    uv = uv * 0.5 + 0.5;
    return uv;
}

void DrawVignette( inout vec3 color, vec2 uv )
{    
    float vignette = uv.x * uv.y * ( 1.0 - uv.x ) * ( 1.0 - uv.y );
    vignette = clamp( pow( 16.0 * vignette, 0.3 ), 0.0, 1.0 );
    color *= vignette;
}


void DrawScanline( inout vec3 color, vec2 uv )
{
    float iTime = 0.1;
    float scanline 	= clamp( 0.95 + 0.05 * cos( 3.14 * ( uv.y + 0.008 * iTime ) * 240.0 * 1.0 ), 0.0, 1.0 );
    float grille 	= 0.85 + 0.15 * clamp( 1.5 * cos( 3.14 * uv.x * 640.0 * 1.0 ), 0.0, 1.0 );    
    color *= scanline * grille * 1.2;
}

void main() {
    
    vec2 crtUV = CRTCurveUV(uv);
    
    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
 	
    if (crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0)
    {
        res = vec3(0.0, 0.0, 0.0);
    } 
    DrawVignette(res, crtUV);
    DrawScanline(res, uv);
    gl_FragColor = vec4(res, 1.0);

}
"#;

const CRT_VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";
