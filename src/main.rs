use bevy::{prelude::*, window::Cursor};
use bevy_xpbd_2d::{math::*, prelude::*};
use evdev::Device;
use libc::F_SETFL;
use libc::O_NONBLOCK;
use std::env;
use std::os::fd::AsRawFd;

#[derive(Resource)]
struct Mice {
    r: Device,
    l: Device,
}

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
    let mut mice = Mice {
        l: Device::open(args[1].clone()).unwrap(),
        r: Device::open(args[2].clone()).unwrap(),
    };
    assert!(unsafe { libc::fcntl(mice.l.as_raw_fd(), F_SETFL, O_NONBLOCK) } == 0);
    assert!(unsafe { libc::fcntl(mice.l.as_raw_fd(), F_SETFL, O_NONBLOCK) } == 0);
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
        .insert_resource(mice)
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
    mut mice: ResMut<Mice>,
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
    if let Ok(evs) = mice.l.fetch_events() {
        for ev in evs {
            dbg!(ev);
        }
    }
    if let Ok(evs) = mice.l.fetch_events() {
        for ev in evs {
            dbg!(ev);
        }
    }
}

fn resolve_rel_move(m_move: &RelMouseMove, mice_pos: &mut MicePos, spaces: &MouseSpaces) {
    let space = match m_move.lr {
        Lr::Left => &spaces.l,
        Lr::Right => &spaces.r,
    };
}
