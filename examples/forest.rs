use dolly::prelude::*;
use macroquad::{
    prelude::*,
    scene::Scene,
    sprite_layer::{Axis, SpriteLayer},
};

struct FrustumsDbg {
    target: RenderTarget,
    canvas: SpriteLayer,
    canvas_world: SpriteLayer,
}
impl FrustumsDbg {
    fn new(ctx: &macroquad::Context3) -> FrustumsDbg {
        FrustumsDbg {
            target: ctx.render_target(512, 512),
            canvas: ctx.new_canvas(),
            canvas_world: ctx.new_canvas(),
        }
    }

    fn frustum(canvas: &mut SpriteLayer, camera: &Camera) {
        canvas.set_axis(Axis::Y);
        let matrices =
            macroquad::shadowmap::light_view_porijection_matrices(4, &camera, vec3(1.5, 1.5, 1.0));
        for (_, ndc1, _) in &matrices {
            canvas.draw_line(ndc1[2].x, ndc1[2].z, ndc1[6].x, ndc1[6].z, 1.0, RED);
            canvas.draw_line(ndc1[3].x, ndc1[3].z, ndc1[7].x, ndc1[7].z, 1.0, RED);
            canvas.draw_line(ndc1[3].x, ndc1[3].z, ndc1[2].x, ndc1[2].z, 1.0, RED);
            canvas.draw_line(ndc1[7].x, ndc1[7].z, ndc1[6].x, ndc1[6].z, 1.0, RED);
        }
        for (m, _, _) in matrices {
            let m = m.inverse();
            let p0 = m.transform_point3(vec3(-1.0, 0.0, -1.0));
            let p1 = m.transform_point3(vec3(-1.0, 0.0, 1.0));
            let p2 = m.transform_point3(vec3(1.0, 0.0, 1.0));
            let p3 = m.transform_point3(vec3(1.0, 0.0, -1.0));
            canvas.draw_line(p0.x, p0.z, p1.x, p1.z, 0.8, BLACK);
            canvas.draw_line(p1.x, p1.z, p2.x, p2.z, 0.8, BLACK);
            canvas.draw_line(p2.x, p2.z, p3.x, p3.z, 0.8, BLACK);
            canvas.draw_line(p3.x, p3.z, p0.x, p0.z, 0.8, BLACK);
        }
    }
    fn draw(
        &mut self,
        all_models: &[ModelHandle],
        canvas: &mut SpriteLayer,
        scene: &mut Scene,
        camera: &Camera,
    ) {
        canvas.draw_text(
            &format!("fps: {:.0}", 1.0 / get_frame_time()),
            10.0,
            30.0,
            32.0,
            BLACK,
        );
        canvas.draw_text(&format!("draw calls: {:.0}", 666.), 10.0, 60.0, 32.0, BLACK);
        ShapeBuilder::rectangle(vec2(10., 10.), RED)
            .position(vec2(120., 25.))
            .draw(canvas);

        let camera2 = Camera {
            environment: Environment::Solid(RED),
            render_target: Some(self.target.clone()),
            depth_enabled: true,
            projection: Projection::Perspective,
            position: vec3(0., 90.0, 0.1),
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            aspect: Some(1.0),
            z_near: 10.0,
            z_far: 300.0,
            ..Default::default()
        };
        scene.draw(&camera2);

        self.canvas.reset();

        Self::frustum(&mut self.canvas, &camera);

        self.canvas.draw2(&camera2);

        self.canvas_world.reset();
        for model in all_models.iter() {
            let aabb = scene.aabb(model);

            let planes = macroquad::scene::frustum::projection_planes(camera);
            let color = if planes.iter().any(|p| !p.clip(aabb)) {
                RED
            } else {
                GREEN
            };
            let points = [
                vec3(aabb.min.x, aabb.min.y, aabb.min.z),
                vec3(aabb.min.x, aabb.max.y, aabb.min.z),
                vec3(aabb.max.x, aabb.max.y, aabb.min.z),
                vec3(aabb.max.x, aabb.min.y, aabb.min.z),
                vec3(aabb.min.x, aabb.min.y, aabb.max.z),
                vec3(aabb.min.x, aabb.max.y, aabb.max.z),
                vec3(aabb.max.x, aabb.max.y, aabb.max.z),
                vec3(aabb.max.x, aabb.min.y, aabb.max.z),
            ];
            self.canvas_world
                .draw_line_3d(points[0], points[1], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[1], points[2], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[2], points[3], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[3], points[0], 10.0, color);

            self.canvas_world
                .draw_line_3d(points[4], points[5], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[5], points[6], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[6], points[7], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[7], points[4], 10.0, color);

            self.canvas_world
                .draw_line_3d(points[0], points[4], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[1], points[5], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[2], points[6], 10.0, color);
            self.canvas_world
                .draw_line_3d(points[3], points[7], 10.0, color);
        }
        self.canvas_world.draw2(&camera2);

        canvas.draw_texture(self.target.texture.clone(), 70., 50., WHITE);
    }
}

async fn game(ctx: macroquad::Context3) {
    let mut scene = ctx.new_scene();

    let mut dbg = FrustumsDbg::new(&ctx);

    let mut all_models = vec![];
    let helmet = ctx.load_gltf("examples/DamagedHelmet.gltf").await.unwrap();
    let tree = scene.add_model(&helmet);
    scene.set_translation(&tree, vec3(0.0, 10.0, 0.0));
    scene.set_scale(&tree, vec3(10.0, 10.0, 10.0));
    all_models.push(tree.clone());

    let tree = ctx.load_gltf("examples/tree.gltf").await.unwrap();
    {
        let tree = scene.add_model(&tree);
        all_models.push(tree.clone());
        scene.set_translation(&tree, vec3(-40.0, 0.0, -40.0));
    }
    {
        let tree = scene.add_model(&tree);
        all_models.push(tree.clone());
        scene.set_translation(&tree, vec3(40.0, 0.0, 40.0));
        scene.set_scale(&tree, vec3(2.0, 3.0, 2.0));
    }

    for _ in 0..1000 {
        let tree = scene.add_model(&tree);
        all_models.push(tree.clone());

        scene.set_translation(
            &tree,
            vec3(
                rand::gen_range(-50.0, 50.),
                0.0,
                rand::gen_range(-50.0, 50.),
            ),
        );
        scene.set_scale(
            &tree,
            vec3(
                rand::gen_range(0.5, 1.2),
                rand::gen_range(0.5, 1.2),
                rand::gen_range(0.5, 1.2),
            ),
        );
    }

    let plane = scene.mesh(macroquad::models::square());
    let plane = scene.add_model(&plane);
    {
        scene.set_translation(&plane, vec3(0., 0.0, 0.));
        scene.set_scale(&plane, vec3(100.0, 1.0, 100.));
    }

    let skybox = ctx
        .load_cubemap(
            "examples/skybox/skybox_px.png",
            "examples/skybox/skybox_nx.png",
            "examples/skybox/skybox_py.png",
            "examples/skybox/skybox_ny.png",
            "examples/skybox/skybox_pz.png",
            "examples/skybox/skybox_nz.png",
        )
        .await
        .unwrap();

    let mut wtf = Vec3::ZERO;
    let mut zoom = 20.0;
    let mut dolly_rig: CameraRig = CameraRig::builder()
        .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-10.0))
        .with(Smooth::new_rotation(0.7))
        .with(Arm::new(Vec3::Z * zoom))
        .build();

    let mut camera = Camera {
        environment: Environment::Skybox(skybox),
        depth_enabled: true,
        projection: Projection::Perspective,
        position: vec3(0., 1.5, 4.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        fovy: 45.,
        aspect: None,
        z_near: 0.1,
        z_far: 50.0,
        ..Default::default()
    };

    scene.add_shadow_caster(ShadowCaster {
        direction: vec3(1.5, 1.5, 1.0),
        split: ShadowSplit::PSSM4,
    });
    let mut canvas = ctx.new_canvas();

    loop {
        canvas.reset();
        if is_key_down(KeyCode::Up) {
            wtf += Vec3::X * 0.5;
        }
        if is_key_down(KeyCode::Down) {
            wtf -= Vec3::X * 0.5;
        }
        if is_key_down(KeyCode::Left) {
            wtf += Vec3::Z * 0.5;
        }
        if is_key_down(KeyCode::Right) {
            wtf -= Vec3::Z * 0.5;
        }
        if is_mouse_button_down(MouseButton::Left) {
            dolly_rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(mouse_delta().x * 100., mouse_delta().y * 100.);
        }
        if mouse_wheel().1 != 0.0 {
            zoom -= mouse_wheel().1 * 0.8;
            zoom = zoom.clamp(1.8, 100.0);
            dolly_rig.driver_mut::<Arm>().offset = Vec3::Z * zoom;
        }
        let dolly_transform = dolly_rig.update(get_frame_time());
        camera.position = dolly_transform.position + wtf;
        camera.up = dolly_transform.up();
        camera.target = dolly_transform.position + dolly_transform.forward() + wtf;

        scene.draw(&camera);
        scene.draw_shadow_debug();

        dbg.draw(&all_models, &mut canvas, &mut scene, &camera);

        canvas.draw();

        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
