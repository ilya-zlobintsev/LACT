mod info_page;
mod oc_page;
mod software_page;
mod thermals_page;

use gtk::*;

use info_page::InformationPage;
use oc_page::OcPage;
use software_page::SoftwarePage;
use thermals_page::ThermalsPage;

#[derive(Clone)]
pub struct RootStack {
    pub container: Stack,
    pub info_page: InformationPage,
    pub thermals_page: ThermalsPage,
    pub software_page: SoftwarePage,
    pub oc_page: OcPage,
}

impl RootStack {
    pub fn new() -> Self {
        let container = Stack::builder().vexpand(true).build();

        let info_page = InformationPage::new();

        container.add_titled(&info_page.container, Some("info_page"), "Information");

        let oc_page = OcPage::new();

        container.add_titled(&oc_page.container, Some("oc_page"), "OC");

        let thermals_page = ThermalsPage::new();

        container.add_titled(&thermals_page.container, Some("thermals_page"), "Thermals");

        let software_page = SoftwarePage::new();

        container.add_titled(&software_page.container, Some("software_page"), "Software");

        Self {
            container,
            info_page,
            thermals_page,
            oc_page,
            software_page,
        }
    }
}
