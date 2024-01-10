use std::env;
use std::sync::mpsc::TryRecvError;
use std::thread;

use bevy::{prelude::*, window::Cursor};
use bevy_xpbd_2d::{math::*, prelude::*};
use evdev::Device;
use run::mouse_thread::mouse_thread;
use run::mouse_thread::AxType;
use run::mouse_thread::Lr;
use run::mouse_thread::MouseMove;

struct MouseStream(std::sync::mpsc::Receiver<MouseMove>);

struct MouseSpace {
    top: i32,
    bottom: i32,
    right: i32,
    left: i32,
}

#[derive(Resource)]
struct MouseSpaces {
    r: MouseSpace,
    l: MouseSpace,
}

#[derive(Resource)]
struct MicePos {
    rx: i32,
    ry: i32,
    lx: i32,
    ly: i32,
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
    let args: Vec<String> = env::args().collect();
    let mouse_stream = create_mouse_stream(&args);
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
        .insert_non_send_resource(mouse_stream)
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(SubstepCount(50))
        .insert_resource(Gravity(Vector::NEG_Y * 1000.0))
        .insert_resource(MicePos {
            rx: 0,
            ry: 0,
            lx: 0,
            ly: 0,
        })
        .insert_resource(MouseSpaces {
            l: MouseSpace {
                top: 0,
                bottom: 100,
                right: 100,
                left: 0,
            },
            r: MouseSpace {
                top: 0,
                bottom: 100,
                right: 100,
                left: 0,
            },
        })
        .add_systems(Startup, setup)
        .add_systems(Update, mice_input)
        .run();
}

fn create_mouse_stream(args: &Vec<String>) -> MouseStream {
    let (tx, rx) = std::sync::mpsc::channel();
    let l_dev = Device::open(args[1].clone()).unwrap();
    let r_dev = Device::open(args[2].clone()).unwrap();
    thread::spawn(move || {
        mouse_thread(tx, l_dev, r_dev).unwrap();
    });
    MouseStream(rx)
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
    mut mice_pos: ResMut<MicePos>,
    mouse_spaces: Res<MouseSpaces>,
    mut q_body: Query<
        (&Transform, &mut ExternalTorque),
        (With<Body>, Without<NearThigh>, Without<NearShin>),
    >,
    mut q_near_thigh: Query<
        (&Transform, &mut ExternalTorque),
        (Without<Body>, With<NearThigh>, Without<NearShin>),
    >,
    mut q_near_shin: Query<
        (&Transform, &mut ExternalTorque),
        (Without<Body>, Without<NearThigh>, With<NearShin>),
    >,
) {
    loop {
        match mouse_stream.0.try_recv() {
            Ok(m) => {
                dbg!(m);
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("damb"),
        }
    }
}

fn resolve_rel_move(m_move: &MouseMove, mice_pos: &mut MicePos, spaces: &MouseSpaces) {
    debug_assert!(m_move.ax_type == AxType::Rel);
    let space = match m_move.lr {
        Lr::Left => &spaces.l,
        Lr::Right => &spaces.r,
    };
}
