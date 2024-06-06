use std::process::exit;

use bevy::{
    core::FrameCount,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

enum DirectionX {
    Left,
    Right,
    None,
}

enum DirectionY {
    Up,
    Down,
    None,
}

#[derive(Component)]
struct Player {
    direction_x: DirectionX,
    direction_y: DirectionY,
}

#[derive(Component)]
struct Projectile {
    has_hit: bool,
}

const SHIP_RADIUS: f32 = 30.;
const FLIGHT_DISTANCE: f32 = 10.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty Invaders".into(),
                name: Some("Rusty Invaders.app".into()),
                resolution: (1920., 1080.).into(),
                focused: true,
                visible: false,
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                keyboard_input,
                make_visible,
                projectile_movement,
            ),
        )
        .add_systems(FixedUpdate, shoot_projectile)
        .insert_resource(Time::<Fixed>::from_seconds(0.75))
        .run();
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 10 {
        window.single_mut().visible = true;
    }
}

fn setup(
    windows: Query<&Window>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let shape = Mesh2dHandle(meshes.add(Circle {
        radius: SHIP_RADIUS,
    }));
    let color = Color::hex("#ffffff").unwrap_or_else(|err| {
        println!("!! Error: {}", err);
        exit(1);
    });

    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::hex("#000000").unwrap_or_else(|err| {
                println!("!! Error: {}", err);
                exit(1);
            })),
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(-window.resolution.width() / 4., 0., 0.),
            ..default()
        },
        Player {
            direction_x: DirectionX::None,
            direction_y: DirectionY::None,
        },
    ));
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut player: Query<&mut Player>) {
    let mut player = player.single_mut();

    if keys.pressed(KeyCode::KeyW) {
        player.direction_y = DirectionY::Up;
    }
    if keys.pressed(KeyCode::KeyS) {
        player.direction_y = DirectionY::Down;
    }
    if keys.pressed(KeyCode::KeyA) {
        player.direction_x = DirectionX::Left;
    }
    if keys.pressed(KeyCode::KeyD) {
        player.direction_x = DirectionX::Right;
    }
}

fn player_movement(mut player: Query<(&mut Transform, &mut Player)>, windows: Query<&Window>) {
    let (mut transform, mut player_info) = player.single_mut();
    let window = windows.single();

    match player_info.direction_y {
        DirectionY::Up => {
            if transform.translation.y + SHIP_RADIUS + 10. < window.resolution.height() / 2. {
                transform.translation.y += FLIGHT_DISTANCE;
                player_info.direction_y = DirectionY::None;
            };
        }
        DirectionY::Down => {
            if transform.translation.y - SHIP_RADIUS - 10. > -window.resolution.height() / 2. {
                transform.translation.y -= FLIGHT_DISTANCE;
                player_info.direction_y = DirectionY::None;
            }
        }
        DirectionY::None => (),
    }

    match player_info.direction_x {
        DirectionX::Left => {
            if transform.translation.x - SHIP_RADIUS - 10. > -window.resolution.width() / 2. {
                transform.translation.x -= FLIGHT_DISTANCE;
            }
            player_info.direction_x = DirectionX::None;
        }
        DirectionX::Right => {
            if transform.translation.x + SHIP_RADIUS + 10. < 0. {
                transform.translation.x += FLIGHT_DISTANCE;
            }
            player_info.direction_x = DirectionX::None;
        }
        DirectionX::None => (),
    }
}

fn shoot_projectile(
    player: Query<(&Transform, &Player)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (transform, _) = player.single();

    let shape = Mesh2dHandle(meshes.add(Rectangle {
        half_size: Vec2::new(12., 4.),
    }));

    let color = Color::hex("#ffffff").unwrap_or_else(|err| {
        println!("!! Error: {}", err);
        exit(1);
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ),
            ..default()
        },
        Projectile { has_hit: false },
    ));
}

fn projectile_movement(mut projectiles: Query<(&mut Transform, &mut Projectile)>) {
    for (mut transform, _) in projectiles.iter_mut() {
        transform.translation.x += 10.;
    }
}
