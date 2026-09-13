use super::{
    detachable_page::{DetachablePage, DetachablePageInit, DetachablePageMsg},
    utils::ext::RelmLaunchable,
};
use crate::I18N;
use adw::prelude::*;
use i18n_embed_fl::fl;
use relm4::{
    ComponentController, ComponentParts, ComponentSender, RelmWidgetExt, binding::BoolBinding,
};

pub struct PageNavigation {
    pages: Vec<relm4::Controller<DetachablePage>>,
}

pub struct PageNavigationInit {
    pub pages: Vec<(&'static str, String, gtk::Widget)>,
    pub parent: adw::ApplicationWindow,
    pub sensitive: BoolBinding,
}

#[derive(Debug)]
pub enum PageNavigationMsg {
    Select(usize),
    SyncSelection,
    CloseWindows,
}

#[relm4::component(pub)]
impl relm4::Component for PageNavigation {
    type Init = PageNavigationInit;
    type Input = PageNavigationMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::ListBox {
            add_css_class: "navigation-sidebar",
            set_margin_vertical: 1,
            set_vexpand: true,
            connect_row_selected[sender] => move |_, row| {
                if let Some(row) = row {
                    sender.input(PageNavigationMsg::Select(row.index() as usize));
                }
            } @ selection_signal,
        },

        #[name = "stack"]
        gtk::Stack {
            set_vhomogeneous: false,
            connect_visible_child_name_notify => PageNavigationMsg::SyncSelection,
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let pages = init
            .pages
            .into_iter()
            .map(|(name, title, content)| {
                DetachablePage::detach(DetachablePageInit {
                    name,
                    title,
                    content,
                    parent: init.parent.clone(),
                    sensitive: init.sensitive.clone(),
                })
            })
            .collect();
        let model = Self { pages };
        let widgets = view_output!();

        for page in &model.pages {
            root.append(&page.widgets().row);
            widgets.stack.add_titled(
                page.widget(),
                Some(page.model().init.name),
                &page.model().init.title,
            );
        }

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            PageNavigationMsg::Select(index) => {
                widgets
                    .stack
                    .set_visible_child_name(self.pages[index].model().init.name);
            }
            PageNavigationMsg::SyncSelection => {
                let index = self.pages.iter().position(|page| {
                    Some(page.model().init.name) == widgets.stack.visible_child_name().as_deref()
                });
                root.block_signal(&widgets.selection_signal);
                root.select_row(
                    index
                        .and_then(|index| root.row_at_index(index as i32))
                        .as_ref(),
                );
                root.unblock_signal(&widgets.selection_signal);
            }
            PageNavigationMsg::CloseWindows => {
                for page in &self.pages {
                    page.emit(DetachablePageMsg::Attach);
                }
            }
        }
    }
}
