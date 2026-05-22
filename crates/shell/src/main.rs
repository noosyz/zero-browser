use anyhow::Result;
use gtk::prelude::CssProviderExt;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod tabs;
mod url_normalize;

fn install_chrome_font() {
    let css = gtk::CssProvider::new();
    if css
        .load_from_data(b"* { font-family: \"Cascadia Code\", monospace; }")
        .is_err()
    {
        return;
    }
    if let Some(screen) = gtk::gdk::Screen::default() {
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

const HOME_URL: &str = "zero://newtab/";

fn main() -> Result<()> {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("GDK_BACKEND", "x11");

    let event_loop = EventLoop::new();

    install_chrome_font();

    let _app = app::App::new(&event_loop, HOME_URL)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
