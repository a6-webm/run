use bevy::{prelude::*, window::Cursor};
use bevy_rapier2d::geometry::Group;
use bevy_rapier2d::prelude::*;
use evdev::AbsoluteAxisType;
use evdev::Device;
use evdev::InputEvent;
use evdev::InputEventKind;
use evdev::RelativeAxisType;
use libc::F_SETFL;
use libc::O_NONBLOCK;
use std::env;
use std::f32::consts::PI;
use std::os::fd::AsRawFd;

#[derive(Resource)]
struct Mice {
    l: Device,
    r: Device,
}

struct MouseSpace {
    top: i32,
    bottom: i32,
    right: i32,
    left: i32,
}

#[derive(Resource)]
struct MouseSpaces {
    l: MouseSpace,
    r: MouseSpace,
}

#[derive(Debug)]
struct MousePos {
    x: i32,
    y: i32,
}

#[derive(Resource, Debug)]
struct MicePos {
    l: MousePos,
    r: MousePos,
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let mice = Mice {
        l: Device::open(args[1].clone()).unwrap(),
        r: Device::open(args[2].clone()).unwrap(),
    };
    assert!(unsafe { libc::fcntl(mice.l.as_raw_fd(), F_SETFL, O_NONBLOCK) } == 0);
    assert!(unsafe { libc::fcntl(mice.r.as_raw_fd(), F_SETFL, O_NONBLOCK) } == 0);
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
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1000.0),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(mice)
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(MicePos {
            l: MousePos { x: 0, y: 0 },
            r: MousePos { x: 0, y: 0 },
        })
        .insert_resource(MouseSpaces {
            l: MouseSpace {
                top: 100,
                bottom: 500,
                right: 500,
                left: 100,
            },
            r: MouseSpace {
                top: 100,
                bottom: 500,
                right: 500,
                left: 100,
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
        RigidBody::Fixed,
        Collider::cuboid(25.0, 25.0),
    ));

    let body = commands
        .spawn((
            Body,
            SpriteBundle {
                sprite: square_sprite.clone(),
                transform: Transform::from_xyz(0.0, 100.0, 0.0).with_scale(Vec3 {
                    x: 1.3,
                    y: 1.0,
                    z: 1.0,
                }),
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(25.0, 25.0),
            CollisionGroups::new(
                Group::from_bits(0b01).unwrap(),
                Group::from_bits(0b10).unwrap(),
            ),
        ))
        .id();

    let b_nt_joint = RevoluteJointBuilder::new()
        .local_anchor1(Vec2::new(0.0, -50.0))
        .local_anchor2(Vec2::new(0.0, 50.0))
        .limits([0.0, PI - 0.1]);

    let near_thigh = commands
        .spawn((
            NearThigh,
            SpriteBundle {
                sprite: square_sprite.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(25.0, 25.0),
            CollisionGroups::new(
                Group::from_bits(0b01).unwrap(),
                Group::from_bits(0b10).unwrap(),
            ),
            ImpulseJoint::new(body, b_nt_joint),
        ))
        .id();

    let nt_ns_joint = RevoluteJointBuilder::new()
        .local_anchor1(Vec2::new(0.0, -50.0))
        .local_anchor2(Vec2::new(0.0, 50.0))
        .limits([-180f32.to_radians(), 0.0]);

    let near_shin = commands
        .spawn((
            NearShin,
            SpriteBundle {
                sprite: square_sprite,
                transform: Transform::from_xyz(0.0, -100.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(25.0, 25.0),
            CollisionGroups::new(
                Group::from_bits(0b01).unwrap(),
                Group::from_bits(0b10).unwrap(),
            ),
            ImpulseJoint::new(near_thigh, nt_ns_joint),
        ))
        .id();
}

#[derive(Debug)]
pub enum Lr {
    Left,
    Right,
}

#[derive(Debug)]
pub enum Ax {
    X,
    Y,
}

#[derive(Debug)]
pub struct MouseMoveData {
    pub lr: Lr,
    pub ax: Ax,
    pub v: i32,
}

pub type RelMouseMove = MouseMoveData;
pub type AbsMouseMove = MouseMoveData;

#[derive(Debug)]
pub enum MouseMove {
    Rel(RelMouseMove),
    Abs(AbsMouseMove),
}

fn mice_input(
    mut _commands: Commands,
    mut mice: ResMut<Mice>,
    mut mice_pos: ResMut<MicePos>,
    mouse_spaces: Res<MouseSpaces>,
    mut q_body: Query<&Transform, (With<Body>, Without<NearThigh>, Without<NearShin>)>,
    mut q_near_thigh: Query<
        (&Transform, &mut ImpulseJoint),
        (Without<Body>, With<NearThigh>, Without<NearShin>),
    >,
    mut q_near_shin: Query<
        (&Transform, &mut ImpulseJoint),
        (Without<Body>, Without<NearThigh>, With<NearShin>),
    >,
) {
    let mut moves = Vec::new();
    if let Ok(evs) = mice.l.fetch_events() {
        for ev in evs {
            if let Some(m_move) = event_to_mouse_move(ev, false) {
                moves.push(m_move);
            }
        }
    }
    if let Ok(evs) = mice.r.fetch_events() {
        for ev in evs {
            if let Some(m_move) = event_to_mouse_move(ev, true) {
                moves.push(m_move);
            }
        }
    }
    for m_move in moves {
        match m_move {
            MouseMove::Rel(m_m) => resolve_rel_m_move(&m_m, &mut mice_pos, &mouse_spaces),
            MouseMove::Abs(m_m) => resolve_abs_m_move(&m_m, &mut mice_pos, &mouse_spaces),
        }
    }
    let normal_target = (mice_pos.l.x - mouse_spaces.l.left) as f32
        / (mouse_spaces.l.right - mouse_spaces.l.left) as f32;
    let target_vel = dbg!((normal_target - 0.5) * 2.0 * 100.0);
    let mut imp_joint = q_near_thigh.get_single_mut().unwrap().1;
    let joint = imp_joint.data.as_revolute_mut().unwrap();
    joint.set_motor_velocity(target_vel, 1.0);
}

fn event_to_mouse_move(ev: InputEvent, right: bool) -> Option<MouseMove> {
    let lr = if right { Lr::Right } else { Lr::Left };
    let v = ev.value();
    match ev.kind() {
        InputEventKind::RelAxis(ax_) => {
            let ax = match ax_ {
                RelativeAxisType::REL_X => Ax::X,
                RelativeAxisType::REL_Y => Ax::Y,
                _ => return None,
            };
            Some(MouseMove::Rel(MouseMoveData { lr, ax, v }))
        }
        InputEventKind::AbsAxis(ax_) => {
            let ax = match ax_ {
                AbsoluteAxisType::ABS_X => Ax::X,
                AbsoluteAxisType::ABS_Y => Ax::Y,
                _ => return None,
            };
            Some(MouseMove::Abs(MouseMoveData { lr, ax, v }))
        }
        _ => None,
    }
}

fn resolve_rel_m_move(m_move: &RelMouseMove, mice_pos: &mut MicePos, spaces: &MouseSpaces) {
    let (space, mouse_pos) = match m_move.lr {
        Lr::Left => (&spaces.l, &mut mice_pos.l),
        Lr::Right => (&spaces.r, &mut mice_pos.r),
    };
    let (value_to_change, min_bound, max_bound) = match m_move.ax {
        Ax::X => (&mut mouse_pos.x, space.left, space.right),
        Ax::Y => (&mut mouse_pos.y, space.top, space.bottom),
    };

    *value_to_change = (*value_to_change + m_move.v)
        .max(min_bound)
        .min(max_bound - 1);
}

fn resolve_abs_m_move(m_move: &AbsMouseMove, mice_pos: &mut MicePos, spaces: &MouseSpaces) {
    let (space, mouse_pos) = match m_move.lr {
        Lr::Left => (&spaces.l, &mut mice_pos.l),
        Lr::Right => (&spaces.r, &mut mice_pos.r),
    };
    let (value_to_change, min_bound, max_bound) = match m_move.ax {
        Ax::X => (&mut mouse_pos.x, space.left, space.right),
        Ax::Y => (&mut mouse_pos.y, space.top, space.bottom),
    };

    *value_to_change = m_move.v.max(min_bound).min(max_bound - 1);
}
