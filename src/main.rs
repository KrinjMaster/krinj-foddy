//game
use std::process::exit;

use bevy::{
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, keyboard_input))
        .run();
}

fn setup(
    windows: Query<&Window>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let shape = Mesh2dHandle(meshes.add(Circle { radius: 10. }));
    let color = Color::hex("#ffffff").unwrap_or_else(|err| {
        println!("{}", err);
        exit(1);
    });

    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(-window.resolution.width() / 2. + 50., 0., 0.),
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
            if transform.translation.y + 20. < window.resolution.height() / 2. {
                transform.translation.y += 5.;
                player_info.direction_y = DirectionY::None;
            };
        }
        DirectionY::Down => {
            if transform.translation.y - 20. > -window.resolution.height() / 2. {
                transform.translation.y -= 5.;
                player_info.direction_y = DirectionY::None;
            }
        }
        DirectionY::None => (),
    }

    match player_info.direction_x {
        DirectionX::Left => {
            if transform.translation.x - 20. > -window.resolution.width() / 2. {
                transform.translation.x -= 5.;
            }
            player_info.direction_x = DirectionX::None;
        }
        DirectionX::Right => {
            if transform.translation.x + 20. < 0. {
                transform.translation.x += 5.;
            }
            player_info.direction_x = DirectionX::None;
        }
        DirectionX::None => (),
    }
}
