use bevy::prelude::*;
use rand::random;

use std::collections::HashSet;
use std::time::Duration;

const SIZE: f32 = 10.0;
const GAP: f32 = 4.0;
const HALF_LEN: i32 = 40;
const INIT_ALIVE_COUNT: usize = (HALF_LEN * HALF_LEN) as usize;
const TICK: Duration = Duration::from_millis(100);

type Coord = (i32, i32);

#[derive(Component, Debug, Clone)]
struct Cell {
    state: State,
    index_xy: Coord,
}

#[derive(Debug, PartialEq, Clone)]
enum State {
    Dead,
    Alive,
}

#[derive(Resource, Default, Debug)]
struct Dashboard {
    round: usize,
    survival: usize,
}

fn seed() -> HashSet<Coord> {
    let mut result = HashSet::new();

    while result.len() < INIT_ALIVE_COUNT {
        // not on edge
        let x = random::<i32>() % HALF_LEN;
        let y = random::<i32>() % HALF_LEN;
        result.insert((x, y));
    }

    result
}

fn setup(mut commands: Commands) {
    // cells
    let mut cells_with_mm = vec![];
    let rand_alives = seed();

    for x in -HALF_LEN..HALF_LEN {
        for y in -HALF_LEN..HALF_LEN {
            let cell = Cell {
                state: if rand_alives.contains(&(x, y)) {
                    State::Alive
                } else {
                    State::Dead
                },
                index_xy: (x, y),
            };

            let pos = Vec3::new(x as f32 * (SIZE + GAP), y as f32 * (SIZE + GAP), 0.0);
            // dbg!(&pos);
            cells_with_mm.push((
                cell,
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2 { x: SIZE, y: SIZE }),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(pos),
                    ..Default::default()
                },
            ));
        }
    }
    // dbg!(&cells_with_mm.iter().map(|cm| &cm.0).collect::<Vec<_>>());
    commands.spawn_batch(cells_with_mm);
    // dashboard
    let ts = TextStyle {
        font_size: 30.0,
        ..Default::default()
    };
    commands.spawn(Text2dBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: String::new(),
                    style: ts.clone(),
                },
                TextSection {
                    value: String::new(),
                    style: ts,
                },
            ],
            ..Default::default()
        },
        transform: Transform::from_xyz((HALF_LEN as f32 + 10.0) * (SIZE + GAP), 0.0, 0.0),
        ..Default::default()
    });
    commands.spawn(Camera2dBundle::default());
}

// Any live cell with fewer than two live neighbours dies (referred to as underpopulation).
// Any live cell with more than three live neighbours dies (referred to as overpopulation).
// Any live cell with two or three live neighbours lives, unchanged, to the next generation.
// Any dead cell with exactly three live neighbours comes to life.
fn dead_or_alive(mut db: ResMut<Dashboard>, mut query: Query<&mut Cell>) {
    let alive_coords: Vec<_> = query
        .iter()
        .filter(|c| c.state == State::Alive)
        .map(|c| c.index_xy)
        .collect();
    if alive_coords.len() == 0 {
        // end of the world
        return;
    }

    db.survival = 0;
    for mut cell in query.iter_mut() {
        let live_count = alive_neighbor_count(&cell.index_xy, &alive_coords);
        match cell.state {
            State::Alive => {
                if live_count < 2 || live_count > 3 {
                    cell.state = State::Dead
                }
            }
            State::Dead => {
                if live_count == 3 {
                    // println!("come to live");
                    cell.state = State::Alive
                }
            }
        }
        // dbg!(&cell);
        if cell.state == State::Alive {
            db.survival += 1;
        }
    }
    db.round += 1;
}

fn alive_neighbor_count(me: &Coord, alives: &Vec<Coord>) -> usize {
    let (x, y) = me.to_owned();
    // surrounding 8
    [
        (x - 1, y - 1),
        (x - 1, y),
        (x - 1, y + 1),
        (x, y - 1),
        (x, y + 1),
        (x + 1, y - 1),
        (x + 1, y),
        (x + 1, y + 1),
    ]
    .iter()
    .filter(|(x, y)| {
        let valid = *x >= -HALF_LEN && *y >= -HALF_LEN && *x <= HALF_LEN && *y <= HALF_LEN;
        valid && alives.iter().any(|(ax, ay)| x == ax && y == ay)
    })
    .count()
}

#[test]
fn test_alive_neighbor() {
    let alives = vec![(-1, -1), (-1, 0), (0, -1)];
    assert_eq!(alive_neighbor_count(&(0, 0), &alives), 3);
    assert_eq!(alive_neighbor_count(&(-2, -1), &alives), 2);
}

fn update_cell_color(mut query: Query<(&mut Sprite, &Cell)>) {
    for (mut sprite, cell) in query.iter_mut() {
        match cell.state {
            State::Dead => sprite.color = Color::GRAY,
            State::Alive => sprite.color = Color::WHITE,
        }
    }
}

fn update_dashboard(db: Res<Dashboard>, mut query: Query<&mut Text>) {
    for mut t in query.iter_mut() {
        t.sections[0].value = format!("Round: {} \n", db.round);
        t.sections[1].value = format!("Survival: {} ", db.survival);
    }
}

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_duration(TICK))
        .insert_resource(Dashboard::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, dead_or_alive)
        .add_systems(Update, update_cell_color)
        .add_systems(Update, update_dashboard)
        .run();
}
