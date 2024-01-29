use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use rand::prelude::*;
use std::time::Duration;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const WINDOW_MARGIN: f32 = 50.0;

const PLAYER_WIDTH: f32 = 25.0;
const PLAYER_HEIGHT: f32 = 50.0;

const ASTEROID_RADIUS: f32 = 50.0;
const ASTEROID_SPAWN_RATE: f32 = 1.0;
const ASTEROID_MIN_SPEED: f32 = 50.0;
const ASTEROID_MAX_SPEED: f32 = 100.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct Velocity {
    speed: f32,
}

#[derive(Resource)]
struct SpawnTimer {
    timer: Timer,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(ASTEROID_SPAWN_RATE, TimerMode::Once),
        }
    }
}

#[derive(Default, Resource)]
struct Score {
    value: u32,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    InGame,
    GameOver,
}

fn random_position_in_corner() -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(0.0..1.0);
    let y = rng.gen_range(0.0..1.0);

    let map_x = if x < 0.5 {
        x * 2.0 * WINDOW_MARGIN
    } else {
        (x - 0.5) * 2.0 * WINDOW_MARGIN + WINDOW_WIDTH - WINDOW_MARGIN
    } - WINDOW_WIDTH / 2.0;
    let map_y = if y < 0.5 {
        y * 2.0 * WINDOW_MARGIN
    } else {
        (y - 0.5) * 2.0 * WINDOW_MARGIN + WINDOW_HEIGHT - WINDOW_MARGIN
    } - WINDOW_HEIGHT / 2.0;

    Vec3::new(map_x, map_y, 0.0)
}

fn random_asteroid_speed() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED)
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Asteroids".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }),))
        .add_state::<AppState>()
        .init_resource::<SpawnTimer>()
        .init_resource::<Score>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_rotation,
                asteroid_spawn,
                asteroid_movement,
                player_shooting,
                player_collision,
            ).run_if(in_state(AppState::InGame)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Player);
}

fn player_rotation(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut transform_query: Query<&mut Transform, With<Player>>,
) {
    if let Some(position) = window_query.single().cursor_position() {
        let x = position.x - WINDOW_WIDTH / 2.0;
        let y = WINDOW_HEIGHT / 2.0 - position.y;

        for mut transform in transform_query.iter_mut() {
            transform.rotation = Quat::from_rotation_z(x.atan2(-y));
        }
    }
}

fn asteroid_spawn(
    time: Res<Time>,
    mut commands: Commands,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_timer
        .timer
        .tick(Duration::from_secs_f32(time.delta_seconds()));

    if spawn_timer.timer.finished() {
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Circle::new(ASTEROID_RADIUS).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(random_position_in_corner()),
                ..default()
            })
            .insert(Velocity {
                speed: random_asteroid_speed(),
            })
            .insert(Asteroid);

        spawn_timer.timer.reset();
    }
}

fn asteroid_movement(
    time: Res<Time>,
    mut asteroid_query: Query<(&mut Transform, &Velocity), With<Asteroid>>,
) {
    for (mut transform, velocity) in asteroid_query.iter_mut() {
        let direction = -1.0 * transform.translation.normalize();
        let translation = direction * velocity.speed * time.delta_seconds();
        transform.translation += translation;
    }
}

fn player_shooting(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asteroid_query: Query<(Entity, &Transform), With<Asteroid>>,
    buttons: Res<Input<MouseButton>>,
    mut score: ResMut<Score>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(position) = window_query.single().cursor_position() {
        let x = position.x - WINDOW_WIDTH / 2.0;
        let y = WINDOW_HEIGHT / 2.0 - position.y;

        for (entity, transform) in asteroid_query.iter() {
            let dx = transform.translation.x - x;
            let dy = transform.translation.y - y;

            let d = (dx * dx + dy * dy).sqrt();

            if d < ASTEROID_RADIUS {
                score.value += 1;
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_collision(
    asteroid_query: Query<&Transform, With<Asteroid>>,
    mut next_state: ResMut<NextState<AppState>>,
    score: Res<Score>,
) {
    for transform in asteroid_query.iter() {
        let x = transform.translation.x;
        let y = transform.translation.y;

        let d = (x * x + y * y).sqrt();
        if d < ASTEROID_RADIUS {
            next_state.set(AppState::GameOver);
            println!("Game Over! Score: {}", score.value);
        }
    }
}

