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

enum Tile {
    Ground,
    Chest,
    Monster,
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
                row.push(Tile::Ground);
            } else {
                row.push(Tile::Monster);
            }
        }
        matrix.push(row);
    }

    let image = screen.capture_area(
        TILE_X - TILE_SIZE,
        TILE_Y + TILE_SIZE*0,
        TILE_SIZE as u32,
        TILE_SIZE as u32,
    ).unwrap();
    detect_number(image);

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

    // print debug
    print!(" ");
    for col_num in nums_columns {
        print!("{col_num}");
    }
    println!();
    for (i, row) in matrix.iter().enumerate() {
        print!("{}", nums_rows[i]);
        for item in row {
            match item {
                Tile::Chest => print!("#"),
                Tile::Ground => print!("_"),
                Tile::Monster => print!("!"),
            }
        }
        println!();
    }

    // let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // enigo.move_mouse(x as i32, y as i32, Coordinate::Abs).unwrap();
    // sleep(Duration::from_millis(50));
    // enigo.button(enigo::Button::Left, enigo::Direction::Click).unwrap();
    // sleep(Duration::from_millis(50));
}

fn detect_number(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> u8 {
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
