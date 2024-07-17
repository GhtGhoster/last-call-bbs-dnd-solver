use std::{collections::HashSet, thread::sleep, time::Duration};

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

#[derive(PartialEq, Eq, Clone, Hash)]
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

    // solve the matrix by random collapses if necessary
    let mut past_matrices = HashSet::new();
    if matrix.iter().any(|row| row.iter().any(|tile| tile == &Tile::Unsure)) {
        matrix = solve(&matrix, &nums_columns, &nums_rows, &mut past_matrices).unwrap();
    }
    debug_print(&matrix, &nums_columns, &nums_rows);

    // focus window
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.move_mouse(1920 + 5, 5, Coordinate::Abs).unwrap();
    sleep(Duration::from_millis(50));
    enigo.button(enigo::Button::Left, enigo::Direction::Click).unwrap();
    sleep(Duration::from_millis(50));

    // execute solution
    for y in 0..8 {
        for x in 0..8 {
            if matrix[y][x] == Tile::Wall {
                enigo.move_mouse(
                    1920 + TILE_X + (x as i32 * TILE_SIZE) + (TILE_SIZE / 2),
                    TILE_Y + (y as i32 * TILE_SIZE) + (TILE_SIZE / 2),
                    Coordinate::Abs,
                ).unwrap();
                sleep(Duration::from_millis(50));
                enigo.button(enigo::Button::Left, enigo::Direction::Click).unwrap();
                sleep(Duration::from_millis(50));
            }
        }
    }
}

fn solve(matrix: &Vec<Vec<Tile>>, nums_columns: &Vec<usize>, nums_rows: &Vec<usize>, past_matrices: &mut HashSet<Vec<Vec<Tile>>>) -> Option<Vec<Vec<Tile>>> {
    // generate random collapses and weed out impossible and previously checked ones
    let mut collapses = collapse_random(matrix);
    'collapses_loop: for i in (0..collapses.len()).rev() {
        loop {
            if !past_matrices.insert(collapses[i].clone()) {
                collapses.remove(i);
                continue 'collapses_loop;
            }
            if !is_possible(&collapses[i], nums_columns, nums_rows) {
                collapses.remove(i);
                continue 'collapses_loop;
            }
            let last_matrix = collapses[i].clone();
            collapse_certainties(&mut collapses[i], nums_columns, nums_rows);
            if last_matrix == collapses[i] {
                break;
            }
        }
    }

    if collapses.is_empty() {
        return None;
    }

    // score and matrices based on unsure tiles
    let mut collapses: Vec<(Vec<Vec<Tile>>, usize)> = collapses.into_iter().map(|collapse| {
        let mut certainty = 64;
        for x in 0..8 {
            for y in 0..8 {
                if collapse[y][x] == Tile::Unsure {
                    certainty -= 1;
                }
            }
        }
        (collapse, certainty)
    }).collect();
    collapses.sort_by(|(_, a), (_, b)| b.cmp(&a));

    // seek win
    for (collapse, certainty) in collapses.iter() {
        if certainty == &64 {
            return Some(collapse.clone());
        }
        let solution = solve(collapse, nums_columns, nums_rows, past_matrices);
        if solution.is_some() {
            return solution;
        }
    }
    None
}

fn collapse_random(matrix: &Vec<Vec<Tile>>) -> Vec<Vec<Vec<Tile>>> {
    let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut collapses = vec![];

    // collapse monsters
    for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Monster {
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            if tile == &Tile::Unsure {
                                let mut new_matrix = matrix.clone();
                                for (ddx, ddy) in directions {
                                    let nnx = (x as i32 + ddx) as usize;
                                    let nny = (y as i32 + ddy) as usize;
                                    if let Some(row) = new_matrix.get_mut(nny) {
                                        if let Some(tile) = row.get_mut(nnx) {
                                            *tile = Tile::Wall;
                                        }
                                    }
                                }
                                new_matrix[ny][nx] = Tile::Ground;
                                collapses.push(new_matrix);
                            }
                        }
                    }
                }
            }
        }
    }

    // collapse treasure rooms
    for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Chest {
                for chest_offset_x in 0..3 {
                    'seek_top_left: for chest_offset_y in 0..3 {
                        let top_left_x = x - 1 - chest_offset_x;
                        let top_left_y = y - 1 - chest_offset_y;
                        // verify room position plausible
                        let mut side_wall_ground = false;
                        for room_x in 0..5 {
                            for room_y in 0..5 {
                                if let Some(row) = matrix.get(top_left_y + room_y) {
                                    if let Some(tile) = row.get(top_left_x + room_x) {
                                        if room_x == 0 || room_x == 4 || room_y == 0 || room_y == 4 {
                                            // room wall
                                            if !((room_x == 0 || room_x == 4) && (room_y == 0 || room_y == 4)) {
                                                // not a corner (side wall)
                                                match tile {
                                                    Tile::Chest | Tile::Monster => continue 'seek_top_left,
                                                    Tile::Ground => {
                                                        if side_wall_ground {
                                                            continue 'seek_top_left;
                                                        } else {
                                                            side_wall_ground = true;
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        } else {
                                            // room inside
                                            match tile {
                                                Tile::Monster | Tile::Wall => continue 'seek_top_left,
                                                _ => (),
                                            }
                                        }
                                    } else {
                                        if !(room_x == 0 || room_x == 4 || room_y == 0 || room_y == 4) {
                                            continue 'seek_top_left;
                                        }
                                    }
                                } else {
                                    if !(room_x == 0 || room_x == 4 || room_y == 0 || room_y == 4) {
                                        continue 'seek_top_left;
                                    }
                                }
                            }
                        }
                        // top left corner coord verified to be plausible
                        // generate all certain structures with all possible exits
                        let mut tmp_matrix = matrix.clone();
                        for room_x in 0..5 {
                            for room_y in 0..5 {
                                if let Some(row) = tmp_matrix.get_mut(top_left_y + room_y) {
                                    if let Some(tile) = row.get_mut(top_left_x + room_x) {
                                        if room_x == 0 || room_x == 4 || room_y == 0 || room_y == 4 {
                                            // room wall
                                            if !((room_x == 0 || room_x == 4) && (room_y == 0 || room_y == 4)) {
                                                // not a corner (side wall)
                                                if tile == &Tile::Unsure {
                                                    *tile = Tile::Wall;
                                                }
                                            }
                                        } else {
                                            // room inside
                                            if tile == &Tile::Unsure {
                                                *tile = Tile::Ground;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // tmp_matrix contains walled off room or 1 exit room
                        if side_wall_ground {
                            // exit already predetermined
                            collapses.push(tmp_matrix);
                        } else {
                            // any exit position plausible
                            for room_x in 0..5 {
                                for room_y in 0..5 {
                                    if let Some(row) = tmp_matrix.get(top_left_y + room_y) {
                                        if let Some(_) = row.get(top_left_x + room_x) {
                                            if room_x == 0 || room_x == 4 || room_y == 0 || room_y == 4 {
                                                if !((room_x == 0 || room_x == 4) && (room_y == 0 || room_y == 4)) {
                                                    // room wall, not a corner (side wall)
                                                    let mut exit_matrix = tmp_matrix.clone();
                                                    exit_matrix[top_left_y + room_y][top_left_x + room_x] = Tile::Ground;
                                                    collapses.push(exit_matrix);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // collapse random tiles
    // (no need to check for validity as this is only ever used on a certainty collapsed matrix)
    for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Unsure {
                let mut new_matrix = matrix.clone();
                new_matrix[y][x] = Tile::Ground;
                collapses.push(new_matrix);
                let mut new_matrix = matrix.clone();
                new_matrix[y][x] = Tile::Wall;
                collapses.push(new_matrix);
            }
        }
    }

    collapses
}

fn is_possible(matrix: &Vec<Vec<Tile>>, nums_columns: &Vec<usize>, nums_rows: &Vec<usize>) -> bool {
    // check for 2x2 spaces
    for x in 0..7 {
        for y in 0..7 {
            if 
                matrix[y][x] == Tile::Ground &&
                matrix[y+1][x] == Tile::Ground &&
                matrix[y][x+1] == Tile::Ground &&
                matrix[y+1][x+1] == Tile::Ground
            {
                let mut chest_found = false;
                for cx in 0..4 {
                    for cy in 0..4 {
                        if cx == 0 || cy == 0 || cx == 3 || cy == 3 {
                            if let Some(row) = matrix.get(y - 1 + cy) {
                                if let Some(tile) = row.get(x - 1 + cx) {
                                    if tile == &Tile::Chest {
                                        chest_found = true;
                                    }
                                }
                            }
                        }
                    }
                }
                if !chest_found {
                    return false;
                }
            }
        }
    }

    // check that there exists at least one possible way to have the treasure room
    for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Chest {
                let mut ground_possible_coords = vec![];
                // check for monsters or walls within the area of all possible treasure rooms
                for dx in 0..3 {
                    'chest_loop: for dy in 0..3 {
                        for ix in 0..3 {
                            for iy in 0..3 {
                                if let Some(row) = matrix.get(y - dy + iy) {
                                    if let Some(tile) = row.get(x - dx + ix) {
                                        if tile == &Tile::Monster || tile == &Tile::Wall {
                                            continue 'chest_loop;
                                        }
                                    } else {
                                        continue 'chest_loop;
                                    }
                                } else {
                                    continue 'chest_loop;
                                }
                            }
                        }
                        ground_possible_coords.push((dx, dy));
                    }
                }
                // check for monsters and at least one non-wall tile along the edges
                'wall_loop: for i in (0..ground_possible_coords.len()).rev() {
                    let mut entrance_possible = false;
                    let (dx, dy) = ground_possible_coords[i];
                    for ix in 0..5 {
                        for iy in 0..5 {
                            if ix == 0 || iy == 0 || ix == 4 || iy == 4 {
                                if let Some(row) = matrix.get(y - dy - 1 + iy) {
                                    if let Some(tile) = row.get(x - dx - 1 + ix) {
                                        match tile {
                                            Tile::Monster | Tile::Chest => {
                                                if !((ix == 0 || ix == 4) && (iy == 0 || iy == 4)) {
                                                    ground_possible_coords.remove(i);
                                                    continue 'wall_loop;
                                                }
                                            },
                                            Tile::Ground | Tile::Unsure => {
                                                entrance_possible = true;
                                            },
                                            _ => (),
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !entrance_possible {
                        ground_possible_coords.remove(i);
                    }
                }
                if ground_possible_coords.is_empty() {
                    return false;
                }
            }
        }
    }

    // check for monster being in dead-ends (no more than 1 ground tile around them)
    let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Monster {
                let mut ground_tile_possible = false;
                let mut ground_tile = false;
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            match tile {
                                Tile::Unsure => {
                                    ground_tile_possible = true;
                                },
                                Tile::Ground => {
                                    ground_tile_possible = true;
                                    if ground_tile {
                                        return false;
                                    } else {
                                        ground_tile = true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                if !ground_tile_possible {
                    return false;
                }
            }
        }
    }

    // check that dead-ends contain monsters
    for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Ground {
                let mut adj_grounds = 0;
                let mut adj_monster = false;
                for (dx, dy) in directions {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            match tile {
                                Tile::Ground | Tile::Unsure => adj_grounds += 1,
                                Tile::Monster => adj_monster = true,
                                _ => (),
                            }
                        }
                    }
                }
                if adj_grounds == 1 && !adj_monster {
                    return false;
                }
            }
        }
    }

    // check wall numbers
    for i in 0..8 {
        let wall_count = matrix[i].iter().filter(|item| item == &&Tile::Wall).count();
        let unsure_count = matrix[i].iter().filter(|item| item == &&Tile::Unsure).count();
        if wall_count > nums_rows[i] {
            return false;
        }
        if wall_count + unsure_count < nums_rows[i] {
            return false;
        }
        let wall_count = matrix.iter().filter(|row| row[i] == Tile::Wall).count();
        let unsure_count = matrix.iter().filter(|row| row[i] == Tile::Unsure).count();
        if wall_count > nums_columns[i] {
            return false;
        }
        if wall_count + unsure_count < nums_columns[i] {
            return false;
        }
    }

    // check for ground tile continuity
    let mut flooded_coords = HashSet::new();
    'seek_ground: for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Ground {
                flooded_coords.insert((x, y));
                break 'seek_ground;
            }
        }
    }
    if !flooded_coords.is_empty() {
        loop {
            let mut new_flooded_coords = vec![];
            for (x, y) in flooded_coords.iter() {
                for (dx, dy) in directions {
                    let nx = (*x as i32 + dx) as usize;
                    let ny = (*y as i32 + dy) as usize;
                    if let Some(row) = matrix.get(ny) {
                        if let Some(tile) = row.get(nx) {
                            match tile {
                                Tile::Ground | Tile::Unsure => {
                                    if !flooded_coords.contains(&(nx, ny)) {
                                        new_flooded_coords.push((nx, ny));
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                }
            }
            if new_flooded_coords.is_empty() {
                break;
            } else {
                for new_flooded_coord in new_flooded_coords {
                    flooded_coords.insert(new_flooded_coord);
                }
            }
        }
    }
    for x in 0..8 {
        for y in 0..8 {
            if matrix[y][x] == Tile::Ground && !flooded_coords.contains(&(x, y)){
                return false;
            }
        }
    }

    // TODO (or not)
    // check for treasure room placement rules (1 exit, 3x3ness, ...)
    true
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
