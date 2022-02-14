use bevy::{
    core::FixedTimestep,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

const SCREEN_WIDTH: f32 = 1600.0;
const SCREEN_HEIGHT: f32 = 900.0;
const BALL_SIZE: f32 = SCREEN_WIDTH * 0.03;
const PADDLE_WIDTH: f32 = BALL_SIZE * 0.75;
const PADDLE_HEIGHT: f32 = SCREEN_HEIGHT * 0.24;
const TIME_STEP: f32 = 1.0 / 60.0;
const BALL_SPEED: f32 = 250.0 * TIME_STEP;
const PADDLE_SPEED: f32 = BALL_SPEED * 1.66;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(WindowDescriptor {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)

        .add_startup_system(setup)

        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(paddle_movement)
                .with_system(ball_movement)
                .with_system(ball_collision),
        )

        .run();
}

#[derive(Component)]
struct Court {
    top: f32,
    bottom: f32
}

#[derive(Component)]
struct Player {
    binds: Binds,
    speed: f32,
    score: usize
}

struct Binds {
    up: KeyCode,
    down: KeyCode
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity(Vec3);

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // court
    commands
        .spawn()
        .insert(Court { top: SCREEN_HEIGHT * 0.5, bottom: -SCREEN_HEIGHT * 0.5 });

    // ball
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.9, 0.9, 0.9),
                custom_size: Some(Vec2::new(BALL_SIZE, BALL_SIZE)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec3::new(BALL_SPEED, BALL_SPEED, 0.0)))
        .insert(Ball);

    // paddles
    let paddle_size = Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT);

    // player 1
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.9, 0.9, 0.9),
            custom_size: Some(paddle_size),
            ..Default::default()
        },
        transform: Transform::from_xyz(-SCREEN_WIDTH * 0.5 + PADDLE_WIDTH * 2.0, 0.0, 0.0),
        ..Default::default()
    }).insert(Player {
        binds: Binds { up: KeyCode::W, down: KeyCode::S },
        speed: PADDLE_SPEED,
        score: 0,
    });

    // player 2
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.9, 0.9, 0.9),
            custom_size: Some(paddle_size),
            ..Default::default()
        },
        transform: Transform::from_xyz(SCREEN_WIDTH * 0.5 - PADDLE_WIDTH * 2.0, 0.0, 0.0),
        ..Default::default()
    }).insert(Player {
        binds: Binds { up: KeyCode::Up, down: KeyCode::Down },
        speed: PADDLE_SPEED,
        score: 0,
    });
}

fn ball_movement(mut ball_q: Query<(&mut Transform, &mut Velocity), With<Ball>>, court_q: Query<&Court>) {
    let (mut transform, mut velocity) = ball_q.single_mut();

    let translation = &mut transform.translation;
    *translation += velocity.0;
    let radius = BALL_SIZE * 0.5;

    let court = court_q.single();
    if translation.y - radius <= court.bottom {
        translation.y = court.bottom + radius;
        velocity.0.y *= -1.0;
    } else if translation.y + radius >= court.top {
        translation.y = court.top - radius;
        velocity.0.y *= -1.0;
    }

    if translation.x >= SCREEN_WIDTH * 0.5 - radius {
       velocity.0.x *= -1.0;
    } else if translation.x <= -SCREEN_WIDTH * 0.5 + radius {
        velocity.0.x *= -1.0;
    }
}

fn ball_collision(mut ball_q: Query<(&mut Transform, &mut Velocity), With<Ball>>, paddle_q: Query<&Transform, (With<Player>, Without<Ball>)>) {
    let (mut ball_transform, mut velocity) = ball_q.single_mut();
    for transform in paddle_q.iter() {
        let horizontal_adjust = (BALL_SIZE * 0.5) + (PADDLE_WIDTH * 0.5);
        let vertical_adjust = (BALL_SIZE * 0.5) + (PADDLE_HEIGHT * 0.5);
        while let Some(collision) = collide(ball_transform.translation, Vec2::new(BALL_SIZE, BALL_SIZE), transform.translation, Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)) {
            match collision {
                Collision::Left => {
                    velocity.0.x *= -1.0;
                    ball_transform.translation.x = transform.translation.x - horizontal_adjust;
                }
                Collision::Right => {
                    velocity.0.x *= -1.0;
                    ball_transform.translation.x = transform.translation.x + horizontal_adjust;
                }
                Collision::Top => {
                    velocity.0.y *= -1.0;
                    ball_transform.translation.y = transform.translation.y + vertical_adjust;
                }
                Collision::Bottom => {
                    velocity.0.y *= -1.0;
                    ball_transform.translation.y = transform.translation.y - vertical_adjust;
                }
            }
        }
    }
}

fn paddle_movement(keyboard: Res<Input<KeyCode>>, mut paddle_q: Query<(&mut Transform, &Player)>, court_q: Query<&Court>) {
    let court = court_q.single();
    for (mut transform, player) in paddle_q.iter_mut() {
        if keyboard.pressed(player.binds.up) {
            transform.translation.y += player.speed;
        } else if keyboard.pressed(player.binds.down) {
            transform.translation.y -= player.speed;
        }

        transform.translation.y = transform.translation.y.max(court.bottom + PADDLE_HEIGHT * 0.5).min(court.top - PADDLE_HEIGHT * 0.5)
    }
}
