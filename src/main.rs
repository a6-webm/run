use std::env;
use std::sync::mpsc::TryRecvError;
use std::thread;

use bevy::{prelude::*, window::Cursor};
use bevy_xpbd_2d::{math::*, prelude::*};
use evdev::Device;
use run::mouse_thread::mouse_thread;
use run::mouse_thread::MouseMove;

struct MouseStream(std::sync::mpsc::Receiver<MouseMove>);

struct Rect {
    top: i16,
    bottom: i16,
    right: i16,
    left: i16,
}

#[derive(Resource)]
struct MouseSpace {
    r: Rect,
    l: Rect,
}

#[derive(Resource)]
struct MousePos {
    rx: i16,
    ry: i16,
    lx: i16,
    ly: i16,
}

#[derive(Component)]
struct Body;
#[derive(Component)]
struct FarThigh;
#[derive(Component)]
struct FarShin;
#[derive(Component)]
struct NearThigh;
#[derive(Component)]
struct NearShin;

#[derive(PhysicsLayer)]
enum Layer {
    PlayerNear,
    PlayerFar,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    cursor: Cursor {
                        visible: false,
                        grab_mode: bevy::window::CursorGrabMode::Confined,
                        ..Default::default()
                    },
                    title: "Hooh heeh".into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            PhysicsPlugins::default(),
        ))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(SubstepCount(50))
        .insert_resource(Gravity(Vector::NEG_Y * 1000.0))
        .insert_resource(MouseSpace {
            rw: 100,
            rh: 100,
            lw: 100,
            lh: 100,
        })
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_mouse_stream)
        .add_systems(Update, mice_input)
        .run();
}

fn setup_mouse_stream(world: &mut World) {
    let args: Vec<String> = env::args().collect();
    let (tx, rx) = std::sync::mpsc::channel();
    let l_dev = Device::open(args[1].clone()).unwrap();
    let r_dev = Device::open(args[2].clone()).unwrap();
    thread::spawn(move || {
        mouse_thread(tx, l_dev, r_dev).unwrap();
    });
    world.insert_non_send_resource(MouseStream(rx));
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let square_sprite = Sprite {
        color: Color::rgb(0.2, 0.7, 0.9),
        custom_size: Some(Vec2::splat(50.0)),
        ..default()
    };

    let _plane = commands.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_xyz(0.0, -500.0, 0.0).with_scale(Vec3 {
                x: 100.0,
                y: 1.0,
                z: 1.0,
            }),
            ..default()
        },
        RigidBody::Kinematic,
        Collider::cuboid(50.0, 50.0),
    ));

    let body = commands
        .spawn((
            Body,
            SpriteBundle {
                sprite: square_sprite.clone(),
                transform: Transform::from_xyz(0.0, 100.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(50.0, 50.0),
            ExternalTorque::new(0.0).with_persistence(false),
        ))
        .id();

    let close_thigh = commands
        .spawn((
            NearThigh,
            SpriteBundle {
                sprite: square_sprite.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(50.0, 50.0),
            CollisionLayers::new([Layer::PlayerNear], [Layer::PlayerNear]),
            ExternalTorque::new(0.0).with_persistence(false),
        ))
        .id();

    let close_shin = commands
        .spawn((
            NearShin,
            SpriteBundle {
                sprite: square_sprite,
                transform: Transform::from_xyz(0.0, -100.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(50.0, 50.0),
            CollisionLayers::new([Layer::PlayerNear], [Layer::PlayerNear]),
            ExternalTorque::new(0.0).with_persistence(false),
        ))
        .id();

    commands.spawn(
        RevoluteJoint::new(body, close_thigh)
            .with_local_anchor_1(Vector::new(0.0, -50.0))
            .with_local_anchor_2(Vector::new(0.0, 50.0))
            .with_angle_limits(-1.0, 1.0),
    );

    commands.spawn(
        RevoluteJoint::new(close_thigh, close_shin)
            .with_local_anchor_1(Vector::new(0.0, -50.0))
            .with_local_anchor_2(Vector::new(0.0, 50.0))
            .with_angle_limits(-1.0, 1.0),
    );
}

fn mice_input(
    mut _commands: Commands,
    mouse_stream: NonSend<MouseStream>,
    mut mouse_pos: ResMut<MousePos>,
    mouse_space: Res<MouseSpace>,
    mut q_body: Query<(&Transform, &mut ExternalTorque), With<Body>>,
    mut q_near_thigh: Query<(&Transform, &mut ExternalTorque), With<NearThigh>>,
    mut q_near_shin: Query<(&Transform, &mut ExternalTorque), With<NearShin>>,
) {
    loop {
        match mouse_stream.0.try_recv() {
            // TODO
            // Ok(m) => match m {
            //     MouseMove::LeftX(d) => todo!(),
            //     MouseMove::LeftY(d) => todo!(),
            //     MouseMove::RightX(d) => todo!(),
            //     MouseMove::RightY(d) => todo!(),
            // },
            Ok(m) => {
                dbg!(m);
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("damb"),
        }
    }
}
