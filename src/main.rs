mod support;
mod output;
mod logger;

use std::str::FromStr;

use imgui::{ChildWindow, ColorPicker, ColorPreview, ComboBox, Condition, ImString, Window, im_str};
use logger::{VecLogger};
use output::{Output, magic_home::{MagicHome}};

use crate::output::Mode;

const LOGGING_HEIGHT: f32 = 128.0;

fn main() -> anyhow::Result<()> {
    VecLogger::new().init().unwrap();
    let mut magic_home = MagicHome::new("");
    let mut current_mode = 0;
    let modes = vec![im_str!("Instant"), im_str!("Fade")];
    let mut color: [f32; 3] = [0.0; 3];

    let mut address = ImString::with_capacity(22);
    address.push_str("192.168.10.131:5577");

    let system = support::init("LED controller");
    system.main_loop(move |_, ui, window_size| {
        Window::new(im_str!("Controller"))
            .position([0.0, 0.0], Condition::Always)
            .size([window_size[0], window_size[1] - LOGGING_HEIGHT], Condition::Always)
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .build(ui, || {
                ui.text(im_str!("Address"));
                ui.same_line(0.0);
                ui.input_text(im_str!(""), &mut address).build();
                if !magic_home.is_connected() {
                    if ui.button(im_str!("Connect"), [64.0, 32.0]) {
                        magic_home = MagicHome::new(address.to_str());
                        match magic_home.connect() {
                            Ok(()) => {
                                log::info!("Connected")
                            },
                            Err(e) => {
                                log::error!("Error: {}", e)
                            }
                        }
                    }
                } else {
                    ui.text(format!("Status: {:?}", magic_home.state));
                }
                ui.separator();
                if magic_home.is_connected() {
                    ui.text(im_str!("Mode"));
                    ui.same_line(0.0);
                    ComboBox::new(im_str!(" ")).build_simple_string(ui,
                        &mut current_mode,
                        &modes
                    );
                    magic_home.set_mode(Mode::from_str(modes[current_mode].to_str()).unwrap());
                    if ui.button(im_str!("On/Off"), [64.0, 32.0]) {
                        log::info!("Toggling state");
                        magic_home.on_off().unwrap();
                    }
                    if ui.button(im_str!("Red"), [64.0, 32.0]) {
                        magic_home.set_color([255, 0, 0]).unwrap();
                        log::info!("Setting colour to RED")
                    }
                    ui.same_line(0.0);
                    if ui.button(im_str!("Green"), [64.0, 32.0]) {
                        magic_home.set_color([0, 255, 0]).unwrap();
                        log::info!("Setting colour to GREEN")
                    }
                    ui.same_line(0.0);
                    if ui.button(im_str!("Blue"), [64.0, 32.0]) {
                        magic_home.set_color([0, 0, 255]).unwrap();
                        log::info!("Setting colour to BLUE")
                    }
                    ChildWindow::new("color picker").size([256.0, 256.0]).build(&ui, || {
                        if ColorPicker::new(im_str!(""), &mut color).preview(ColorPreview::Opaque).side_preview(false).mode(imgui::ColorPickerMode::HueWheel).build(&ui) {
                            let rgb = [(color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8];
                            log::info!("Setting colour to {:?}", rgb);
                            magic_home.set_color(rgb).unwrap();
                        }
                    });
                }
            });

            Window::new(im_str!("Logging"))
                .position([0.0, window_size[1] - LOGGING_HEIGHT], Condition::Always)
                .size([window_size[0], LOGGING_HEIGHT], Condition::Always)
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .build(ui, || {
                    let log_text = log_text!();
                    let mut log = ImString::with_capacity(log_text.len());
                    log.push_str(&log_text);
                    ui.input_text_multiline(im_str!(""), &mut log, [window_size[0]-16.0, LOGGING_HEIGHT - 16.0])
                        .read_only(true)
                        .no_horizontal_scroll(true)

                        .build();
                });
    });

    Ok(())
}
