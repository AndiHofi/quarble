use iced_winit::winit::dpi::PhysicalSize;
use iced_winit::winit::event_loop::EventLoopWindowTarget;
use iced_winit::winit::window::WindowBuilder;

#[derive(Debug, Copy, Clone)]
pub enum DisplaySelection<'a> {
    Largest,
    ByIndex(usize),
    ByName(&'a str),
}

#[derive(Debug)]
pub struct MyWindowConfigurator<'a> {
    pub display_selection: DisplaySelection<'a>,
}

impl<'a, A> iced_winit::settings::WindowConfigurator<A> for MyWindowConfigurator<'a> {
    fn configure_builder(
        &self,
        window_target: &EventLoopWindowTarget<A>,
        window_builder: WindowBuilder,
    ) -> WindowBuilder {
        let monitors: Vec<_> = window_target.available_monitors().collect();
        let monitor = match &self.display_selection {
            DisplaySelection::Largest => monitors
                .iter()
                .max_by_key(|m| m.size().width * m.size().height)
                .unwrap(),
            DisplaySelection::ByIndex(index) => monitors.get(index % monitors.len()).unwrap(),
            DisplaySelection::ByName(name) => {
                let name = name.to_lowercase();
                monitors
                    .iter()
                    .find(|m| m.name().map(|n| n.to_lowercase() == name).unwrap_or(false))
                    .or(monitors.first())
                    .unwrap()
            }
        };
        let is_wayland = is_wayland(window_target);

        let size = monitor.size();
        if is_wayland {
            window_builder
                .with_resizable(true)
                .with_inner_size(PhysicalSize::new(
                    (size.width / 2).max(1024),
                    (size.height / 3).max(300),
                ))
                .with_decorations(true)
        } else {
            window_builder
                .with_decorations(false)
                .with_inner_size(PhysicalSize::new(size.width, 100))
        }
    }
}

#[cfg(target_os = "linux")]
fn platform_specific(window_builder: WindowBuilder) -> WindowBuilder {
    use iced_winit::winit::platform::unix::WindowBuilderExtUnix;
    window_builder.with_app_id("quarble".to_string())
}

#[cfg(not(target_os = "linux"))]
fn platform_specific(window_builder: WindowBuilder) -> WindowBuilder {
    window_builder
}

#[cfg(target_os = "linux")]
fn is_wayland<A>(window_target: &EventLoopWindowTarget<A>) -> bool {
    use iced_winit::winit::platform::unix::EventLoopWindowTargetExtUnix;
    window_target.is_wayland()
}

#[cfg(not(target_os = "linux"))]
fn is_wayland<A>(window_target: &EventLoopWindowTarget<A>) -> bool {
    false
}
