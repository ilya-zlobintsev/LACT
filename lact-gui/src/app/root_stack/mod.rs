mod info_page;
mod oc_page;
mod software_page;
mod thermals_page;

use gtk::*;

use self::software_page::software_page;
use info_page::InformationPage;
use lact_client::schema::SystemInfo;
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
    pub fn new(system_info: SystemInfo, embedded_daemon: bool) -> Self {
        let container = Stack::builder().vexpand(true).build();

        let info_page = InformationPage::new();

        container.add_titled(&info_page.container, Some("info_page"), "Information");

        let oc_page = OcPage::new();

        container.add_titled(&oc_page.container, Some("oc_page"), "OC");

        let thermals_page = ThermalsPage::new();

        container.add_titled(&thermals_page.container, Some("thermals_page"), "Thermals");

        let software_page = software_page(system_info, embedded_daemon);
        container.add_titled(&software_page, Some("software_page"), "Software");

        Self {
            container,
            info_page,
            thermals_page,
            oc_page,
        }
    }
}
