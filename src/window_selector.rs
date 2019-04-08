use super::{artist, connection::*, layout::*, window_manager};
use std::rc::Rc;

struct WindowSelectorArtist {
    windows: Vec<xcb::Window>,
}

struct Point {
    x: u16,
    y: u16,
}

impl WindowSelectorArtist {
    const FONT_FACE: &'static str = "Noto Sans Mono";
    const FONT_SIZE: u16 = 12;

    const MARGIN: Point = Point { x: 4, y: 4 };
    const LABEL_PADDING: Point = Point { x: 4, y: 1 };
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
        let connection = connection();
        if let Ok(geometry) = xcb::get_geometry(&connection, self.windows[0]).get_reply() {
            if let Ok(surface) = get_cairo_surface(window) {
                let context = cairo::Context::new(&surface);

                self.configure_label_font(&context);
                let font_extents = context.font_extents();
                let line_height = font_extents.height.ceil() as u16;

                let mut label_width = 0;
                let mut name_width = 0;
                for window in &self.windows {
                    self.configure_label_font(&context);
                    let label = get_string_property(*window, *ATOM_CERAMIC_WINDOW_SELECTOR_LABEL);
                    let text_extents = context.text_extents(&label);
                    label_width = label_width.max(text_extents.width.ceil() as u16);
                    self.configure_name_font(&context);
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
                        + self.windows.len() as u16
                            * (Self::LABEL_PADDING.y
                                + line_height
                                + Self::LABEL_PADDING.y
                                + Self::LINE_SPACING)
                        - Self::LINE_SPACING
                        + Self::MARGIN.y,
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
            for window in &self.windows {
                let label = get_string_property(*window, *ATOM_CERAMIC_WINDOW_SELECTOR_LABEL);
                let text_extents = context.text_extents(&label);
                label_width = label_width.max(text_extents.width.ceil() as u16);
            }

            let focused_window = xcb::get_input_focus(&connection()).get_reply();
            {
                let mut top = Self::MARGIN.y;
                let label_left = Self::MARGIN.x;
                let label_right =
                    label_left + Self::LABEL_PADDING.x + label_width + Self::LABEL_PADDING.x;
                let name_left = label_right + Self::LABEL_TO_NAME_GAP;
                for window in &self.windows {
                    let bottom = top + Self::LABEL_PADDING.y + line_height + Self::LABEL_PADDING.y;

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
                        (top + Self::LABEL_PADDING.y) as f64 + ascent,
                    );
                    self.configure_name_font(&context);
                    let name = get_string_property(*window, *ATOM__NET_WM_NAME);
                    context.show_text(&name);

                    context.set_source_rgb(1.0, 1.0, 1.0);
                    context.move_to(
                        (label_left + Self::LABEL_PADDING.x) as f64,
                        (top + Self::LABEL_PADDING.y) as f64 + ascent,
                    );
                    self.configure_label_font(&context);
                    let label = get_string_property(*window, *ATOM_CERAMIC_WINDOW_SELECTOR_LABEL);
                    context.show_text(&label);

                    top = bottom + Self::LINE_SPACING;
                }
            }
        }
    }
}

pub fn add_actions(actions: &mut Vec<Action>) {
    let mut selector_chars = "ASDFGHJKLQWERTYUIOPZXCVBNM1234567890".chars();
    let mut selector_artists: Vec<(LayoutRect, WindowSelectorArtist)> = Vec::new();
    for action in actions.iter() {
        match action {
            Action::Position {
                id,
                rect,
                border_width: _,
                border_color: _,
            } => match selector_chars.next() {
                Some(c) => {
                    let label = format!("{}", c);
                    set_string_property(*id, *ATOM_CERAMIC_WINDOW_SELECTOR_LABEL, &label);
                    match selector_artists
                        .iter_mut()
                        .find(|(r, _)| r.origin.x == rect.origin.x && r.origin.y == rect.origin.y)
                    {
                        Some((_, artist)) => {
                            artist.windows.push(*id);
                        }
                        None => {
                            selector_artists
                                .push((*rect, WindowSelectorArtist { windows: vec![*id] }));
                        }
                    }
                }
                None => {}
            },
            _ => (),
        }
    }

    for (_, artist) in selector_artists {
        actions.push(Action::Draw {
            artist: Rc::new(artist),
        });
    }
}

pub fn run(wm: &mut window_manager::WindowManager) -> Option<xcb::Window> {
    let mut key_press_count = 0;
    let mut selected_window: Option<xcb::Window> = None;
    grab_keyboard();
    allow_events();
    let connection = connection();
    let key_symbols = xcb_util::keysyms::KeySymbols::new(&connection);
    while let Some(e) = connection.wait_for_event() {
        match e.response_type() & 0x7f {
            xcb::KEY_PRESS => {
                let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&e) };
                if key_press_count == 0 {
                    let keycode = press_event.detail();
                    let keysym = key_symbols.get_keysym(keycode, 0);
                    if keysym != xcb::base::NO_SYMBOL {
                        let key_string = unsafe {
                            std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
                                .to_str()
                                .unwrap()
                                .to_uppercase()
                        };
                        selected_window = wm.workspaces[wm.current_workspace]
                            .windows
                            .iter()
                            .find(|w| {
                                get_string_property(w.id, *ATOM_CERAMIC_WINDOW_SELECTOR_LABEL)
                                    == key_string
                            })
                            .map(|w| w.id)
                    }
                } else {
                    selected_window = None;
                }
                key_press_count += 1;
            }
            xcb::KEY_RELEASE => {
                key_press_count -= 1;
                if key_press_count == 0 {
                    break;
                }
            }
            _ => {
                wm.dispatch_wm_event(&e);
            }
        }
        allow_events();
    }
    ungrab_keyboard();
    allow_events();
    selected_window
}

fn grab_keyboard() {
    let connection = connection();
    let screen = connection.get_setup().roots().nth(0).unwrap();
    match xcb::grab_keyboard(
        &connection,
        false,
        screen.root(),
        xcb::CURRENT_TIME,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_SYNC as u8,
    )
    .get_reply()
    {
        Ok(_) => (),
        Err(x) => eprintln!("Failed to grab keyboard: {:?}", x),
    }
    connection.flush();
}

fn ungrab_keyboard() {
    let connection = connection();
    xcb::ungrab_keyboard(&connection, xcb::CURRENT_TIME);
    connection.flush();
}

fn allow_events() {
    let connection = connection();
    xcb::xproto::allow_events(
        &connection,
        xcb::ALLOW_SYNC_KEYBOARD as u8,
        xcb::CURRENT_TIME,
    );
    connection.flush();
}
