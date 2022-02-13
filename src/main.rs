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
const SPEED: f32 = 250.0 * TIME_STEP;

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

        .add_startup_system(setup_camera)
        .add_startup_system(setup_ball)
        .add_startup_system(setup_paddles)

        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(paddle_movement)
                .with_system(ball_movement)
                .with_system(ball_collision),
        )

        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity(Vec3);

fn setup_ball(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.9, 0.9, 0.9),
                custom_size: Some(Vec2::new(BALL_SIZE, BALL_SIZE)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec3::new(SPEED, SPEED, 0.0)))
        .insert(Ball);

    commands
        .spawn()
        .insert(Court { top: SCREEN_HEIGHT * 0.5, bottom: -SCREEN_HEIGHT * 0.5 });
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

fn ball_collision(mut ball_q: Query<(&Transform, &mut Velocity), With<Ball>>, paddle_q: Query<&Transform, With<Player>>) {
    let (ball_transform, mut velocity) = ball_q.single_mut();
    let mut count = 0;
    for transform in paddle_q.iter() {
        count += 1;
        match collide(ball_transform.translation, Vec2::new(BALL_SIZE, BALL_SIZE), transform.translation, Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)) {
            Some(Collision::Left) | Some(Collision::Right) => {
                println!("left/right");
                velocity.0.x *= - 1.0;
            }
            Some(Collision::Top) | Some(Collision::Bottom) => {
                println!("top/bottom");
                velocity.0.y *= - 1.0;
            }
            None => break
        }
    }
    println!("ball_collision: {}", count);
}

#[derive(Component)]
struct Court {
    top: f32,
    bottom: f32
}

#[derive(Component)]
struct Player {
    binds: Binds,
    score: usize
}

struct Binds {
    up: KeyCode,
    down: KeyCode
}

fn setup_paddles(mut commands: Commands) {
    let paddle_size = Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT);

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
        score: 0
    });

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
        score: 0
    });
}

fn paddle_movement(keyboard: Res<Input<KeyCode>>, mut paddle_q: Query<(&mut Transform, &Player)>, court_q: Query<&Court>) {
    let court = court_q.single();
    let mut count = 0;
    for (mut transform, player) in paddle_q.iter_mut() {
        count += 1;
        if keyboard.pressed(player.binds.up) {
            transform.translation.y += SPEED;
        } else if keyboard.pressed(player.binds.down) {
            transform.translation.y -= SPEED;
        }

        transform.translation.y = transform.translation.y.max(court.bottom + PADDLE_HEIGHT * 0.5).min(court.top - PADDLE_HEIGHT * 0.5)
    }
    println!("paddle_movement: {}", count)
}
