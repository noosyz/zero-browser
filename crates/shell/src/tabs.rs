use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};

use anyhow::Result;
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, EntryExt, GtkWindowExt, LabelExt, NotebookExt,
    NotebookExtManual, ProgressBarExt, WidgetExt,
};
use webkit2gtk::{
    HitTestResultExt, LoadEvent, SettingsExt, UserContentInjectedFrames, UserContentManagerExt,
    UserStyleLevel, UserStyleSheet, WebViewExt as WebkitWebViewExt,
};
use wry::{
    http::{Request, Response},
    PageLoadEvent, WebContext, WebView, WebViewBuilder, WebViewBuilderExtUnix, WebViewExtUnix,
};

const NEWTAB_HTML: &[u8] = include_bytes!("../assets/newtab.html");

fn newtab_handler(_req: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Cow::Borrowed(NEWTAB_HTML))
        .expect("static response is valid")
}

const WEB_FONT_CSS: &str = "* { font-family: 'Cascadia Code', monospace !important; }";

fn apply_font_overrides(webkit: &webkit2gtk::WebView) {
    if let Some(settings) = WebkitWebViewExt::settings(webkit) {
        settings.set_default_font_family("Cascadia Code");
        settings.set_serif_font_family("Cascadia Code");
        settings.set_sans_serif_font_family("Cascadia Code");
        settings.set_monospace_font_family("Cascadia Code");
        settings.set_fantasy_font_family("Cascadia Code");
        settings.set_cursive_font_family("Cascadia Code");
    }
    if let Some(manager) = webkit.user_content_manager() {
        let sheet = UserStyleSheet::new(
            WEB_FONT_CSS,
            UserContentInjectedFrames::AllFrames,
            UserStyleLevel::User,
            &[],
            &[],
        );
        manager.add_style_sheet(&sheet);
    }
}

pub(crate) type TabId = u64;

pub(crate) struct Tab {
    pub(crate) id: TabId,
    pub(crate) webview: Rc<WebView>,
    pub(crate) title_label: gtk::Label,
    pub(crate) title: String,
    pub(crate) url: String,
}

pub(crate) struct TabManager {
    pub(crate) notebook: gtk::Notebook,
    tabs: Vec<Tab>,
    next_id: TabId,
    entry: gtk::Entry,
    back_btn: gtk::Button,
    forward_btn: gtk::Button,
    progress_bar: gtk::ProgressBar,
    status_label: gtk::Label,
    gtk_window: gtk::ApplicationWindow,
    on_empty: Box<dyn Fn()>,
    web_context: Rc<RefCell<WebContext>>,
    zero_registered: Cell<bool>,
    pub(crate) home_url: String,
}

impl TabManager {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        notebook: gtk::Notebook,
        entry: gtk::Entry,
        back_btn: gtk::Button,
        forward_btn: gtk::Button,
        progress_bar: gtk::ProgressBar,
        status_label: gtk::Label,
        gtk_window: gtk::ApplicationWindow,
        on_empty: Box<dyn Fn()>,
        web_context: Rc<RefCell<WebContext>>,
        home_url: String,
    ) -> Self {
        Self {
            notebook,
            tabs: Vec::new(),
            next_id: 0,
            entry,
            back_btn,
            forward_btn,
            progress_bar,
            status_label,
            gtk_window,
            on_empty,
            web_context,
            zero_registered: Cell::new(false),
            home_url,
        }
    }

    pub(crate) fn open_tab(this: &Rc<RefCell<Self>>, url: &str) -> Result<TabId> {
        let id = {
            let mut tm = this.borrow_mut();
            tm.next_id += 1;
            tm.next_id
        };

        let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
        page.show_all();

        let title_label = gtk::Label::new(Some(url));
        title_label.set_max_width_chars(20);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        let close_btn = gtk::Button::with_label("×");
        close_btn.set_relief(gtk::ReliefStyle::None);
        let label_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        label_box.pack_start(&title_label, true, true, 0);
        label_box.pack_start(&close_btn, false, false, 0);
        label_box.show_all();

        {
            let this_clone = Rc::clone(this);
            close_btn.connect_clicked(move |_| {
                TabManager::close_tab(&this_clone, id);
            });
        }

        let title_handler = {
            let this_clone = Rc::clone(this);
            move |title: String| {
                let mut tm = this_clone.borrow_mut();
                if let Some(tab) = tm.tab_by_id_mut(id) {
                    tab.title = title.clone();
                    tab.title_label.set_text(&title);
                }
                if tm.is_active(id) {
                    tm.gtk_window.set_title(&format!("{title} — Zero Browser"));
                }
            }
        };

        let load_handler = {
            let this_clone = Rc::clone(this);
            move |event: PageLoadEvent, loaded_url: String| {
                if !matches!(event, PageLoadEvent::Finished) {
                    return;
                }
                let mut tm = this_clone.borrow_mut();
                if let Some(tab) = tm.tab_by_id_mut(id) {
                    tab.url = loaded_url.clone();
                }
                if tm.is_active(id) {
                    tm.entry.set_text(&loaded_url);
                    tm.entry.set_position(-1);
                    tm.sync_nav_buttons();
                }
            }
        };

        let ctx_rc = this.borrow().web_context.clone();
        let should_register = !this.borrow().zero_registered.get();

        let webview = {
            let mut ctx = ctx_rc.borrow_mut();
            let mut builder = WebViewBuilder::new_gtk(&page)
                .with_url(url)
                .with_document_title_changed_handler(title_handler)
                .with_on_page_load_handler(load_handler)
                .with_web_context(&mut ctx);
            if should_register {
                builder = builder.with_custom_protocol("zero".into(), newtab_handler);
            }
            builder.build()?
        };

        if should_register {
            this.borrow().zero_registered.set(true);
        }

        let webview = Rc::new(webview);

        let webkit = webview.webview();
        apply_font_overrides(&webkit);

        {
            let this_clone = Rc::clone(this);
            let webkit = webkit.clone();
            webkit.connect_load_changed(move |_, event| {
                let tm = this_clone.borrow();
                if !tm.is_active(id) {
                    return;
                }
                tm.sync_nav_buttons();
                match event {
                    LoadEvent::Started => {
                        tm.progress_bar.set_fraction(0.0);
                        tm.progress_bar.set_visible(true);
                    }
                    LoadEvent::Finished => {
                        tm.progress_bar.set_fraction(1.0);
                        tm.progress_bar.set_visible(false);
                    }
                    _ => {}
                }
            });
        }

        {
            let this_clone = Rc::clone(this);
            let webkit = webkit.clone();
            webkit.connect_estimated_load_progress_notify(move |wv| {
                let tm = this_clone.borrow();
                if tm.is_active(id) {
                    tm.progress_bar.set_fraction(wv.estimated_load_progress());
                }
            });
        }

        {
            let this_clone = Rc::clone(this);
            let webkit = webkit.clone();
            webkit.connect_mouse_target_changed(move |_, hit, _| {
                let tm = this_clone.borrow();
                if !tm.is_active(id) {
                    return;
                }
                if hit.context_is_link() {
                    if let Some(uri) = hit.link_uri() {
                        tm.status_label.set_text(&uri);
                        return;
                    }
                }
                tm.status_label.set_text("");
            });
        }

        {
            let mut tm = this.borrow_mut();
            tm.tabs.push(Tab {
                id,
                webview,
                title_label,
                title: url.to_string(),
                url: url.to_string(),
            });
        }

        let notebook = this.borrow().notebook.clone();
        notebook.append_page(&page, Some(&label_box));
        notebook.set_tab_reorderable(&page, false);

        let idx = this.borrow().tabs.len() - 1;
        notebook.set_current_page(Some(idx as u32));

        this.borrow().update_chrome();

        Ok(id)
    }

    pub(crate) fn close_tab(this: &Rc<RefCell<Self>>, id: TabId) {
        let (idx, was_last) = {
            let mut tm = this.borrow_mut();
            let Some(idx) = tm.tabs.iter().position(|t| t.id == id) else {
                return;
            };
            tm.tabs.remove(idx);
            (idx, tm.tabs.is_empty())
        };

        let tm = this.borrow();
        tm.notebook.remove_page(Some(idx as u32));

        if was_last {
            (tm.on_empty)();
        }
    }

    pub(crate) fn active(&self) -> Option<&Tab> {
        let idx = self.notebook.current_page()? as usize;
        self.tabs.get(idx)
    }

    fn tab_by_id_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    fn is_active(&self, id: TabId) -> bool {
        self.active().map(|t| t.id) == Some(id)
    }

    pub(crate) fn update_chrome(&self) {
        self.apply_chrome(self.active());
    }

    pub(crate) fn update_chrome_for_index(&self, idx: usize) {
        self.apply_chrome(self.tabs.get(idx));
    }

    fn apply_chrome(&self, tab: Option<&Tab>) {
        self.status_label.set_text("");
        if let Some(tab) = tab {
            self.entry.set_text(&tab.url);
            self.entry.set_position(-1);
            self.gtk_window
                .set_title(&format!("{} — Zero Browser", tab.title));
            let wk = tab.webview.webview();
            self.back_btn.set_sensitive(wk.can_go_back());
            self.forward_btn.set_sensitive(wk.can_go_forward());
            let fraction = wk.estimated_load_progress();
            self.progress_bar.set_fraction(fraction);
            self.progress_bar
                .set_visible(fraction > 0.0 && fraction < 1.0);
        } else {
            self.entry.set_text("");
            self.gtk_window.set_title("Zero Browser");
            self.back_btn.set_sensitive(false);
            self.forward_btn.set_sensitive(false);
            self.progress_bar.set_visible(false);
        }
    }

    fn sync_nav_buttons(&self) {
        if let Some(tab) = self.active() {
            let wk = tab.webview.webview();
            self.back_btn.set_sensitive(wk.can_go_back());
            self.forward_btn.set_sensitive(wk.can_go_forward());
        }
    }
}
