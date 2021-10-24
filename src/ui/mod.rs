use std::sync::mpsc;

use anyhow::Result;
use orbclient::DisplayInfo;
use orbtk::prelude::*;

pub enum DisplaySelection<'a> {
    Largest,
    ByIndex(usize),
    ByName(&'a str),
}

pub struct Ui {
    init_pos: Rectangle,
}

impl Ui {
    pub fn new(display_selection: DisplaySelection) -> Result<Self> {
        let displays = orbclient::get_display_details().unwrap();
        let display = match display_selection {
            DisplaySelection::ByIndex(index) => {
                if let Some(display) = displays.get(index) {
                    display
                } else {
                    default_display(&displays)?
                }
            }
            DisplaySelection::ByName(name) => {
                if let Some(display) = displays.iter().find(|d| d.name.eq_ignore_ascii_case(name)) {
                    display
                } else {
                    default_display(&displays)?
                }
            }
            DisplaySelection::Largest => default_display(&displays)?,
        };

        Ok(Ui {
            init_pos: Rectangle::new(
                (display.x, display.y),
                Size::new(display.width as f64, display.height as f64 / 2.0),
            ),
        })
    }

    pub fn show(self) {
        Application::from_name("Quarble")
            .window(move |ctx| {
                Window::new()
                    .title("OrbTk - minimal example")
                    .position(self.init_pos.position())
                    .size(self.init_pos.width(), self.init_pos.height())
                    .resizeable(true)
                    .child(TextBlock::new().text("OrbTk").build(ctx))
                    .build(ctx)
            })
            .run();
    }
}

fn default_display(displays: &[DisplayInfo]) -> Result<&DisplayInfo> {
    match displays {
        [] => anyhow::bail!("No displays available"),
        [display] => Ok(display),
        [displays @ ..] => {
            let max_width = displays.iter().map(|d| d.width).max();
            let biggest =
                max_width.and_then(|max_width| displays.iter().find(|d| d.width == max_width));

            Ok(biggest.unwrap_or(&displays[0]))
        }
    }
}
