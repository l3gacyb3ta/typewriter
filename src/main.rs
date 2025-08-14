extern crate sdl3;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::TextureQuery;

static SCREEN_WIDTH: u32 = 1366;
static SCREEN_HEIGHT: u32 = 768;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = 30;
    let cy = 30;
    rect!(cx, cy, w, h)
}

enum State {
    NoFile,
    OpenFile(String),
}

impl State {
    pub fn open(&mut self, path: String) -> Result<String, Box<dyn std::error::Error>> {
        let reader = File::open(&path);
        let mut out: String = Default::default();

        match reader {
            Ok(mut reader) => match reader.read_to_string(&mut out) {
                Ok(_) => {}
                Err(_) => out = "".to_owned(),
            },
            Err(_) => out = "".to_owned(),
        }

        *self = State::OpenFile(path.clone());

        Ok(out)
    }
}

enum SavedState {
    Saved,
    Dirty,
}

fn get_text_files(fnt_path: &Path) -> String {
    let path = fnt_path.parent().unwrap().join("text");
    std::fs::create_dir_all(&path).unwrap();

    let mut out = "".to_string();

    for file in std::fs::read_dir(path).unwrap() {
        if file.is_ok() {
            let file = file.unwrap();

            if file.path().extension().is_some_and(|ext| ext == "txt") {
                out = out + " * " + file.path().file_stem().unwrap().to_str().unwrap() + ".txt\n";
            }
        }
    }

    out
}

fn run(font_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl3::ttf::init().map_err(|e| e.to_string())?;

    let shift_map = HashMap::from([
        ("1", "!"),
        ("2", "@"),
        ("3", "#"),
        ("4", "$"),
        ("5", "%"),
        ("6", "^"),
        ("7", "&"),
        ("8", "*"),
        ("9", "("),
        ("0", ")"),
        ("-", "_"),
        ("=", "+"),
        ("[", "{"),
        ("]", "}"),
        ("\\", "|"),
        (";", ":"),
        ("'", "\""),
        (",", "<"),
        (".", ">"),
        ("/", "?"),
        ("`", "~"),
    ]);

    let window = video_subsys
        .window("typewriter", SCREEN_WIDTH, SCREEN_HEIGHT)
        .borderless()
        .fullscreen()
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();

    // Load a font
    let mut font = ttf_context.load_font(font_path, 16.0)?;
    font.set_style(sdl3::ttf::FontStyle::NORMAL);

    let mut state = State::NoFile;
    let mut saved_state = SavedState::Saved;
    let mut text = get_text_files(font_path) + "Open File: ";

    render(&mut canvas, &texture_creator, &font, &text, &saved_state)?;


    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    repeat,
                    scancode: Some(_),
                    ..
                } => {
                    match keycode {
                        // Keycode::Escape => break 'mainloop,
                        keycode => {
                            // UPDATE SCREEN

                            // render a surface, and convert it to a texture bound to the canvas

                            render(&mut canvas, &texture_creator, &font, &text, &saved_state)?;

                            if (!repeat) && keycode.to_string().len() == 1 {
                                let shifted = keymod
                                    .iter()
                                    .map(|m| m == Mod::LSHIFTMOD || m == Mod::RSHIFTMOD)
                                    .fold(false, |acc, m| m || acc);

                                let controlled = keymod
                                    .iter()
                                    .map(|m| m == Mod::LCTRLMOD || m == Mod::RCTRLMOD)
                                    .fold(false, |acc, m| m || acc);

                                if controlled {
                                    match keycode {
                                        Keycode::S => match state {
                                            State::NoFile => {
                                                saved_state = SavedState::Saved;

                                                render(
                                                    &mut canvas,
                                                    &texture_creator,
                                                    &font,
                                                    &text,
                                                    &saved_state,
                                                )?;
                                                continue;
                                            }
                                            State::OpenFile(ref file) => {
                                                saved_state = SavedState::Saved;
                                                std::fs::write(file, &text).unwrap();

                                                render(
                                                    &mut canvas,
                                                    &texture_creator,
                                                    &font,
                                                    &text,
                                                    &saved_state,
                                                )?;
                                                continue;
                                            }
                                        },
                                        Keycode::O => {
                                            match state {
                                                State::NoFile => {}
                                                State::OpenFile(ref file) => {
                                                    std::fs::write(file, text).unwrap();
                                                }
                                            }
                                            state = State::NoFile;
                                            saved_state = SavedState::Saved;

                                            text = get_text_files(font_path) + "Open File: ";

                                            render(
                                                &mut canvas,
                                                &texture_creator,
                                                &font,
                                                &text,
                                                &saved_state,
                                            )?;
                                            continue;
                                        }

                                        Keycode::Q => {
                                            match state {
                                                State::NoFile => {}
                                                State::OpenFile(ref file) => {
                                                    if !shifted {
                                                        std::fs::write(file, text).unwrap();
                                                    }
                                                }
                                            }

                                            break 'mainloop;
                                        }
                                        _ => {}
                                    }
                                }

                                let key = keycode.to_string().to_lowercase();
                                if key != "" {
                                    saved_state = SavedState::Dirty;
                                }

                                text = text
                                    + &if shifted {
                                        if shift_map.contains_key(&key.as_ref()) {
                                            shift_map
                                                .get(&key.as_ref())
                                                .unwrap()
                                                .to_owned()
                                                .to_owned()
                                        } else {
                                            key.to_uppercase()
                                        }
                                    } else {
                                        keycode.to_string().to_lowercase()
                                    };
                            } else if keycode == Keycode::Backspace {
                                text.pop();
                                saved_state = SavedState::Dirty;
                            } else {
                                let key = match keycode {
                                    Keycode::Space => " ",
                                    Keycode::Underscore => "_",
                                    Keycode::Return => match state {
                                        State::NoFile => {
                                            let path = if text.split("\n").last().unwrap().starts_with("Open File: ") {
                                                text.split("\n").last().unwrap().strip_prefix("Open File: ").unwrap()
                                            } else {
                                                &text
                                            };

                                            let rl_path = font_path.parent().unwrap().join("text").join(path);
                                            text = state.open(rl_path.to_str().unwrap().to_owned()).unwrap();

                                            ""
                                        }
                                        State::OpenFile(..) => "\n",
                                    },
                                    _ => {
                                        // println!("Unknown keycode {}", keycode);
                                        ""
                                    }
                                };

                                text = text + key;
                                if key != "" {
                                    saved_state = SavedState::Dirty;
                                }

                                render(&mut canvas, &texture_creator, &font, &text, &saved_state)?;
                            }
                        }
                    }

                    render(&mut canvas, &texture_creator, &font, &text, &saved_state)?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn render(
    canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
    texture_creator: &sdl3::render::TextureCreator<sdl3::video::WindowContext>,
    font: &sdl3::ttf::Font<'static>,
    text: &String,
    saved_state: &SavedState,
) -> Result<(), Box<dyn std::error::Error>> {
    let subs = (text.to_owned() + "_")
        .split('\n')
        .map(|line| {
            line.chars()
                .collect::<Vec<char>>()
                .chunks(120)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join("\n")
        })
        .collect::<Vec<String>>()
        .join("\n")
        .lines()
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .take(30)
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, line)| {
            if (i + 1) % 5 == 0 {
                format!("{:>4} │ {}", i + 1, line)
            } else {
                format!("     │ {}", line)
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    let surface = font
        .render(&subs)
        .blended_wrapped(Color::RGBA(0, 0, 0, 255), 1000)
        .map_err(|e| e.to_string())?;

    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGBA(0xfa, 0xfa, 0xfa, 255));
    canvas.clear();

    match *saved_state {
        SavedState::Saved => canvas.set_draw_color(Color::RGBA(0xf0, 0xf0, 0xf0, 255)),
        SavedState::Dirty => canvas.set_draw_color(Color::RGBA(0x0a, 0x0a, 0x0a, 255)),
    }

    canvas.fill_rect(rect!(SCREEN_WIDTH - 30, 35, 5, 5)).unwrap();
    let TextureQuery { width, height, .. } = texture.query();
    let padding = 64;
    let target = get_centered_rect(width, height, SCREEN_WIDTH - padding, SCREEN_HEIGHT);

    canvas.copy(&texture, None, Some(target.into()))?;
    canvas.present();

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("linked sdl3_ttf: {}", sdl3::ttf::get_linked_version());

    let path = env::home_dir().unwrap().join("typewriter");
    std::fs::create_dir_all(&path)?;

    run(&(path.join("VictorMono.ttf").as_path()))?;

    Ok(())
}
