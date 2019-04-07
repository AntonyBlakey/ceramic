use super::{artist, connection::*, layout::*};

pub struct WindowSelectorArtist {
    pub labels: Vec<String>,
    pub windows: Vec<xcb::Window>,
}

struct Point {
    x: u16,
    y: u16,
}

impl WindowSelectorArtist {
    const FONT_FACE: &'static str = "Noto Sans Mono";
    const FONT_SIZE: u16 = 12;

    const MARGIN: Point = Point { x: 6, y: 2 };
    const LABEL_PADDING: Point = Point { x: 4, y: 2 };
    const LABEL_TO_NAME_GAP: u16 = 6;
    const LINE_SPACING: u16 = 2;

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

impl artist::Artist for WindowSelectorArtist {
    fn calculate_bounds(&self, window: xcb::Window) -> Option<LayoutRect> {
        match xcb::get_geometry(&connection(), self.windows[0]).get_reply() {
            Ok(geometry) => {
                if let Ok(surface) = get_cairo_surface(window) {
                    let context = cairo::Context::new(&surface);

                    self.configure_label_font(&context);
                    let font_extents = context.font_extents();
                    let line_height = font_extents.height.ceil() as u16;

                    let mut label_width = 0;
                    for label in &self.labels {
                        let text_extents = context.text_extents(label);
                        label_width = label_width.max(text_extents.width.ceil() as u16);
                    }

                    self.configure_name_font(&context);
                    let mut name_width = 0;
                    for window in &self.windows {
                        let name = get_string_property(*window, *ATOM__NET_WM_NAME);
                        let text_extents = context.text_extents(&name);
                        name_width = name_width.max(text_extents.width.ceil() as u16);
                    }

                    return Some(euclid::rect(
                        geometry.x() as u16,
                        geometry.y() as u16,
                        Self::MARGIN.x
                            + Self::LABEL_PADDING.x
                            + label_width
                            + Self::LABEL_PADDING.x
                            + Self::LABEL_TO_NAME_GAP
                            + name_width
                            + Self::MARGIN.x,
                        Self::MARGIN.y
                            + self.labels.len() as u16
                                * (Self::LABEL_PADDING.y
                                    + line_height
                                    + Self::LABEL_PADDING.y
                                    + Self::LINE_SPACING)
                            - Self::LINE_SPACING
                            + Self::MARGIN.y,
                    ));
                }
            }
            _ => {}
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
            for label in &self.labels {
                let text_extents = context.text_extents(label);
                label_width = label_width.max(text_extents.width.ceil() as u16);
            }

            {
                let mut top = Self::MARGIN.y;
                let left = Self::MARGIN.x;
                let right = left + Self::LABEL_PADDING.x + label_width + Self::LABEL_PADDING.x;
                for label in &self.labels {
                    let bottom = top + Self::LABEL_PADDING.y + line_height + Self::LABEL_PADDING.y;

                    context.set_source_rgb(0.4, 0.0, 0.0);
                    context.move_to(left as f64, top as f64);
                    context.line_to(right as f64, top as f64);
                    context.line_to(right as f64, bottom as f64);
                    context.line_to(left as f64, bottom as f64);
                    context.close_path();
                    context.fill();

                    context.set_source_rgb(1.0, 1.0, 1.0);
                    context.move_to(
                        (left + Self::LABEL_PADDING.x) as f64,
                        (top + Self::LABEL_PADDING.y) as f64 + ascent,
                    );
                    context.show_text(label);

                    top = bottom + Self::LINE_SPACING;
                }
            }

            {
                self.configure_name_font(&context);
                let mut top = Self::MARGIN.y;
                let left = Self::MARGIN.x
                    + Self::LABEL_PADDING.x
                    + label_width
                    + Self::LABEL_PADDING.x
                    + Self::LABEL_TO_NAME_GAP;
                context.set_source_rgb(0.2, 0.2, 0.5);
                for window in &self.windows {
                    let bottom = top + Self::LABEL_PADDING.y + line_height + Self::LABEL_PADDING.y;

                    let name = get_string_property(*window, *ATOM__NET_WM_NAME);
                    context.move_to(left as f64, (top + Self::LABEL_PADDING.y) as f64 + ascent);
                    context.show_text(&name);

                    top = bottom + Self::LINE_SPACING;
                }
            }
        }
    }
}
