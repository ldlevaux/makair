#[macro_use]
extern crate log;

#[macro_use]
extern crate conrod_winit;

#[macro_use]
extern crate conrod_core;

mod support;

use plotters::prelude::*;
use std::sync::{Arc, Mutex};

use chrono::offset::{Local, TimeZone};
use chrono::prelude::*;
use chrono::{Date, Duration};
use log::{info, warn};
use std::{thread, time};

use image::{buffer::ConvertBuffer, ImageBuffer, Rgb, RgbImage, RgbaImage};
use rand::Rng;

use conrod_core::{color, widget, Colorable, Positionable, Sizeable, Widget};
use glium::Surface;

use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use telemetry::{gather_telemetry, structures::TelemetryMessage};

pub struct App {
    gl: conrod_glium::Renderer, // OpenGL drawing backend.
    rotation: f64,              // Rotation for the square.
    display: support::GliumDisplayWinitWrapper,
    ui: conrod_core::Ui,
}

type DataPressure = Vec<(DateTime<Local>, u16)>;

impl App {
    fn render(
        &mut self,
        data_pressure: &DataPressure,
    ) -> conrod_core::image::Map<glium::texture::Texture2d> {
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        //let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        //let (x, y) = (
        //args.window_size[0] / 2.0,
        //args.window_size[1] / 2.0 + args.window_size[1] / 4.0,
        //);

        let mut buffer = vec![0; (780 * 200 * 4) as usize];
        let root = BitMapBackend::with_buffer(&mut buffer, (780, 200)).into_drawing_area();
        //let root = BitMapBackend::new(Path::new("/tmp/foo.png"), (780, 200)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let oldest = data_pressure.first().unwrap().0 - chrono::Duration::seconds(40);
        let newest = data_pressure.first().unwrap().0;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(0)
            .y_label_area_size(40)
            .build_ranged(oldest..newest, 0..70)
            .unwrap();
        chart.configure_mesh().draw().unwrap();
        chart
            .draw_series(LineSeries::new(
                data_pressure.iter().map(|x| (x.0, x.1 as i32)),
                ShapeStyle::from(&BLACK).filled(),
            ))
            .unwrap();

        drop(chart);
        drop(root);
        let rgba_image: RgbaImage = RgbImage::from_raw(780, 200, buffer).unwrap().convert();
        let image_dimensions = rgba_image.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
            &rgba_image.into_raw(),
            image_dimensions,
        );
        let image_texture = glium::texture::Texture2d::new(&self.display.0, raw_image).unwrap();

        let (w, h) = (
            image_texture.get_width(),
            image_texture.get_height().unwrap(),
        );
        let mut image_map = conrod_core::image::Map::new();
        let image = image_map.insert(image_texture);

        // The `WidgetId` for our background and `Image` widgets.
        widget_ids!(struct Ids { background, content });
        let ids = Ids::new(self.ui.widget_id_generator());

        {
            let mut ui = self.ui.set_widgets();
            // Draw a light blue background.
            widget::Canvas::new()
                .color(color::LIGHT_BLUE)
                .set(ids.background, &mut ui);
            // Instantiate the `Image` at its full size in the middle of the window.
            widget::Image::new(image)
                .w_h(w as f64, h as f64)
                .middle()
                .set(ids.content, &mut ui);
        }

        image_map
    }

    fn update(&mut self) {
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt;
    }
}

fn addPressure(data: &mut DataPressure, new_point: u16) {
    data.insert(0, (Local::now(), new_point));
    let oldest = data.first().unwrap().0 - chrono::Duration::seconds(40);
    let newest = data.first().unwrap().0;
    let mut i = 0;
    while i != data.len() {
        if oldest > (&mut data[i]).0 || newest < (&mut data[i]).0 {
            let _ = data.remove(i);
        } else {
            i += 1;
        }
    }
}

fn addFakeData(data: &mut Vec<(DateTime<Local>, i32)>) {
    let mut rng = rand::thread_rng();

    data.insert(0, (Local::now(), rng.gen_range(0, 10)));
    let oldest = data.first().unwrap().0 - chrono::Duration::seconds(40);
    let newest = data.first().unwrap().0;
    let mut i = 0;
    while i != data.len() {
        if oldest > (&mut data[i]).0 || newest < (&mut data[i]).0 {
            let _ = data.remove(i);
        } else {
            i += 1;
        }
    }
}

fn main() {
    if std::env::args().len() < 2 {
        println!("Please specify the device to use as a serial port as the first argument");
        return;
    }

    let port_id = std::env::args().nth(1).unwrap();

    simple_logger::init().unwrap();
    let mut data_pressure = Vec::new();

    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Conrod Graph Widget")
        .with_dimensions((800, 400).into());

    let context = glium::glutin::ContextBuilder::new()
        .with_multisampling(4)
        .with_vsync(true);

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([800 as f64, 400 as f64]).build();

    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);
    let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();

    // Create a new game and run it.
    let mut app_core = App {
        gl: renderer,
        display,
        rotation: 0.0,
        ui,
    };

    let mut event_loop = support::EventLoop::new();

    let (tx, rx): (Sender<TelemetryMessage>, Receiver<TelemetryMessage>) =
        std::sync::mpsc::channel();

    std::thread::spawn(move || {
        gather_telemetry(&port_id, tx);
    });

    'main: loop {
        trace!("Enter main loop");

        match rx.try_recv() {
            Ok(msg) => match msg {
                TelemetryMessage::DataSnapshot(snapshot) => {
                    addPressure(&mut data_pressure, snapshot.pressure);
                }
                _ => {}
            },
            Err(TryRecvError::Empty) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(TryRecvError::Disconnected) => {
                panic!("Channel to serial port thread was closed");
            }
        }

        event_loop.needs_update();
        // Handle all events.
        for event in event_loop.next(&mut events_loop) {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = support::convert_event(event.clone(), &app_core.display) {
                app_core.ui.handle_event(event);
            }

            // Break from the loop upon `Escape` or closed window.
            match event.clone() {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    glium::glutin::WindowEvent::CloseRequested
                    | glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        info!("About to render");
        if data_pressure.len() == 0 {
            continue;
        }

        let image_map = app_core.render(&data_pressure);
        info!("Rendered");

        // Draw the `Ui` if it has changed.
        if let Some(primitives) = app_core.ui.draw_if_changed() {
            info!("It has changed");
            app_core
                .gl
                .fill(&app_core.display.0, primitives, &image_map);
            let mut target = app_core.display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            app_core
                .gl
                .draw(&app_core.display.0, &mut target, &image_map)
                .unwrap();
            target.finish().unwrap();
        }
    }
}
