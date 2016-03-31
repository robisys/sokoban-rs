// This file is part of sokoban-rs
// Copyright 2015 Sébastien Watteau
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This is an implementation of Sokoban in Rust.

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate xml;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use clap::App;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2_image::INIT_PNG;
use xml::reader::EventReader;
use xml::reader::XmlEvent;

pub mod error;
pub mod game;
pub mod render;

use game::Level;
use render::Drawer;

///  # Sokoban  Main funktion
///
///  * aufruf
///
///  Inhalt
///
pub fn main() {
//! load_yaml!
//!
    let yml = load_yaml!("clap.yml");
    let matches = App::from_yaml(yml).get_matches();

    let slc_file = matches.value_of("slc_file").unwrap();
    let width = value_t!(matches.value_of("width"), u32).unwrap_or(1024);
    let height = value_t!(matches.value_of("height"), u32).unwrap_or(768);
//  panic!
//
    let sdl_context = sdl2::init().unwrap_or_else(|err| {
        panic!("Failed to initialize an SDL context: {}", err)
    });

    let video_subsystem = sdl_context.video()
                                     .unwrap_or_else(|err| {
                                         panic!("Failed to initialize the video subsystem: {}", err)
                                     });

    let mut window_builder = video_subsystem.window("sokoban-rs", width, height);
    if matches.is_present("fullscreen") {
        window_builder.fullscreen();
    } else {
        window_builder.position_centered();
    }

    let window = window_builder.opengl()
                               .build()
                               .unwrap_or_else(|err| {
                                   panic!("Failed to create the window: {}", err)
                               });

    let renderer = window.renderer()
                         .build()
                         .unwrap_or_else(|err| {
                             panic!("Failed to get an SDL renderer for the main window: {}", err)
                         });

    sdl2_image::init(INIT_PNG);
    let _ttf = sdl2_ttf::init();

    let mut drawer = Drawer::new(renderer);

    let mut collection = load_slc_file(Path::new(&slc_file))
                             .unwrap_or_else(|err| panic!("{}", err))
                             .into_iter();
    let mut reference_level = collection.next().unwrap();
    let mut level = reference_level.clone();

    let mut running = true;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut skip = false;
    while running {
        if level.is_completed() || skip {
            match collection.next() {
                Some(l) => {
                    level = l;
                    reference_level = level.clone();
                    skip = false;
                }
                None => {
                    break;
                }
            }
        }
        drawer.draw(&level);

        match event_pump.wait_event() {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                running = false
            }
            Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                level.step(game::Direction::Left);
            }
            Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                level.step(game::Direction::Right);
            }
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                level.step(game::Direction::Up);
            }
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                level.step(game::Direction::Down);
            }
            Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                level = reference_level.clone();
            }
            Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                skip = true;
            }
            _ => {}
        }
    }
    sdl2_image::quit();
}

/// Builds levels from a level collection file in the SLC format.
fn load_slc_file(path: &Path) -> Result<Vec<Level>, error::SokobanError> {
    let mut collection = Vec::new();
    let file = try!(File::open(&path));
    let reader = BufReader::new(file);
    let parser = EventReader::new(reader);

    let mut level_title = String::new();
    let mut level_str = String::new();
    let mut reading_level = false;
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { ref name, ref attributes, .. }) => {
                if name.local_name == "L" {
                    reading_level = true;
                } else if name.local_name == "Level" {
                    if let Some(id) = attributes.iter().find(|&attr| attr.name.local_name == "Id") {
                        level_title = id.value.clone();
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "Level" {
                    let mut level = try!(Level::from_str(&level_str));
                    level.set_title(level_title.clone());
                    collection.push(level);
                    level_str.clear();
                }
            }
            Ok(XmlEvent::Characters(ref data)) => {
                if reading_level {
                    level_str.push_str(data);
                    level_str.push('\n');
                }
            }
            _ => {}
        }
    }

    Ok(collection)
}
