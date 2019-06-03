use cairo::XCBSurface;
use lazy_static::lazy_static;

pub fn connection() -> &'static xcb::Connection {
    static mut CONNECTION: Option<xcb::Connection> = None;
    unsafe {
        CONNECTION.get_or_insert_with(|| {
            let (connection, _screen_number) = xcb::Connection::connect(None).unwrap();
            connection
        })
    }
}

pub fn get_atom(name: &str) -> u32 {
    xcb::intern_atom(connection(), false, name)
        .get_reply()
        .unwrap()
        .atom()
}

lazy_static! {
    pub static ref ATOM_UTF8_STRING: u32 = get_atom("UTF8_STRING");
    //
    pub static ref ATOM__NET_WM_NAME: u32 = get_atom("_NET_WM_NAME");
    pub static ref ATOM__NET_SUPPORTED: u32 = get_atom("_NET_SUPPORTED");
    pub static ref ATOM__NET_SUPPORTING_WM_CHECK: u32 = get_atom("_NET_SUPPORTING_WM_CHECK");
    pub static ref ATOM__NET_ACTIVE_WINDOW: u32 = get_atom("_NET_ACTIVE_WINDOW");
    pub static ref ATOM__NET_NUMBER_OF_DESKTOPS: u32 = get_atom("_NET_NUMBER_OF_DESKTOPS");
    pub static ref ATOM__NET_DESKTOP_NAMES: u32 = get_atom("_NET_DESKTOP_NAMES");
    pub static ref ATOM__NET_CURRENT_DESKTOP: u32 = get_atom("_NET_CURRENT_DESKTOP");
    pub static ref ATOM__NET_WM_STRUT: u32 = get_atom("_NET_WM_STRUT");
    pub static ref ATOM__NET_WM_DESKTOP: u32 = get_atom("_NET_WM_DESKTOP");
    //
    pub static ref ATOM__NET_WM_WINDOW_TYPE: u32 = get_atom("_NET_WM_WINDOW_TYPE");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_DESKTOP: u32 = get_atom("_NET_WM_WINDOW_TYPE_DESKTOP");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_DOCK: u32 = get_atom("_NET_WM_WINDOW_TYPE_DOCK");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_TOOLBAR: u32 = get_atom("_NET_WM_WINDOW_TYPE_TOOLBAR");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_MENU: u32 = get_atom("_NET_WM_WINDOW_TYPE_MENU");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_UTILITY: u32 = get_atom("_NET_WM_WINDOW_TYPE_UTILITY");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_SPLASH: u32 = get_atom("_NET_WM_WINDOW_TYPE_SPLASH");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_DIALOG: u32 = get_atom("_NET_WM_WINDOW_TYPE_DIALOG");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_DROPDOWN_MENU: u32 =
        get_atom("_NET_WM_WINDOW_TYPE_DROPDOWN_MENU");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_POPUP_MENU: u32 =
        get_atom("_NET_WM_WINDOW_TYPE_POPUP_MENU");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_TOOLTIP: u32 = get_atom("_NET_WM_WINDOW_TYPE_TOOLTIP");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_NOTIFICATION: u32 =
        get_atom("_NET_WM_WINDOW_TYPE_NOTIFICATION");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_COMBO: u32 = get_atom("_NET_WM_WINDOW_TYPE_COMBO");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_DND: u32 = get_atom("_NET_WM_WINDOW_TYPE_DND");
    pub static ref ATOM__NET_WM_WINDOW_TYPE_NORMAL: u32 = get_atom("_NET_WM_WINDOW_TYPE_NORMAL");
    //
    pub static ref ATOM__NET_WM_STATE: u32 = get_atom("_NET_WM_STATE");
    pub static ref ATOM__NET_WM_STATE_MODAL: u32 = get_atom("_NET_WM_STATE_MODAL");
    pub static ref ATOM__NET_WM_STATE_STICKY: u32 = get_atom("_NET_WM_STATE_STICKY");
    pub static ref ATOM__NET_WM_STATE_MAXIMIZED_VERT: u32 =
        get_atom("_NET_WM_STATE_MAXIMIZED_VERT");
    pub static ref ATOM__NET_WM_STATE_MAXIMIZED_HORZ: u32 =
        get_atom("_NET_WM_STATE_MAXIMIZED_HORZ");
    pub static ref ATOM__NET_WM_STATE_SHADED: u32 = get_atom("_NET_WM_STATE_SHADED");
    pub static ref ATOM__NET_WM_STATE_SKIP_TASKBAR: u32 = get_atom("_NET_WM_STATE_SKIP_TASKBAR");
    pub static ref ATOM__NET_WM_STATE_SKIP_PAGER: u32 = get_atom("_NET_WM_STATE_SKIP_PAGER");
    pub static ref ATOM__NET_WM_STATE_HIDDEN: u32 = get_atom("_NET_WM_STATE_HIDDEN");
    pub static ref ATOM__NET_WM_STATE_FULLSCREEN: u32 = get_atom("_NET_WM_STATE_FULLSCREEN");
    pub static ref ATOM__NET_WM_STATE_ABOVE: u32 = get_atom("_NET_WM_STATE_ABOVE");
    pub static ref ATOM__NET_WM_STATE_BELOW: u32 = get_atom("_NET_WM_STATE_BELOW");
    pub static ref ATOM__NET_WM_STATE_DEMANDS_ATTENTION: u32 =
        get_atom("_NET_WM_STATE_DEMANDS_ATTENTION");
    //
    pub static ref ATOM__NET_WM_ALLOWED_ACTIONS: u32 = get_atom("_NET_WM_ALLOWED_ACTIONS");
    pub static ref ATOM__NET_WM_ACTION_MOVE: u32 = get_atom("_NET_WM_ACTION_MOVE");
    pub static ref ATOM__NET_WM_ACTION_RESIZE: u32 = get_atom("_NET_WM_ACTION_RESIZE");
    pub static ref ATOM__NET_WM_ACTION_MINIMIZE: u32 = get_atom("_NET_WM_ACTION_MINIMIZE");
    pub static ref ATOM__NET_WM_ACTION_SHADE: u32 = get_atom("_NET_WM_ACTION_SHADE");
    pub static ref ATOM__NET_WM_ACTION_STICK: u32 = get_atom("_NET_WM_ACTION_STICK");
    pub static ref ATOM__NET_WM_ACTION_MAXIMIZE_HORZ: u32 =
        get_atom("_NET_WM_ACTION_MAXIMIZE_HORZ");
    pub static ref ATOM__NET_WM_ACTION_MAXIMIZE_VERT: u32 =
        get_atom("_NET_WM_ACTION_MAXIMIZE_VERT");
    pub static ref ATOM__NET_WM_ACTION_FULLSCREEN: u32 = get_atom("_NET_WM_ACTION_FULLSCREEN");
    pub static ref ATOM__NET_WM_ACTION_CHANGE_DESKTOP: u32 =
        get_atom("_NET_WM_ACTION_CHANGE_DESKTOP");
    pub static ref ATOM__NET_WM_ACTION_CLOSE: u32 = get_atom("_NET_WM_ACTION_CLOSE");
    pub static ref ATOM__NET_WM_ACTION_ABOVE: u32 = get_atom("_NET_WM_ACTION_ABOVE");
    pub static ref ATOM__NET_WM_ACTION_BELOW: u32 = get_atom("_NET_WM_ACTION_BELOW");
    //
    pub static ref ATOM_CERAMIC_COMMAND: u32 = get_atom("CERAMIC_COMMAND");
    pub static ref ATOM_CERAMIC_AVAILABLE_COMMANDS: u32 = get_atom("CERAMIC_AVAILABLE_COMMANDS");
    pub static ref ATOM_CERAMIC_SELECTOR_LABEL: u32 = get_atom("CERAMIC_SELECTOR_LABEL");
}

pub fn set_cardinal_property(window: xcb::Window, name_atom: u32, value: u32) {
    set_cardinals_property(window, name_atom, &[value]);
}

pub fn set_cardinals_property(window: xcb::Window, name_atom: u32, values: &[u32]) {
    xcb::change_property(
        connection(),
        xcb::PROP_MODE_REPLACE as u8,
        window,
        name_atom,
        xcb::ATOM_CARDINAL,
        32,
        values,
    );
}

pub fn set_string_property(window: xcb::Window, name_atom: u32, value: &str) {
    xcb::change_property(
        connection(),
        xcb::PROP_MODE_REPLACE as u8,
        window,
        name_atom,
        *ATOM_UTF8_STRING,
        8,
        value.as_bytes(),
    );
}

pub fn set_strings_property(window: xcb::Window, name_atom: u32, values: &[&str]) {
    xcb::change_property(
        connection(),
        xcb::PROP_MODE_REPLACE as u8,
        window,
        name_atom,
        *ATOM_UTF8_STRING,
        8,
        values
            .iter()
            .fold(String::from(""), |accum, value| {
                format!("{}{}\0", accum, value)
            })
            .as_bytes(),
    );
}

pub fn set_window_property(window: xcb::Window, name_atom: u32, value: xcb::Window) {
    set_windows_property(window, name_atom, &[value]);
}

pub fn set_windows_property(window: xcb::Window, name_atom: u32, values: &[xcb::Window]) {
    xcb::change_property(
        connection(),
        xcb::PROP_MODE_REPLACE as u8,
        window,
        name_atom,
        xcb::ATOM_WINDOW,
        32,
        values,
    );
}

pub fn set_atom_property(window: xcb::Window, name_atom: u32, value: u32) {
    set_atoms_property(window, name_atom, &[value]);
}

pub fn set_atoms_property(window: xcb::Window, name_atom: u32, values: &[u32]) {
    xcb::change_property(
        connection(),
        xcb::PROP_MODE_REPLACE as u8,
        window,
        name_atom,
        xcb::ATOM_ATOM,
        32,
        values,
    );
}

pub fn get_cardinal_property(window: xcb::Window, name_atom: u32) -> Option<u32> {
    let result = get_cardinals_property(window, name_atom);
    if result.is_empty() {
        None
    } else {
        Some(result[0])
    }
}

pub fn get_cardinals_property(window: xcb::Window, name_atom: u32) -> Vec<u32> {
    // TODO: handle case where property is bigger than we allowed for
    xcb::get_property(
        connection(),
        false,
        window,
        name_atom,
        xcb::ATOM_CARDINAL,
        0,
        32,
    )
    .get_reply()
    .map(|reply| reply.value().to_vec())
    .unwrap_or_default()
}

pub fn get_ascii_string_property(window: xcb::Window, name_atom: u32) -> String {
    // TODO: find a better method to go from ascii (latin-1?) to utf-8.
    xcb::get_property(
        connection(),
        false,
        window,
        name_atom,
        xcb::ATOM_STRING,
        0,
        1024,
    )
    .get_reply()
    .map(|reply| String::from_utf8(reply.value().to_vec()).unwrap_or_default())
    .unwrap_or_default()
}

pub fn get_string_property(window: xcb::Window, name_atom: u32) -> String {
    xcb::get_property(
        connection(),
        false,
        window,
        name_atom,
        *ATOM_UTF8_STRING,
        0,
        1024,
    )
    .get_reply()
    .map(|reply| String::from_utf8(reply.value().to_vec()).unwrap_or_default())
    .unwrap_or_default()
}

pub fn get_ascii_strings_property(window: xcb::Window, name_atom: u32) -> Vec<String> {
    // TODO: handle case where property is bigger than we allowed for
    // TODO: find a better method to go from ascii (latin-1?) to utf-8.
    xcb::get_property(
        connection(),
        false,
        window,
        name_atom,
        xcb::ATOM_STRING,
        0,
        1024,
    )
    .get_reply()
    .map(|reply| {
        String::from_utf8(reply.value().to_vec())
            .unwrap_or_default()
            .trim_matches('\0')
            .split("\0")
            .map(|s| String::from(s))
            .collect()
    })
    .unwrap_or_default()
}

pub fn get_strings_property(window: xcb::Window, name_atom: u32) -> Vec<String> {
    // TODO: handle case where property is bigger than we allowed for
    xcb::get_property(
        connection(),
        false,
        window,
        name_atom,
        *ATOM_UTF8_STRING,
        0,
        1024,
    )
    .get_reply()
    .map(|reply| {
        String::from_utf8(reply.value().to_vec())
            .unwrap_or_default()
            .trim_matches('\0')
            .split("\0")
            .map(|s| String::from(s))
            .collect()
    })
    .unwrap_or_default()
}

pub fn get_window_property(window: xcb::Window, name_atom: u32) -> Option<xcb::Window> {
    let result = get_windows_property(window, name_atom);
    if result.is_empty() {
        None
    } else {
        Some(result[0])
    }
}

pub fn get_windows_property(window: xcb::Window, name_atom: u32) -> Vec<xcb::Window> {
    // TODO: handle case where property is bigger than we allowed for
    xcb::get_property(
        connection(),
        false,
        window,
        name_atom,
        xcb::ATOM_WINDOW,
        0,
        32,
    )
    .get_reply()
    .map(|reply| reply.value().to_vec())
    .unwrap_or_default()
}

pub fn get_atom_property(window: xcb::Window, name_atom: u32) -> Option<u32> {
    let result = get_atoms_property(window, name_atom);
    if result.is_empty() {
        None
    } else {
        Some(result[0])
    }
}

pub fn get_atoms_property(window: xcb::Window, name_atom: u32) -> Vec<u32> {
    // TODO: handle case where property is bigger than we allowed for
    xcb::get_property(
        connection(),
        false,
        window,
        name_atom,
        xcb::ATOM_ATOM,
        0,
        32,
    )
    .get_reply()
    .map(|reply| reply.value().to_vec())
    .unwrap_or_default()
}

pub fn grab_keyboard() {
    let root = connection().get_setup().roots().nth(0).unwrap().root();
    match xcb::xproto::grab_keyboard(
        connection(),
        false,
        root,
        xcb::CURRENT_TIME,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_SYNC as u8,
    )
    .get_reply()
    .unwrap()
    .status() as u32
    {
        xcb::xproto::GRAB_STATUS_SUCCESS => log::debug!("Grab keyboard: Success"),
        xcb::xproto::GRAB_STATUS_ALREADY_GRABBED => log::debug!("Grab keyboard: Already Grabbed"),
        xcb::xproto::GRAB_STATUS_INVALID_TIME => log::debug!("Grab keyboard: Invalid Time"),
        xcb::xproto::GRAB_STATUS_NOT_VIEWABLE => log::debug!("Grab keyboard: Not Viewable"),
        xcb::xproto::GRAB_STATUS_FROZEN => log::debug!("Grab keyboard: Frozen"),
        x => log::debug!("Grab keyboard: Unknown status: {}", x),
    }
    connection().flush();
}

pub fn ungrab_keyboard() {
    log::debug!("Ungrab keyboard");
    xcb::xproto::ungrab_keyboard(connection(), xcb::CURRENT_TIME);
    connection().flush();
}

pub fn allow_events() {
    xcb::xproto::allow_events(
        connection(),
        xcb::ALLOW_SYNC_KEYBOARD as u8,
        xcb::CURRENT_TIME,
    );
    connection().flush();
}

pub fn wait_for_event() -> Option<xcb::base::GenericEvent> {
    allow_events();
    connection().wait_for_event()
}

pub fn get_cairo_surface(window: xcb::Window) -> Result<cairo::Surface, xcb::GenericError> {
    let connection = connection();

    let geometry = xcb::get_geometry(&connection, window).get_reply()?;
    let cairo_connection = unsafe {
        cairo::XCBConnection::from_raw_none(
            connection.get_raw_conn() as *mut cairo_sys::xcb_connection_t
        )
    };

    let cairo_drawable = cairo::XCBDrawable(window);

    let screen = connection.get_setup().roots().nth(0).unwrap();
    let mut visual = screen
        .allowed_depths()
        .filter(|d| d.depth() == screen.root_depth())
        .flat_map(|d| d.visuals())
        .find(|v| v.visual_id() == screen.root_visual())
        .unwrap();
    let cairo_visualtype = unsafe {
        cairo::XCBVisualType::from_raw_none(
            (&mut visual.base as *mut xcb::ffi::xproto::xcb_visualtype_t)
                as *mut cairo_sys::xcb_visualtype_t,
        )
    };

    Ok(cairo::Surface::create(
        &cairo_connection,
        &cairo_drawable,
        &cairo_visualtype,
        geometry.width() as i32,
        geometry.height() as i32,
    ))
}
