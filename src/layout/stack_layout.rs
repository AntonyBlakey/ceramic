use crate::{
    artist::Artist, commands::Commands, connection::*, layout::*, window_data::WindowData,
};

pub fn new() -> Box<StackLayout> {
    Box::new(StackLayout {})
}

struct StackIndicatorArtist {
    axis: Axis,
    window: xcb::Window,
}

impl Artist for StackIndicatorArtist {
    fn calculate_bounds(&self, _window: xcb::Window) -> Option<Bounds> {
        match xcb::get_geometry(&connection(), self.window).get_reply() {
            Ok(geometry) => Some(match self.axis {
                Axis::X => Bounds::new(geometry.x() - 8, geometry.y(), 4, geometry.height()),
                Axis::Y => Bounds::new(geometry.x(), geometry.y() - 8, geometry.width(), 4),
            }),
            _ => None,
        }
    }

    fn draw(&self, window: xcb::Window) {
        if let Ok(geometry) = xcb::get_geometry(&connection(), window).get_reply() {
            if let Ok(surface) = get_cairo_surface(window) {
                let context = cairo::Context::new(&surface);
                context.set_source_rgb(0.125, 0.375, 0.5);
                context.move_to(0.0, 0.0);
                context.line_to(geometry.width() as f64, 0.0);
                context.line_to(geometry.width() as f64, geometry.height() as f64);
                context.line_to(0.0, geometry.height() as f64);
                context.close_path();
                context.fill();
            }
        }
    }
}

pub struct StackLayout {}

impl Layout for StackLayout {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        if windows.is_empty() {
            return Default::default();
        }

        let axis = rect.size.largest_axis();

        let artists: Vec<Box<Artist>> = vec![Box::new(StackIndicatorArtist {
            window: windows[0].window(),
            axis: axis,
        })];

        let r = match axis {
            Axis::X => Bounds::new(
                rect.origin.x + 8,
                rect.origin.y,
                rect.size.width - 8,
                rect.size.height,
            ),
            Axis::Y => Bounds::new(
                rect.origin.x,
                rect.origin.y + 8,
                rect.size.width,
                rect.size.height - 8,
            ),
        };

        let mut new_windows = windows.to_vec();
        for window in new_windows.iter_mut() {
            window.bounds = r;
        }
        compute_window_order(&mut new_windows);
        (new_windows, artists)
    }
}

impl Commands for StackLayout {}
