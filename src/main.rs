use std::{thread::sleep, time::Duration};

use enigo::{Coordinate, Enigo, Mouse, Settings};
use screenshots::{image::{io::Reader, DynamicImage, GenericImageView, ImageBuffer}, Screen};

const TILE_X: i32 = 722;
const TILE_Y: i32 = 428;
const TILE_SIZE: i32 = 66;

const CAPTURE_SIZE: i32 = 8;

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

    // print debug
    for row in matrix {
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
