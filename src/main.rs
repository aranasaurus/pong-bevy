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

        .add_system(score)

        .run();
}

#[derive(Component)]
struct Court {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32
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

#[derive(Component)]
struct Size {
    width: f32,
    height: f32
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // court
    commands
        .spawn()
        .insert(Court {
            top: SCREEN_HEIGHT * 0.5,
            bottom: -SCREEN_HEIGHT * 0.5,
            left: -SCREEN_WIDTH * 0.5,
            right: SCREEN_WIDTH * 0.5,
        });

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
        .insert(Size { width: BALL_SIZE, height: BALL_SIZE })
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
    }).insert(Size { width: PADDLE_WIDTH, height: PADDLE_HEIGHT });

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
    }).insert(Size { width: PADDLE_WIDTH, height: PADDLE_HEIGHT });
}

fn ball_movement(mut ball_q: Query<(&mut Transform, &mut Velocity, &Size), With<Ball>>, court_q: Query<&Court>) {
    let (mut transform, mut velocity, size) = ball_q.single_mut();

    let translation = &mut transform.translation;
    *translation += velocity.0;
    let radius = size.width * 0.5;

    let court = court_q.single();
    if translation.y - radius <= court.bottom {
        translation.y = court.bottom + radius;
        velocity.0.y *= -1.0;
    } else if translation.y + radius >= court.top {
        translation.y = court.top - radius;
        velocity.0.y *= -1.0;
    }
}

fn score(mut ball_q: Query<(&Transform, &mut Velocity, &Size), With<Ball>>, mut player_q: Query<(&mut Player, &Transform, &Size)>, court_q: Query<&Court>) {
    let mut ball = ball_q.single_mut();
    let ball_x = ball.0.translation.x;
    let ball_velocity = &mut ball.1.0;
    let court = court_q.single();

    let scored_right = ball_x - ball.2.width > court.right;
    let scored_left = ball_x + ball.2.width < court.left;

    if !scored_left && !scored_right {
        return
    }

    for (mut player, transform, size) in player_q.iter_mut() {
        if (transform.translation.x < 0.0) != (ball.0.translation.x < 0.0) {
            continue
        }

        player.score += 1;
        println!("player scored {}", player.score);
    }

    ball_velocity.x = -ball_velocity.x;

    // TODO: Add events / game states and reset the ball and all that jive. Also show the score. Not here, here should just set the game state or trigger the event or action or whatever approach I end up taking.
}

fn ball_collision(mut ball_q: Query<(&mut Transform, &mut Velocity, &Size), With<Ball>>, paddle_q: Query<(&Transform, &Size), (With<Player>, Without<Ball>)>) {
    let (mut ball_transform, mut velocity, ball_size) = ball_q.single_mut();
    for (paddle_transform, paddle_size) in paddle_q.iter() {
        let horizontal_adjust = (ball_size.width * 0.5) + (paddle_size.width * 0.5);
        let vertical_adjust = (ball_size.height * 0.5) + (paddle_size.height * 0.5);
        if let Some(collision) = collide(ball_transform.translation, Vec2::new(BALL_SIZE, BALL_SIZE), paddle_transform.translation, Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)) {
            match collision {
                Collision::Left => {
                    velocity.0.x *= -1.0;
                    ball_transform.translation.x = paddle_transform.translation.x - horizontal_adjust;
                }
                Collision::Right => {
                    velocity.0.x *= -1.0;
                    ball_transform.translation.x = paddle_transform.translation.x + horizontal_adjust;
                }
                Collision::Top => {
                    velocity.0.y = velocity.0.y.abs();
                    ball_transform.translation.y = paddle_transform.translation.y + vertical_adjust;
                }
                Collision::Bottom => {
                    velocity.0.y = -velocity.0.y.abs();
                    ball_transform.translation.y = paddle_transform.translation.y - vertical_adjust;
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
