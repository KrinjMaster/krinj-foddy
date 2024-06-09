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
    is_out_of_bounds: bool,
}

#[derive(Component)]
struct Obstacle {
    pos: Vec3,
    collision_count: i8,
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
                projectile_obstacle_collision,
            ),
        )
        .add_systems(
            FixedUpdate,
            (shoot_projectile, projectile_obstacle_collision),
        )
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
            mesh: Mesh2dHandle(meshes.add(Rectangle {
                half_size: Vec2::new(6., 20.),
            })),
            material: materials.add(color),
            transform: Transform::from_xyz(window.resolution.width() / 4., 0., 0.),
            ..default()
        },
        Obstacle {
            pos: Vec3::new(window.resolution.width() / 4., 0., 0.),
            collision_count: 0,
        },
    ));
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
        Projectile {
            is_out_of_bounds: false,
        },
    ));
}

fn projectile_movement(
    mut projectiles: Query<(&mut Transform, &mut Projectile, Entity)>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    let window = windows.single();

    for (mut transform, _, entity) in projectiles.iter_mut() {
        transform.translation.x += 10.;

        if transform.translation.x > window.resolution.width() / 2. {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_obstacle_collision(
    projectiles: Query<(&Transform, &Projectile, Entity)>,
    mut obstacles: Query<(&mut Obstacle, Entity)>,
    mut commands: Commands,
) {
    for (transform, _, entity_proj) in projectiles.iter() {
        for (mut obstacle, entity_obst) in obstacles.iter_mut() {
            let distance = (obstacle.pos - transform.translation).length();

            if distance < 35. {
                commands.entity(entity_proj).despawn();
                obstacle.collision_count += 1;

                if obstacle.collision_count == 4 {
                    commands.entity(entity_obst).despawn();
                }
            }
        }
    }
}
