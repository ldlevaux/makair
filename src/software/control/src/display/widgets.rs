// MakAir
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use super::fonts::Fonts;
use conrod_core::widget::Id as WidgetId;
use conrod_core::{
    color::{self, Color},
    widget, Colorable, Positionable, Sizeable, Widget,
};
use telemetry::structures::MachineStateSnapshot;

pub type WidgetIds = (WidgetId, WidgetId, WidgetId, WidgetId);

widget_ids!(pub struct Ids {
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

pub struct WidgetConfig<'a> {
    fonts: &'a Fonts,
    title: &'a str,
    value: String,
    unit: &'a str,
    ids: WidgetIds,
    x_position: f64,
    y_position: f64,
    background_color: Color,
}

pub fn create_widgets<'a>(
    mut ui: conrod_core::UiCell,
    ids: Ids,
    image: conrod_core::image::Id,
    (w, h): (u32, u32),
    fonts: &'a Fonts,
    machine_snapshot: &MachineStateSnapshot,
) {
    widget::Canvas::new()
        .color(color::BLACK)
        .set(ids.background, &mut ui);
    //Instantiate the `Image` at its full size in the middle of the window.
    widget::Image::new(image)
        .w_h(w as f64, h as f64)
        .mid_top_with_margin(10.0)
        .set(ids.pressure_graph, &mut ui);

    let mut cycles_position = 0.0;

    let peak_config = WidgetConfig {
        title: "P(peak)",
        value: format!(
            "{} <- ({})",
            (machine_snapshot.previous_peak_pressure as f64 / 10.0).round(),
            machine_snapshot.peak_command
        ),
        unit: "cmH20",
        ids: (
            ids.peak_parent,
            ids.peak_title,
            ids.peak_value,
            ids.peak_unit,
        ),
        x_position: cycles_position,
        y_position: 10.0,
        background_color: Color::Rgba(39.0 / 255.0, 66.0 / 255.0, 100.0 / 255.0, 1.0),
        fonts,
    };

    cycles_position = create_widget(&mut ui, peak_config);

    let plateau_config = WidgetConfig {
        title: "P(plateau)",
        value: format!(
            "{} <- ({})",
            (machine_snapshot.previous_plateau_pressure as f64 / 10.0).round(),
            machine_snapshot.plateau_command
        ),
        unit: "cmH20",
        ids: (
            ids.plateau_parent,
            ids.plateau_title,
            ids.plateau_value,
            ids.plateau_unit,
        ),
        x_position: cycles_position,
        y_position: 0.0,
        background_color: Color::Rgba(66.0 / 255.0, 44.0 / 255.0, 85.0 / 255.0, 1.0),
        fonts,
    };

    cycles_position = create_widget(&mut ui, plateau_config);

    let peep_config = WidgetConfig {
        title: "P(expiratory)",
        value: format!(
            "{} <- ({})",
            (machine_snapshot.previous_peep_pressure as f64 / 10.0).round(),
            machine_snapshot.peep_command
        ),
        unit: "cmH20",
        ids: (
            ids.peep_parent,
            ids.peep_title,
            ids.peep_value,
            ids.peep_unit,
        ),
        x_position: cycles_position,
        y_position: 0.0,
        background_color: Color::Rgba(76.0 / 255.0, 73.0 / 255.0, 25.0 / 255.0, 1.0),
        fonts,
    };

    cycles_position = create_widget(&mut ui, peep_config);

    let cycles_config = WidgetConfig {
        title: "Cycles/minute",
        value: format!("{}", machine_snapshot.cpm_command),
        unit: "/minute",
        ids: (
            ids.cycles_parent,
            ids.cycles_title,
            ids.cycles_value,
            ids.cycles_unit,
        ),
        x_position: cycles_position,
        y_position: 0.0,
        background_color: Color::Rgba(47.0 / 255.0, 74.0 / 255.0, 16.0 / 255.0, 1.0),
        fonts,
    };

    create_widget(&mut ui, cycles_config);
}

pub fn create_widget(ui: &mut conrod_core::UiCell, config: WidgetConfig) -> f64 {
    let parent = widget::Canvas::new()
        .color(config.background_color)
        .w_h(120.0, 75.0)
        .bottom_left_with_margins(config.y_position, config.x_position + 10.0);

    parent.set(config.ids.0, ui);

    widget::Text::new(config.title)
        .color(color::WHITE)
        .top_left_with_margins_on(config.ids.0, 10.0, 20.0)
        .font_size(12)
        .set(config.ids.1, ui);

    let mut value_style = conrod_core::widget::primitive::text::Style::default();
    value_style.font_id = Some(Some(config.fonts.bold));
    value_style.color = Some(color::WHITE);
    value_style.font_size = Some(14);
    widget::Text::new(&config.value)
        .with_style(value_style)
        .mid_left_with_margin_on(config.ids.0, 20.0)
        .set(config.ids.2, ui);

    widget::Text::new(config.unit)
        .color(color::WHITE)
        .bottom_left_with_margins_on(config.ids.0, 15.0, 20.0)
        .font_size(10)
        .set(config.ids.3, ui);

    10.0 + 120.0
}
