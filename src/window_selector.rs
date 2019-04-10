use super::{
    artist::Artist, commands::Commands, connection::*, layout::*, window_data::WindowData,
    window_manager::WindowManager,
};
use std::collections::HashMap;

pub fn add_window_selector_labels<A: Layout>(child: A) -> AddWindowSelectorLabels<A> {
    AddWindowSelectorLabels {
        is_enabled: false,
        child,
    }
}

pub struct AddWindowSelectorLabels<A: Layout> {
    is_enabled: bool,
    child: A,
}

impl<A: Layout> Layout for AddWindowSelectorLabels<A> {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        if !self.is_enabled {
            return self.child.layout(rect, windows);
        }

        let (mut new_windows, mut artists) = self.child.layout(rect, windows);

        // TODO: allow choice of preserve or refresh label assignment policy

        let selector_chars = "ASDFGHJKLQWERTYUIOPZXCVBNM1234567890".chars();
        let mut selector_artists: HashMap<xcb::Window, WindowSelectorArtist> = HashMap::new();
        for (w, c) in new_windows.iter_mut().zip(selector_chars) {
            w.selector_label = format!("{}", c);
            let leader = w.leader_window.unwrap_or(w.id());
            let artist = selector_artists.entry(leader).or_default();
            artist.windows.push((w.selector_label.clone(), w.id()));
        }

        artists.extend(
            selector_artists
                .drain()
                .map(|(_, artist)| Box::new(artist) as Box<Artist>),
        );

        (new_windows, artists)
    }
}

impl<A: Layout> Commands for AddWindowSelectorLabels<A> {
    fn get_commands(&self) -> Vec<String> {
        let mut commands = self.child.get_commands();
        if self.is_enabled {
            commands.push("hide_window_selector_labels".to_owned());
        } else {
            commands.push("show_window_selector_labels".to_owned());
        }
        commands
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        match command {
            "show_window_selector_labels" => {
                self.is_enabled = true;
                Some(Box::new(|wm| wm.update_layout()))
            }
            "hide_window_selector_labels" => {
                self.is_enabled = false;
                Some(Box::new(|wm| wm.update_layout()))
            }
            _ => self.child.execute_command(command, args),
        }
    }
}

#[derive(Default)]
struct WindowSelectorArtist {
    windows: Vec<(String, xcb::Window)>,
}

impl WindowSelectorArtist {
    const FONT_FACE: &'static str = "Noto Sans Mono";
    const FONT_SIZE: u16 = 12;

    const MARGIN: Size = Size::new(6, 4);
    const LABEL_PADDING: Size = Size::new(4, 1);
    const LABEL_TO_NAME_GAP: u16 = 6;
    const LINE_SPACING: u16 = 3;

    fn configure_label_font(&self, context: &cairo::Context) {
        context.select_font_face(
            Self::FONT_FACE,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Bold,
        );
        context.set_font_size(Self::FONT_SIZE as f64);
    }

    fn configure_name_font(&self, context: &cairo::Context) {
        context.select_font_face(
            Self::FONT_FACE,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        context.set_font_size(Self::FONT_SIZE as f64);
    }
}

impl Artist for WindowSelectorArtist {
    fn calculate_bounds(&self, window: xcb::Window) -> Option<Bounds> {
        let connection = connection();
        if let Ok(geometry) = xcb::get_geometry(&connection, self.windows[0].1).get_reply() {
            if let Ok(surface) = get_cairo_surface(window) {
                let context = cairo::Context::new(&surface);

                self.configure_label_font(&context);
                let font_extents = context.font_extents();
                let line_height = font_extents.height.ceil() as u16;

                let mut label_width = 0;
                let mut name_width = 0;
                for (label, window) in &self.windows {
                    self.configure_label_font(&context);
                    let text_extents = context.text_extents(&label);
                    label_width = label_width.max(text_extents.width.ceil() as u16);
                    self.configure_name_font(&context);
                    let name = get_string_property(*window, *ATOM__NET_WM_NAME);
                    let text_extents = context.text_extents(&name);
                    name_width = name_width.max(text_extents.width.ceil() as u16);
                }

                return Some(Bounds::new(
                    geometry.x(),
                    geometry.y(),
                    Self::MARGIN.width
                        + Self::LABEL_PADDING.width
                        + label_width
                        + Self::LABEL_PADDING.width
                        + Self::LABEL_TO_NAME_GAP
                        + name_width
                        + Self::MARGIN.width,
                    Self::MARGIN.height
                        + self.windows.len() as u16
                            * (Self::LABEL_PADDING.height
                                + line_height
                                + Self::LABEL_PADDING.height
                                + Self::LINE_SPACING)
                        - Self::LINE_SPACING
                        + Self::MARGIN.height,
                ));
            }
        }

        None
    }

    fn draw(&self, window: xcb::Window) {
        if let Ok(surface) = get_cairo_surface(window) {
            let context = cairo::Context::new(&surface);

            self.configure_label_font(&context);
            let font_extents = context.font_extents();
            let line_height = font_extents.height.ceil() as u16;
            let ascent = font_extents.ascent;

            let mut label_width = 0;
            for (label, window) in &self.windows {
                let text_extents = context.text_extents(&label);
                label_width = label_width.max(text_extents.width.ceil() as u16);
            }

            let focused_window = xcb::get_input_focus(&connection()).get_reply();
            {
                let mut top = Self::MARGIN.height;
                let label_left = Self::MARGIN.width;
                let label_right = label_left
                    + Self::LABEL_PADDING.width
                    + label_width
                    + Self::LABEL_PADDING.width;
                let name_left = label_right + Self::LABEL_TO_NAME_GAP;
                for (label, window) in &self.windows {
                    let bottom =
                        top + Self::LABEL_PADDING.height + line_height + Self::LABEL_PADDING.height;

                    match &focused_window {
                        Ok(w) if w.focus() == *window => context.set_source_rgb(0.0, 0.6, 0.0),
                        _ => context.set_source_rgb(0.0, 0.3, 0.6),
                    }

                    context.move_to(label_left as f64, top as f64);
                    context.line_to(label_right as f64, top as f64);
                    context.line_to(label_right as f64, bottom as f64);
                    context.line_to(label_left as f64, bottom as f64);
                    context.close_path();
                    context.fill();

                    context.move_to(
                        name_left as f64,
                        (top + Self::LABEL_PADDING.height) as f64 + ascent,
                    );
                    self.configure_name_font(&context);
                    let name = get_string_property(*window, *ATOM__NET_WM_NAME);
                    context.show_text(&name);

                    context.set_source_rgb(1.0, 1.0, 1.0);
                    context.move_to(
                        (label_left + Self::LABEL_PADDING.width) as f64,
                        (top + Self::LABEL_PADDING.height) as f64 + ascent,
                    );
                    self.configure_label_font(&context);
                    context.show_text(&label);

                    top = bottom + Self::LINE_SPACING;
                }
            }
        }
    }
}
