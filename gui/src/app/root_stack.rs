mod info_page;
mod oc_page;
mod thermals_page;

use gtk::*;

use info_page::InformationPage;
use oc_page::OcPage;
use thermals_page::ThermalsPage;

#[derive(Clone)]
pub struct RootStack {
    pub container: Stack,
    pub info_page: InformationPage,
    pub thermals_page: ThermalsPage,
    pub oc_page: OcPage,
}

impl RootStack {
    pub fn new() -> Self {
        let container = Stack::new();

        let info_page = InformationPage::new();

        container.add_titled(&info_page.container, "info_page", "Information");

        let oc_page = OcPage::new();

        container.add_titled(&oc_page.container, "oc_page", "OC");

        let thermals_page = ThermalsPage::new();

        container.add_titled(&thermals_page.container, "thermals_page", "Thermals");

        Self {
            container,
            info_page,
            thermals_page,
            oc_page,
        }
    }
}
