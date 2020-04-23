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
use telemetry::{gather_telemetry, structures::{TelemetryMessage, MachineSnapshot}};

use conrod_core::widget::Id as WidgetId;

pub struct App {
    gl: conrod_glium::Renderer,
    display: support::GliumDisplayWinitWrapper,
    ui: conrod_core::Ui,
}

type DataPressure = Vec<(DateTime<Local>, u16)>;

impl App {
    fn render(
        &mut self,
        data_pressure: &DataPressure,
        machine_snapshot: &MachineSnapshot,
        bold_font: conrod_core::text::font::Id,
    ) -> conrod_core::image::Map<glium::texture::Texture2d> {
        let mut buffer = vec![0; (780 * 200 * 4) as usize];
        let root = BitMapBackend::with_buffer(&mut buffer, (780, 200)).into_drawing_area();
        root.fill(&BLACK).unwrap();

        let oldest = data_pressure.first().unwrap().0 - chrono::Duration::seconds(40);
        let newest = data_pressure.first().unwrap().0;

        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .x_label_area_size(10)
            .y_label_area_size(40)
            .build_ranged(oldest..newest, 0..70)
            .unwrap();
        chart
            .configure_mesh()
            .line_style_1(&plotters::style::colors::WHITE.mix(0.5))
            .line_style_2(&plotters::style::colors::BLACK)
            .y_labels(5)
            .y_label_style(plotters::style::TextStyle::from(("sans-serif", 20).into_font()).color(&WHITE))
            .draw().unwrap();
        chart
            .draw_series(LineSeries::new(
                data_pressure.iter().map(|x| (x.0, x.1 as i32)),
                ShapeStyle::from(&plotters::style::RGBColor(0, 137, 255)).filled().stroke_width(1),
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
        let image_id = image_map.insert(image_texture);

        // The `WidgetId` for our background and `Image` widgets.
        let ids = Ids::new(self.ui.widget_id_generator());
        let ui = &mut self.ui.set_widgets();
        create_widgets(ui, ids, image_id, (w, h), bold_font, &machine_snapshot);

        image_map
    }
}

fn addPressure(data: &mut DataPressure, new_point: u16) {
    data.insert(0, (Local::now(), new_point / 10));
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

fn addFakeData(data: &mut DataPressure) {
    let mut rng = rand::thread_rng();

    data.insert(0, (Local::now(), rng.gen_range(0, 60)));
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

    let bold_font = ui.fonts.insert_from_file("./NotoSans-Bold.ttf").unwrap();
    ui.fonts.insert_from_file("./NotoSans-Regular.ttf").unwrap();

    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);
    let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();

    // Create a new game and run it.
    let mut app_core = App {
        gl: renderer,
        display,
        ui,
    };

    let mut event_loop = support::EventLoop::new();

    let (tx, rx): (Sender<TelemetryMessage>, Receiver<TelemetryMessage>) =
        std::sync::mpsc::channel();

    std::thread::spawn(move || {
        gather_telemetry(&port_id, tx);
    });

    let mut last_point = Local::now();
    let mut machine_snapshot = MachineSnapshot::default();

    'main: loop {
        // TODO: only update when needed
        event_loop.needs_update();
        let now = Local::now();
        let last = now - last_point;
        if last > chrono::Duration::milliseconds(32) {
            last_point = now;
            addFakeData(&mut data_pressure);
        }
        'thread_rcv: loop {
            match rx.try_recv() {
                Ok(msg) => {
                    match msg {
                        TelemetryMessage::DataSnapshot(snapshot) => {
                            let now = Local::now();
                            let last = now - last_point;
                            if last > chrono::Duration::milliseconds(32) {
                                last_point = now;
                                addPressure(&mut data_pressure, snapshot.pressure);
                            }
                        }
                        TelemetryMessage::MachineStateSnapshot(snapshot) => {
                            machine_snapshot = snapshot;
                        }
                        _ => {}
                    }
                },
                Err(TryRecvError::Empty) => {
                    break 'thread_rcv;
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("Channel to serial port thread was closed");
                }
            }
        }

        // Handle all events.
        for event in event_loop.next(&mut events_loop) {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = support::convert_event(event.clone(), &app_core.display) {
                app_core.ui.handle_event(event);
                event_loop.needs_update();
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

        if data_pressure.len() == 0 {
            continue;
        }

        let image_map = app_core.render(&data_pressure, &machine_snapshot, bold_font);

        // Draw the `Ui` if it has changed.
        if let Some(primitives) = app_core.ui.draw_if_changed() {
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

fn create_widgets(ui: &mut conrod_core::UiCell,
    ids: Ids,
    image: conrod_core::image::Id,
    (w, h): (u32, u32),
    bold_font: conrod_core::text::font::Id,
    machine_snapshot: &MachineSnapshot
) {
    widget::Canvas::new()
        .color(color::BLACK)
        .set(ids.background, ui);
    // Instantiate the `Image` at its full size in the middle of the window.
    widget::Image::new(image)
        .w_h(w as f64, h as f64)
        .mid_top_with_margin(10.0)
        .set(ids.pressure_graph, ui);

    let mut cycles_position = 0.0;

    cycles_position = create_widget(ui,
        "P(peak)",
        &format!("{} <- ({})", machine_snapshot.previous_peak_pressure, machine_snapshot.peak_command),
        "cmH20",
        bold_font,
        (
            ids.peak_parent,
            ids.peak_title,
            ids.peak_value,
            ids.peak_unit
        ),
       cycles_position,
       10.0,
       conrod_core::color::Color::Rgba(39.0 / 255.0, 66.0 / 255.0, 100.0 / 255.0, 1.0)
    );

    cycles_position = create_widget(ui,
        "P(plateau)",
        &format!("{} <- ({})", machine_snapshot.previous_plateau_pressure, machine_snapshot.plateau_command),
        "cmH20",
        bold_font,
        (
            ids.plateau_parent,
            ids.plateau_title,
            ids.plateau_value,
            ids.plateau_unit
        ),
        cycles_position,
        0.0,
        conrod_core::color::Color::Rgba(66.0 / 255.0, 44.0 / 255.0, 85.0 / 255.0, 1.0)
    );

    cycles_position = create_widget(ui,
        "P(expiratory)",
        &format!("{} <- ({})", machine_snapshot.previous_peep_pressure, machine_snapshot.peep_command),
        "cmH20",
        bold_font,
        (
            ids.peep_parent,
            ids.peep_title,
            ids.peep_value,
            ids.peep_unit
        ),
        cycles_position,
        0.0,
        conrod_core::color::Color::Rgba(76.0 / 255.0, 73.0 / 255.0, 25.0 / 255.0, 1.0)
    );

    cycles_position = create_widget(ui,
        "Cycles/minute",
        &format!("{}", machine_snapshot.cycle),
        "/minute",
        bold_font,
        (
            ids.cycles_parent,
            ids.cycles_title,
            ids.cycles_value,
            ids.cycles_unit
        ),
        cycles_position,
        0.0,
        conrod_core::color::Color::Rgba(47.0 / 255.0, 74.0 / 255.0, 16.0 / 255.0, 1.0)
    );
}

type WidgetIds = (WidgetId, WidgetId, WidgetId, WidgetId);

fn create_widget(ui: &mut conrod_core::UiCell, title: &str, value: &str, unit: &str,
    bold_font: conrod_core::text::font::Id,
    ids: WidgetIds,
    x_position: f64, y_position: f64,
    background_color: conrod_core::color::Color) -> f64 {
    let parent = widget::Canvas::new()
        .color(background_color)
        .w_h(120.0, 75.0)
        .bottom_left_with_margins(y_position, x_position + 10.0);

    parent.set(ids.0, ui);

    let position = parent.get_x_position(&ui);

    widget::Text::new(title)
        .color(color::WHITE)
        .top_left_with_margins_on(ids.0, 10.0, 20.0)
        .font_size(12)
        .set(ids.1, ui);

    let mut value_style = conrod_core::widget::primitive::text::Style::default();
    value_style.font_id = Some(Some(bold_font));
    value_style.color = Some(color::WHITE);
    value_style.font_size = Some(14);
    widget::Text::new(value)
        .with_style(value_style)
        .mid_left_with_margin_on(ids.0, 20.0)
        .set(ids.2, ui);

    widget::Text::new(unit)
        .color(color::WHITE)
        .bottom_left_with_margins_on(ids.0, 15.0, 20.0)
        .font_size(10)
        .set(ids.3, ui);

    10.0 + 120.0
}


widget_ids!(struct Ids {
    background,
    pressure_graph,

    cycles_parent,
    cycles_title,
    cycles_value,
    cycles_unit,

    peak_parent,
    peak_title,
    peak_value,
    peak_unit,

    plateau_parent,
    plateau_title,
    plateau_value,
    plateau_unit,

    peep_parent,
    peep_title,
    peep_value,
    peep_unit
});