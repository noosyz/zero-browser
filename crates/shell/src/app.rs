use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, EntryExt, GtkWindowExt, NotebookExt, WidgetExt,
};
use tao::{event_loop::EventLoop, platform::unix::WindowExtUnix, window::WindowBuilder};
use webkit2gtk::WebViewExt as WebkitWebViewExt;
use wry::{WebContext, WebViewExtUnix};

use crate::tabs::TabManager;
use crate::url_normalize::normalize_url;

pub(crate) struct App {
    pub(crate) _window: tao::window::Window,
    pub(crate) _tab_manager: Rc<RefCell<TabManager>>,
}

impl App {
    pub(crate) fn new(event_loop: &EventLoop<()>, home_url: &str) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_title("Zero Browser")
            .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
            .build(event_loop)?;

        let vbox = window
            .default_vbox()
            .ok_or_else(|| anyhow::anyhow!("tao window has no default vbox"))?;

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let back_btn = gtk::Button::with_label("Back");
        let forward_btn = gtk::Button::with_label("Forward");
        let reload_btn = gtk::Button::with_label("Reload");
        let home_btn = gtk::Button::with_label("Home");
        let new_tab_btn = gtk::Button::with_label("+");
        let entry = gtk::Entry::new();

        hbox.pack_start(&back_btn, false, false, 0);
        hbox.pack_start(&forward_btn, false, false, 0);
        hbox.pack_start(&reload_btn, false, false, 0);
        hbox.pack_start(&home_btn, false, false, 0);
        hbox.pack_start(&new_tab_btn, false, false, 0);
        hbox.pack_start(&entry, true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        hbox.show_all();

        back_btn.set_sensitive(false);
        forward_btn.set_sensitive(false);

        let notebook = gtk::Notebook::new();
        notebook.set_scrollable(true);
        vbox.pack_start(&notebook, true, true, 0);
        notebook.show_all();

        let gtk_window = window.gtk_window().clone();
        let on_empty: Box<dyn Fn()> = {
            let gtk_window = gtk_window.clone();
            Box::new(move || gtk_window.close())
        };

        let web_context = Rc::new(RefCell::new(WebContext::new(None)));

        let tab_manager = Rc::new(RefCell::new(TabManager::new(
            notebook.clone(),
            entry.clone(),
            back_btn.clone(),
            forward_btn.clone(),
            gtk_window,
            on_empty,
            web_context,
        )));

        {
            let tm = Rc::clone(&tab_manager);
            notebook.connect_switch_page(move |_, _, page_num| {
                tm.borrow().update_chrome_for_index(page_num as usize);
            });
        }

        {
            let tm = Rc::clone(&tab_manager);
            entry.connect_activate(move |entry| {
                let url = normalize_url(&entry.text());
                if url.is_empty() {
                    return;
                }
                entry.set_text(&url);
                entry.set_position(-1);
                let webview = tm.borrow().active().map(|t| Rc::clone(&t.webview));
                if let Some(wv) = webview {
                    if let Err(e) = wv.load_url(&url) {
                        eprintln!("load_url({url}) failed: {e}");
                    }
                }
            });
        }

        {
            let tm = Rc::clone(&tab_manager);
            back_btn.connect_clicked(move |_| {
                let webkit = tm.borrow().active().map(|t| t.webview.webview());
                if let Some(w) = webkit {
                    w.go_back();
                }
            });
        }

        {
            let tm = Rc::clone(&tab_manager);
            forward_btn.connect_clicked(move |_| {
                let webkit = tm.borrow().active().map(|t| t.webview.webview());
                if let Some(w) = webkit {
                    w.go_forward();
                }
            });
        }

        {
            let tm = Rc::clone(&tab_manager);
            reload_btn.connect_clicked(move |_| {
                let webkit = tm.borrow().active().map(|t| t.webview.webview());
                if let Some(w) = webkit {
                    w.reload();
                }
            });
        }

        {
            let tm = Rc::clone(&tab_manager);
            let home = home_url.to_string();
            home_btn.connect_clicked(move |_| {
                let webview = tm.borrow().active().map(|t| Rc::clone(&t.webview));
                if let Some(wv) = webview {
                    if let Err(e) = wv.load_url(&home) {
                        eprintln!("load_url({home}) failed: {e}");
                    }
                }
            });
        }

        {
            let tm = Rc::clone(&tab_manager);
            let home = home_url.to_string();
            new_tab_btn.connect_clicked(move |_| {
                if let Err(e) = TabManager::open_tab(&tm, &home) {
                    eprintln!("open_tab failed: {e}");
                }
            });
        }

        TabManager::open_tab(&tab_manager, home_url)?;

        Ok(Self {
            _window: window,
            _tab_manager: tab_manager,
        })
    }
}
