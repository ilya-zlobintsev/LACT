mod info_page;

use gtk::*;

use info_page::InformationPage;

#[derive(Clone)]
pub struct RootStack {
    pub container: Stack,
    pub info_page: InformationPage,
}

impl RootStack {
    pub fn new() -> Self {
        let container = Stack::new();

        let info_page = InformationPage::new();

        container.add_titled(&info_page.container, "info_page", "Information");

        Self {
            container,
            info_page,
        }
    }
}
