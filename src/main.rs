extern crate nmg_vulkan as engine;
use self::engine::*;
use self::engine::alg::*;
use self::engine::graphics::*;
use self::engine::components::*;

extern crate rand;

default_traits!(App, [engine::FixedUpdate, components::softbody::Iterate]);

#[macro_use]
macro_rules! expand_container {
    ($container: ident: [$($component: ident),* $(,)*]) => {
        $(let $component = &mut $container.$component;)*
    }
}

#[derive(Copy, Clone)]
struct Road {
    angle: f32,
    length: f32,
}

impl Road {
    fn end(self, query: Query) -> Vec2 {
        let angle = self.angle
            * std::f32::consts::PI
            / 180.0;

        let direction = Vec2::new(
            (query.prev_angle + angle).sin(),
            (query.prev_angle + angle).cos(),
        ).norm();

        let end = query.origin
            + direction
            * self.length;

        end
    }
}

#[derive(Copy, Clone)]
struct Query {
    origin: Vec2,
    prev_angle: f32,
}

#[derive(Copy, Clone)]
struct RoadQuery {
    timer: usize,
    lifetime: usize,
    road: Road,
    query: Query,
    valid: bool,
}

impl Ord for RoadQuery{
    fn cmp(&self, other: &RoadQuery)
    -> std::cmp::Ordering {
        // Proper timer ordering
        other.timer.cmp(&self.timer)
    }
}

impl PartialOrd for RoadQuery {
    fn partial_cmp(&self, other: &RoadQuery)
    -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RoadQuery {
    fn eq(&self, other: &RoadQuery) -> bool {
        self.timer == other.timer
    }
}

impl Eq for RoadQuery {}

struct App {
    camera: Option<entity::Handle>,
    last_angle: Vec2,
    fov: f32,

    /* City-gen params */

    q: std::collections::BinaryHeap<RoadQuery>,
    lines: Vec<Line>,
}

impl engine::Start for App {
#[allow(unused_variables)]
fn start(
    &mut self,
    entities: &mut entity::Manager,
    components: &mut components::Container,
) {
    expand_container!(
        components: [
            transforms,
            cameras,
            lights,
            draws,
        ]
    );

    let camera = entities.add();
    transforms.register(camera);
    cameras.register(camera);
    self.camera = Some(camera);
} }

impl engine::Update for App {
#[allow(unused_variables)]
fn update(
    &mut self,
    time: f64,
    delta: f64,
    metadata: Metadata,
    screen: ScreenData,
    parameters: &mut render::Parameters,
    entities: &mut entity::Manager,
    components: &mut components::Container,
    input: &input::Manager,
    debug: &mut debug::Handler,
) {
    expand_container!(
        components: [
            transforms,
            cameras,
            lights,
        ]
    );

    let angle = self.last_angle + input.mouse_delta * 0.01;
    self.last_angle = angle;

    let distance = 16.0;

    let target_position = (
          Vec3::right()
        + Vec3::fwd()
    ).norm();

    let camera_position = (
          Mat3::rotation_y(angle.x as f32)
        * Mat3::rotation_x(angle.y as f32)
        * Vec3::fwd() * -distance
    ) + target_position;

    let camera_orientation = alg::Quat::look_at(
        camera_position,
        target_position,
        Vec3::up(),
    );

    transforms.set(
        self.camera.unwrap(),
        camera_position,
        camera_orientation,
        Vec3::one(),
    );

    if input.key_held(input::Key::Up)   { self.fov -= 2.0; }
    if input.key_held(input::Key::Down) { self.fov += 2.0; }
    cameras.set_fov(self.camera.unwrap(), self.fov);

    /* City-gen line drawing */

    debug.clear_lines();

    let line_count = self.lines.len();
    for (i, line) in self.lines.iter().enumerate() {
        debug.add_line(
            *line, Color::cyan()
                * (
                    1.0 - (i as f32 / line_count as f32)
                    + 0.1
                )
        );
    }

    debug.add_local_axes(
        Vec3::zero(),
        Vec3::fwd(),
        Vec3::up(),
        1.0,
        0.5,
    );
} }

fn main() {
    let app = App {
        camera: None,
        last_angle: Vec2::zero(),
        fov: 60.0,

        /* City-gen params */

        q: std::collections::BinaryHeap
            ::with_capacity(1),
        lines: Vec::with_capacity(1024),
    };

    engine::go(vec![], app);
}
