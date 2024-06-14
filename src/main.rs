use rand::{thread_rng, Rng};
use std::{process::exit, time::Duration};

use bevy::{
    core::FrameCount,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
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
    pos: Vec3,
    direction_x: DirectionX,
    direction_y: DirectionY,
}

#[derive(Component)]
struct Projectile {
    pos: Vec3,
}

#[derive(Component)]
struct Rocket {
    pos: Vec3,
}

#[derive(Component)]
struct HealthBar {
    max_health: f32,
    current_health: f32,
}

const SHIP_RADIUS: f32 = 70.;
const FLIGHT_DISTANCE: f32 = 10.;
const HP_BAR_FULL_WIDTH: f32 = 20.;
const PLAYER_MAX_HP: f32 = 8.;
const NPC_MAX_HP: f32 = 3.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
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
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                keyboard_input,
                update_rocket_pos,
                update_player_pos,
                update_projectile_pos,
                rocket_movement,
                make_visible,
                projectile_movement,
                collision_system,
                rocket_player_collision_system,
                spawn_rocket.run_if(on_timer(Duration::from_secs(2))),
                shoot_projectile.run_if(on_timer(Duration::from_millis(500))),
            ),
        )
        .run();
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 10 {
        window.single_mut().visible = true;
    }
}

fn setup(
    windows: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let transform_player = Transform {
        translation: Vec3::new(-window.resolution.width() / 4., 0., 0.),
        scale: Vec3::new(6.5, 6.5, 6.5),
        ..Default::default()
    };
    let transform_hp_bar = Transform {
        translation: Vec3::new(0., -10., 0.),
        scale: Vec3::new(0.15, 0.15, 0.15),
        ..Default::default()
    };
    let color_hp_bar = Color::hex("#FF0000").unwrap_or_else(|err| {
        println!("!! Error: {}", err);
        exit(1);
    });

    // camera
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

    // player
    commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("Ship.png"),
                transform: transform_player,
                ..default()
            },
            Player {
                pos: Vec3::new(-window.resolution.width() / 4., 0., 0.),
                direction_x: DirectionX::None,
                direction_y: DirectionY::None,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle {
                        half_size: Vec2::new(HP_BAR_FULL_WIDTH * 2., 4.),
                    })),
                    material: materials.add(color_hp_bar),
                    transform: transform_hp_bar,
                    ..default()
                },
                HealthBar {
                    max_health: PLAYER_MAX_HP,
                    current_health: PLAYER_MAX_HP,
                },
            ));
        });
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut player: Query<&mut Player>) {
    for mut player in player.iter_mut() {
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
}

fn player_movement(mut player: Query<(&mut Transform, &mut Player)>, windows: Query<&Window>) {
    let window = windows.single();
    for (mut transform, mut player_info) in player.iter_mut() {
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
}

fn shoot_projectile(
    player: Query<(&Transform, &Player)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (transform, _) in player.iter() {
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
                pos: Vec3::new(
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ),
            },
        ));
    }
}

fn update_rocket_pos(mut rockets: Query<(&mut Transform, &mut Rocket)>) {
    for (transform, mut rocket) in rockets.iter_mut() {
        rocket.pos = transform.translation;
    }
}

fn update_player_pos(mut player: Query<(&mut Transform, &mut Player)>) {
    for (transform, mut player) in player.iter_mut() {
        player.pos = transform.translation;
    }
}

fn update_projectile_pos(mut projectile: Query<(&mut Transform, &mut Projectile)>) {
    for (transform, mut projectile) in projectile.iter_mut() {
        projectile.pos = transform.translation;
    }
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

fn collision_system(
    projectiles: Query<(&Projectile, Entity)>,
    mut rockets: Query<(&mut Rocket, Entity, &mut Children)>,
    mut hp_bars: Query<(&mut HealthBar, Entity, &mut Transform)>,
    mut commands: Commands,
) {
    for (projectile, entity_proj) in projectiles.iter() {
        for (rocket, entity_obst, children) in rockets.iter_mut() {
            let distance = (rocket.pos - projectile.pos).length();

            if distance < 35. {
                for (mut hp_bar, hp_entity, mut transform) in hp_bars.iter_mut() {
                    if children.get(0) == Some(&hp_entity) {
                        commands.entity(entity_proj).despawn();

                        hp_bar.current_health -= 1.;

                        transform.scale.x = 0.15 * hp_bar.current_health / hp_bar.max_health;

                        if hp_bar.current_health == 0. {
                            commands.entity(entity_obst).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}

fn spawn_rocket(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = thread_rng();
    let window = windows.single();

    let min: f32 = -0.75;
    let max: f32 = 0.75;

    let rocket_pos_y = rng.gen_range(min..max) * window.resolution.height() / 2.;

    let color_hp_bar = Color::hex("#FF0000").unwrap_or_else(|err| {
        println!("!! Error: {}", err);
        exit(1);
    });
    let transform_rocket = Transform {
        translation: Vec3::new(window.resolution.width() / 2. + 100., rocket_pos_y, 0.),
        scale: Vec3::new(6.5, 6.5, 6.5),
        ..Default::default()
    };
    let transform_hp_bar = Transform {
        translation: Vec3::new(0., -10., 0.),
        scale: Vec3::new(0.15, 0.15, 0.15),
        ..Default::default()
    };

    commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("Rocket.png"),
                transform: transform_rocket,
                ..default()
            },
            Rocket {
                pos: Vec3::new(window.resolution.width() / 4., 0., 0.),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle {
                        half_size: Vec2::new(HP_BAR_FULL_WIDTH, 4.),
                    })),
                    material: materials.add(color_hp_bar),
                    transform: transform_hp_bar,
                    ..default()
                },
                HealthBar {
                    max_health: NPC_MAX_HP,
                    current_health: NPC_MAX_HP,
                },
            ));
        });
}

fn rocket_movement(
    mut rockets: Query<(&mut Transform, &mut Rocket, Entity)>,
    mut commands: Commands,
    windows: Query<&Window>,
) {
    let window = windows.single();

    for (mut transform, _, entity) in rockets.iter_mut() {
        transform.translation.x -= 5.;

        if transform.translation.x < -window.resolution.width() / 2. {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn rocket_player_collision_system(
    rockets: Query<(&mut Rocket, Entity)>,
    mut commands: Commands,
    mut player: Query<(&mut Player, Entity, &mut Children)>,
    mut hp_bars: Query<(&mut HealthBar, Entity, &mut Transform)>,
) {
    for (player, entity_pl, children) in player.iter_mut() {
        for (rocket, entity_roc) in rockets.iter() {
            let distance = (rocket.pos - player.pos).length();

            if distance < 100. {
                for (mut hp_bar, hp_entity, mut transform) in hp_bars.iter_mut() {
                    if children.get(0) == Some(&hp_entity) {
                        commands.entity(entity_roc).despawn_recursive();

                        hp_bar.current_health -= 1.;

                        transform.scale.x = 0.15 * hp_bar.current_health / hp_bar.max_health;

                        if hp_bar.current_health == 0. {
                            commands.entity(entity_pl).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}
