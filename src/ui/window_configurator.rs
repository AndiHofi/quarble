use iced_winit::settings::SettingsWindowConfigurator;
use iced_winit::winit::dpi::LogicalSize;
use iced_winit::winit::event_loop::EventLoopWindowTarget;
use iced_winit::winit::window::WindowBuilder;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum DisplaySelection<'a> {
    Largest,
    ByIndex(usize),
    ByName(&'a str),
}

#[derive(Debug)]
pub struct MyWindowConfigurator<'a> {
    pub base: SettingsWindowConfigurator,
    pub display_selection: DisplaySelection<'a>,
}

impl<'a, A> iced_winit::window_configurator::WindowConfigurator<A> for MyWindowConfigurator<'a> {
    fn configure_builder(
        self,
        window_target: &EventLoopWindowTarget<A>,
        window_builder: WindowBuilder,
    ) -> WindowBuilder {
        let window_builder = self.base.configure_builder(window_target, window_builder);
        let window_builder = platform_specific(window_builder);

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
                    .or_else(|| monitors.first())
                    .unwrap()
            }
        };

        let is_wayland = is_wayland(window_target);

        let size: LogicalSize<f64> = monitor.size().to_logical(monitor.scale_factor());
        let window_size = LogicalSize::new((size.width / 2.0).max(800.0).min(2000.0), 300.0);
        if is_wayland {
            window_builder
                .with_resizable(true)
                .with_inner_size(window_size)
                .with_decorations(true)
        } else {
            window_builder
                .with_resizable(true)
                .with_decorations(false)
                .with_inner_size(window_size)
                .with_position(monitor.position())
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
