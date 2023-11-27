use std::pin::Pin;

use bevy::{prelude::*, window::Cursor};
use bevy_xpbd_2d::{math::*, prelude::*};
use evdev::{Device, InputEvent, InputEventKind, RelativeAxisType};
use futures_util::select;
use futures_util::FutureExt;
use futures_util::Stream;
use futures_util::StreamExt;

struct MouseStream(Pin<Box<dyn Stream<Item = (bool, Result<InputEvent, std::io::Error>)>>>);

#[tokio::main]
async fn main() {
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
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_naive_mouse_stream)
        .add_systems(Update, mice_input)
        .run();
}

fn setup_naive_mouse_stream(world: &mut World) {
    let l_dev = Device::open("/dev/input/event9").unwrap();
    let r_dev = Device::open("/dev/input/event16").unwrap();
    let l_stream = l_dev.into_event_stream().unwrap().map(|ev| (false, ev));
    let r_stream = r_dev.into_event_stream().unwrap().map(|ev| (true, ev));
    let events = futures_util::stream::select(l_stream, r_stream);
    world.insert_non_send_resource(MouseStream(Box::pin(events)));
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

    let thigh = commands
        .spawn((
            SpriteBundle {
                sprite: square_sprite.clone(),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(50.0, 50.0),
        ))
        .id();

    let shin = commands
        .spawn((
            SpriteBundle {
                sprite: square_sprite,
                transform: Transform::from_xyz(0.0, -100.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(50.0, 50.0),
        ))
        .id();

    commands.spawn(
        RevoluteJoint::new(thigh, shin)
            .with_local_anchor_1(Vector::new(0.0, -50.0))
            .with_local_anchor_2(Vector::new(0.0, 50.0))
            .with_angle_limits(-1.0, 1.0),
    );
}

fn mice_input(mut _commands: Commands, mut mouse_stream: NonSendMut<MouseStream>) {
    loop {
        let next_ev = select! {
            val = mouse_stream.0.next().fuse() => {
                match val {
                    None => unreachable!("I thiiiiink?"),
                    Some((right, Ok(ev))) => Some((right, ev)),
                    _ => panic!("some io error idk"),
                }
            },
            default => {
                println!("uh");
                None
            },
        };
        let Some((right, ev)) = next_ev else { break };
        dbg!(ev);
        // match (ev.kind(), right) {
        //     (InputEventKind::RelAxis(RelativeAxisType::REL_X), false) => {}
        //     (InputEventKind::RelAxis(RelativeAxisType::REL_Y), false) => {}
        //     (InputEventKind::RelAxis(RelativeAxisType::REL_X), true) => {}
        //     (InputEventKind::RelAxis(RelativeAxisType::REL_Y), true) => {}
        //     _ => (),
        // }
    }
}
