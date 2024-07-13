use std::{thread::sleep, time::Duration};

use enigo::{Coordinate, Enigo, Mouse, Settings};
use screenshots::{image::{imageops::overlay, io::Reader, DynamicImage, ImageBuffer, Rgba}, Screen};

const TILE_X: i32 = 722;
const TILE_Y: i32 = 428;
const TILE_SIZE: i32 = 66;

const CAPTURE_SIZE: i32 = 8;

const ORANGE: [u8; 4] = [250, 91, 69, 255];
const GRAY: [u8; 4] = [98, 91, 77, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];

#[derive(PartialEq, Eq, Clone)]
enum Tile {
    Unsure,
    Chest,
    Monster,
    Ground,
    Wall,
}

fn main() {
    // load comparison image assets
    let mut ground_images = vec![];
    for i in 0..8 {
        for j in 0..8 {
            let tmp = Reader::open(format!("assets/ground/{j}x{i}.png")).unwrap().decode().unwrap();
            ground_images.push(tmp);
        }
    }
    let chest_image = Reader::open(format!("assets/chest.png")).unwrap().decode().unwrap();

    // detect dungeon layout from screen
    let screens = Screen::all().unwrap();
    let screen = screens[0];

    let mut matrix = vec![];
    for i in 0..8 {
        let mut row = vec![];
        for j in 0..8 {
            let image = screen.capture_area(
                TILE_X + (TILE_SIZE-CAPTURE_SIZE)/2 + (j*TILE_SIZE),
                TILE_Y + (TILE_SIZE-CAPTURE_SIZE)/2 + (i*TILE_SIZE),
                CAPTURE_SIZE as u32,
                CAPTURE_SIZE as u32,
            ).unwrap();
            let image = DynamicImage::from(image);
            if image == chest_image {
                row.push(Tile::Chest);
            } else if ground_images.contains(&image) {
                row.push(Tile::Unsure);
            } else {
                row.push(Tile::Monster);
            }
        }
        matrix.push(row);
    }

    // detect numbers
    let mut nums_columns = vec![];
    let mut nums_rows = vec![];
    for i in 0..8 {
        // column numbers
        let image = screen.capture_area(
            TILE_X - TILE_SIZE,
            TILE_Y + (i*TILE_SIZE),
            TILE_SIZE as u32,
            TILE_SIZE as u32,
        ).unwrap();
        nums_rows.push(detect_number(image));

        // row numbers
        let image = screen.capture_area(
            TILE_X + 8 + (i*TILE_SIZE),
            TILE_Y - 6 - TILE_SIZE,
            TILE_SIZE as u32,
            TILE_SIZE as u32,
        ).unwrap();
        nums_columns.push(detect_number(image));
    }

    // resolve certainties
    loop {
        let last_matrix = matrix.clone();
        collapse_certainties(&mut matrix, &nums_columns, &nums_rows);
        if matrix == last_matrix {
            break;
        }
    }
    debug_print(&matrix, &nums_columns, &nums_rows);

    // generate all random collapses
    // for each, loop
    //  if impossible, remove from list
    //  collapse certainties
    //  if no change, break loop
    // if win, return win
    // if list empty, return none
    // sort them by how much they collapsed
    //  either how many collapse cycles they went through
    //  or how many parts of the matrix are now collapsed (probably better)
    // recurse with all until win

    // let mut enigo = Enigo::new(&Settings::default()).unwrap();
    // enigo.move_mouse(x as i32, y as i32, Coordinate::Abs).unwrap();
    // sleep(Duration::from_millis(50));
    // enigo.button(enigo::Button::Left, enigo::Direction::Click).unwrap();
    // sleep(Duration::from_millis(50));
}

fn is_impossible(matrix: &Vec<Vec<Tile>>, nums_columns: &Vec<usize>, nums_rows: &Vec<usize>) -> bool {
    // check for 2x2 spaces
    for x in 0..7 {
        for y in 0..7 {
            if 
                matrix[y][x] == Tile::Ground &&
                matrix[y+1][x] == Tile::Ground &&
                matrix[y][x+1] == Tile::Ground &&
                matrix[y+1][x+1] == Tile::Ground
            {
                return false;
            }
        }
    }
    // check for treasure room impossibilities
    // TODO

    false
}

fn collapse_certainties(matrix: &mut Vec<Vec<Tile>>, nums_columns: &Vec<usize>, nums_rows: &Vec<usize>) {
    let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    // collapse resolved rows
    for i in 0..8 {
        let unsure_count = matrix[i].iter().filter(|item| item == &&Tile::Unsure).count();
        let wall_count = matrix[i].iter().filter(|item| item == &&Tile::Wall).count();
        if unsure_count == nums_rows[i] - wall_count {
            for j in 0..8 {
                if matrix[i][j] == Tile::Unsure {
                    matrix[i][j] = Tile::Wall;
                }
            }
        }
        if wall_count == nums_rows[i] && unsure_count > 0 {
            for j in 0..8 {
                if matrix[i][j] == Tile::Unsure {
                    matrix[i][j] = Tile::Ground;
                }
            }
        }
    }

    // collapse resolved columns
    for i in 0..8 {
        let unsure_count = matrix.iter().filter(|row| row[i] == Tile::Unsure).count();
        let wall_count = matrix.iter().filter(|row| row[i] == Tile::Wall).count();
        if unsure_count == nums_columns[i] - wall_count {
            for j in 0..8 {
                if matrix[j][i] == Tile::Unsure {
                    matrix[j][i] = Tile::Wall;
                }
            }
        }
        if wall_count == nums_columns[i] && unsure_count > 0 {
            for j in 0..8 {
                if matrix[j][i] == Tile::Unsure {
                    matrix[j][i] = Tile::Ground;
                }
            }
        }
    }

    // collapse monster escape routes
    for y in 0..8 {
        for x in 0..8 {
            if matrix[y][x] == Tile::Monster {
                let mut walls = 0;
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            if tile == &Tile::Wall {
                                walls += 1;
                            }
                        } else {
                            walls += 1;
                        }
                    } else {
                        walls += 1;
                    }
                }
                if walls == 3 {
                    for (dx, dy) in directions {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if let Some(row) = matrix.get_mut(ny) {
                            if let Some(tile) = row.get_mut(nx) {
                                if tile != &Tile::Wall {
                                    *tile = Tile::Ground;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // collapse monster wall enclosures
    for y in 0..8 {
        for x in 0..8 {
            if matrix[y][x] == Tile::Monster {
                let mut escape_route = false;
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            if tile == &Tile::Ground {
                                escape_route = true;
                                break;
                            }
                        }
                    }
                }
                if escape_route {
                    for (dx, dy) in directions {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if let Some(row) = matrix.get_mut(ny) {
                            if let Some(tile) = row.get_mut(nx) {
                                if tile != &Tile::Ground {
                                    *tile = Tile::Wall;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // rows and cols with a chest always have at least 2 ground tiles
    for y in 0..8 {
        for x in 0..8 {
            if matrix[y][x] == Tile::Chest {
                // wall is right next to chest
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            if tile == &Tile::Wall {
                                let mx = (x as i32 - dx) as usize;
                                let my = (y as i32 - dy) as usize;
                                let mx2 = (x as i32 - dx * 2) as usize;
                                let my2 = (y as i32 - dy * 2) as usize;
                                matrix[my][mx] = Tile::Ground;
                                matrix[my2][mx2] = Tile::Ground;
                            }
                        } else {
                            let mx = (x as i32 - dx) as usize;
                            let my = (y as i32 - dy) as usize;
                            let mx2 = (x as i32 - dx * 2) as usize;
                            let my2 = (y as i32 - dy * 2) as usize;
                            matrix[my][mx] = Tile::Ground;
                            matrix[my2][mx2] = Tile::Ground;
                        }
                    } else {
                        let mx = (x as i32 - dx) as usize;
                        let my = (y as i32 - dy) as usize;
                        let mx2 = (x as i32 - dx * 2) as usize;
                        let my2 = (y as i32 - dy * 2) as usize;
                        matrix[my][mx] = Tile::Ground;
                        matrix[my2][mx2] = Tile::Ground;
                    }
                }

                // wall is gapped from the chest
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx * 2) as usize;
                    let ny = (y as i32 + dy * 2) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            if tile == &Tile::Wall {
                                let mx = (x as i32 - dx) as usize;
                                let my = (y as i32 - dy) as usize;
                                matrix[my][mx] = Tile::Ground;
                            }
                        } else {
                            let mx = (x as i32 - dx) as usize;
                            let my = (y as i32 - dy) as usize;
                            matrix[my][mx] = Tile::Ground;
                        }
                    } else {
                        let mx = (x as i32 - dx) as usize;
                        let my = (y as i32 - dy) as usize;
                        matrix[my][mx] = Tile::Ground;
                    }
                }
            }
        }
    }

    // let's set some ground rules
    for y in 0..8 {
        for x in 0..8 {
            if matrix[y][x] == Tile::Ground {
                let mut walls = 0;
                let mut monsters = 0;
                let mut unsures = 0;
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            match tile {
                                Tile::Unsure => unsures += 1,
                                Tile::Chest => (),
                                Tile::Monster => monsters += 1,
                                Tile::Ground => (),
                                Tile::Wall => walls += 1,
                            }
                        } else {
                            walls += 1;
                        }
                    } else {
                        walls += 1;
                    }
                }
                if unsures != 0 && (monsters == 3 || walls == 2) {
                    // replace unsure with ground
                    for (dx, dy) in directions {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if let Some(row) = matrix.get_mut(ny) {
                            if let Some(tile) = row.get_mut(nx) {
                                if tile == &Tile::Unsure {
                                    *tile = Tile::Ground
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn debug_print(matrix: &Vec<Vec<Tile>>, nums_columns: &Vec<usize>, nums_rows: &Vec<usize>) {
    print!(" ");
    for col_num in nums_columns {
        print!("{col_num}");
    }
    println!();
    for (i, row) in matrix.iter().enumerate() {
        print!("{}", nums_rows[i]);
        for item in row {
            match item {
                Tile::Chest => print!("O"),
                Tile::Unsure => print!("?"),
                Tile::Monster => print!("!"),
                Tile::Ground => print!("_"),
                Tile::Wall => print!("#"),
            }
        }
        println!();
    }
}

fn detect_number(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> usize {
    for i in 0..8 {
        let comparison_image = Reader::open(format!("assets/numbers/{i}.png")).unwrap().decode().unwrap();
        for dx in -5..=5 {
            for dy in -5..=5 {
                let mut diff = 0;
                let mut comparison_image_shifted = ImageBuffer::from_pixel(
                    comparison_image.width(),
                    comparison_image.height(),
                    Rgba(WHITE),
                );
                overlay(&mut comparison_image_shifted, &comparison_image, dx, dy);
                for x in 0..image.width() {
                    for y in 0..image.height() {
                        let comparison_pixel = comparison_image_shifted.get_pixel(x, y).0;
                        let pixel = image.get_pixel(x, y).0;
                        if comparison_pixel == BLACK && !(pixel == ORANGE || pixel == GRAY) {
                            diff += 1;
                        }
                    }
                }
                if diff == 0 {
                    return i;
                }
            }
        }
    }
    8
}
