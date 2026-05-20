use std::rc::Rc;

use anyhow::Result;
use gtk::prelude::{BoxExt, EditableExt, EntryExt, GtkWindowExt, WidgetExt};
use tao::{event_loop::EventLoop, platform::unix::WindowExtUnix, window::WindowBuilder};
use wry::{PageLoadEvent, WebViewBuilder, WebViewBuilderExtUnix};

pub(crate) struct App {
    pub(crate) _window: tao::window::Window,
    pub(crate) _entry: gtk::Entry,
}

impl App {
    pub(crate) fn new(event_loop: &EventLoop<()>, initial_url: &str) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_title("Zero Browser")
            .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
            .build(event_loop)?;

        let vbox = window
            .default_vbox()
            .ok_or_else(|| anyhow::anyhow!("tao window has no default vbox"))?;

        let entry = gtk::Entry::new();
        entry.set_text(initial_url);
        vbox.pack_start(&entry, false, false, 0);
        entry.show();

        let gtk_window = window.gtk_window().clone();
        let entry_for_title = entry.clone();
        let entry_for_load = entry.clone();

        let webview = WebViewBuilder::new_gtk(vbox)
            .with_url(initial_url)
            .with_document_title_changed_handler(move |title| {
                gtk_window.set_title(&format!("{title} — Zero Browser"));
                entry_for_title.set_position(-1);
            })
            .with_on_page_load_handler(move |event, url| {
                if matches!(event, PageLoadEvent::Finished) {
                    entry_for_load.set_text(&url);
                }
            })
            .build()?;

        let webview = Rc::new(webview);

        let webview_for_entry = Rc::clone(&webview);
        entry.connect_activate(move |entry| {
            let url = entry.text().to_string();
            if let Err(e) = webview_for_entry.load_url(&url) {
                eprintln!("load_url({url}) failed: {e}");
            }
        });

        Ok(Self {
            _window: window,
            _entry: entry,
        })
    }
}
