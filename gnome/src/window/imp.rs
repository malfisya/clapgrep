use adw::subclass::prelude::*;
use clapgrep_core::{
    extended::ExtendedType, manager::{Manager, SearchResult}, options::{Options, Sort}, search::Search
};
use glib::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, CompositeTemplate};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    thread,
};

use crate::search_model::SearchModel;

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/window/window.blp")]
#[properties(wrapper_type = super::Window)]
pub struct Window {
    #[property(get, set)]
    pub file_search: RefCell<String>,
    #[property(get, set)]
    pub content_search: RefCell<String>,
    #[property(get, set)]
    pub results: RefCell<SearchModel>,

    #[property(get, set)]
    pub case_sensitive: Cell<bool>,
    #[property(get, set)]
    pub include_hidden: Cell<bool>,
    #[property(get, set)]
    pub include_ignored: Cell<bool>,
    #[property(get, set)]
    pub disable_regex: Cell<bool>,

    #[property(get, set)]
    pub search_pdf: Cell<bool>,
    #[property(get, set)]
    pub search_office: Cell<bool>,

    pub manager: RefCell<Option<Manager>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "ClapgrepWindow";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl Window {
    #[template_callback]
    fn on_search(&self, _: &adw::ActionRow) {
        if self.manager.borrow().is_none() {
            self.init_manager();
        }

        self.results.borrow().clear();

        if let Some(manager) = self.manager.borrow().as_ref() {
            let search = Search {
                directory: PathBuf::from("."),
                pattern: self.content_search.borrow().to_string(),
            };
            let options = Options {
                sort: Sort::Path,
                case_sensitive: self.case_sensitive.get(),
                ignore_dot: !self.include_hidden.get(),
                use_gitignore: !self.include_ignored.get(),
                fixed_string: self.disable_regex.get(),
                extended: self.get_extended_types(),
                ..Options::default()
            };
            manager.set_options(options);
            manager.search(&search);
        }
    }
}

impl Window {
    fn init_manager(&self) {
        assert!(self.manager.borrow().is_none());

        let (sender, receiver) = std::sync::mpsc::channel();
        let manager = Manager::new(sender);
        manager.set_sort(Sort::Path);
        *self.manager.borrow_mut() = Some(manager);

        let model = self.results.borrow().clone();

        let (async_sender, async_receiver) = flume::unbounded();

        // Relay events from the sync receiver to the async sender
        thread::spawn(move || {
            while let Ok(result) = receiver.recv() {
                let _ = async_sender.send(result);
            }
        });

        // Now handle the event
        glib::MainContext::default().spawn_local(async move {
            while let Ok(result) = async_receiver.recv_async().await {
                match result {
                    SearchResult::FinalResults(results) => {
                        model.clear();
                        for file_info in results.data {
                            model.append_file_info(&file_info);
                        }
                    }
                    SearchResult::InterimResult(file_info) => {
                        model.append_file_info(&file_info);
                    }
                    SearchResult::SearchErrors(_) => {}
                    SearchResult::SearchCount(_) => {}
                }
            }
        });
    }

    fn get_extended_types(&self) -> Vec<ExtendedType> {
        let mut types = Vec::new();

        if self.search_pdf.get() {
            types.push(ExtendedType::Pdf);
        }

        if self.search_office.get() {
            types.push(ExtendedType::Office);
        }

        types
    }
}

#[glib::derived_properties]
impl ObjectImpl for Window {}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}

impl AdwApplicationWindowImpl for Window {}
