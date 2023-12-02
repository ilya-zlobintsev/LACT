mod info_page;
mod oc_page;
mod software_page;
mod thermals_page;

use self::software_page::software_page;
use gtk::{prelude::IsA, *};
use info_page::InformationPage;
use lact_client::schema::SystemInfo;
use oc_page::OcPage;
use thermals_page::ThermalsPage;
use traits::BoxExt;
use traits::WidgetExt;

#[cfg(feature = "adw")]
use adw::prelude::ActionRowExt;

#[derive(Debug, Clone)]
pub struct RootStack {
    #[cfg(feature = "adw")]
    pub container: adw::ViewStack,
    #[cfg(not(feature = "adw"))]
    pub container: Stack,

    pub info_page: InformationPage,
    pub thermals_page: ThermalsPage,
    pub oc_page: OcPage,
}

impl RootStack {
    pub fn new(
        root_win: &impl IsA<Window>,
        system_info: SystemInfo,
        embedded_daemon: bool,
    ) -> Self {
        #[cfg(feature = "adw")]
        let container = adw::ViewStack::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        #[cfg(not(feature = "adw"))]
        let container = Stack::builder().vexpand(true).hexpand(true).build();

        let info_page = InformationPage::new();
        let oc_page = OcPage::new(&system_info);
        let thermals_page = ThermalsPage::new(root_win);
        let software_page = software_page(system_info, embedded_daemon);

        #[cfg(feature = "adw")]
        {
            container.add_titled_with_icon(
                &info_page.container,
                Some("info_page"),
                "Information",
                "info-symbolic",
            );
            container.add_titled_with_icon(
                &oc_page.container,
                Some("oc_page"),
                "Overclock",
                "power-profile-performance-symbolic",
            );
            container.add_titled_with_icon(
                &thermals_page.container,
                Some("thermals_page"),
                "Thermals",
                "temperature-symbolic",
            );
            container.add_titled_with_icon(
                &software_page,
                Some("software_page"),
                "Software",
                "preferences-other-symbolic",
            );
        }

        #[cfg(not(feature = "adw"))]
        {
            container.add_titled(&info_page.container, Some("info_page"), "Information");
            container.add_titled(&oc_page.container, Some("oc_page"), "Overclock");
            container.add_titled(&thermals_page.container, Some("thermals_page"), "Thermals");
            container.add_titled(&software_page, Some("software_page"), "Software");
        }

        Self {
            container,
            info_page,
            thermals_page,
            oc_page,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LabelRow {
    #[cfg(feature = "adw")]
    pub container: adw::ActionRow,

    #[cfg(not(feature = "adw"))]
    pub container: ListBoxRow,

    content_label: Label,
}

impl LabelRow {
    pub fn new(title: &str) -> Self {
        let label = Label::builder()
            .css_classes(["dim-label"])
            .ellipsize(pango::EllipsizeMode::End)
            .xalign(1.0)
            .justify(Justification::Right)
            .selectable(true)
            .build();

        let container = action_row(title, None, &[&label], None);

        Self {
            container,
            content_label: label,
        }
    }

    pub fn new_with_content(title: &str, content: &str) -> Self {
        let row = Self::new(title);
        row.set_content(content);
        row
    }

    pub fn set_content(&self, content: &str) {
        self.content_label.set_label(content);
    }
}

#[cfg(feature = "adw")]
pub fn list_clamp(child: &impl IsA<Widget>) -> adw::Clamp {
    adw::Clamp::builder()
        .maximum_size(600)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(6)
        .margin_end(6)
        .child(child)
        .valign(Align::Start)
        .build()
}

#[cfg(not(feature = "adw"))]
pub fn list_clamp(child: &impl IsA<Widget>) -> Box {
    let container = Box::builder()
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(6)
        .margin_end(6)
        .orientation(Orientation::Vertical)
        .valign(Align::Start)
        .build();
    container.append(child);

    container
}

#[cfg(feature = "adw")]
pub fn action_row(
    title: &str,
    subtitle: Option<&str>,
    suffixes: &[&impl IsA<Widget>],
    css_classes: Option<&[&str]>,
) -> adw::ActionRow {
    let ar = adw::ActionRow::builder()
        .subtitle_lines(0)
        .title(title)
        .build();

    if let Some(css) = css_classes {
        css.iter().for_each(|cls| ar.add_css_class(cls));
    }

    if let Some(sub) = subtitle {
        ar.set_subtitle(sub);
    }
    suffixes.iter().for_each(|suf| {
        ar.add_suffix(*suf);
    });
    ar
}

#[cfg(not(feature = "adw"))]
pub fn action_row(
    title: &str,
    subtitle: Option<&str>,
    suffixes: &[&impl IsA<Widget>],
    css_classes: Option<&[&str]>,
) -> ListBoxRow {
    let inner = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    if let Some(css) = css_classes {
        css.iter().for_each(|cls| inner.add_css_class(cls));
    }

    let title_label = Label::builder()
        .label(title)
        .css_classes(["heading"])
        .hexpand(true)
        .xalign(0.0)
        .wrap(true)
        .wrap_mode(pango::WrapMode::Word)
        .build();
    if let Some(sub) = subtitle {
        let vert = Box::new(Orientation::Vertical, 6);
        vert.append(&title_label);
        vert.append(
            &Label::builder()
                .label(sub)
                .hexpand(true)
                .xalign(0.0)
                .wrap(true)
                .wrap_mode(pango::WrapMode::Word)
                .build(),
        );
        inner.append(&vert);
    } else {
        inner.append(&title_label);
    }

    suffixes.iter().for_each(|suf| {
        inner.append(*suf);
    });

    ListBoxRow::builder()
        .activatable(false)
        .child(&inner)
        .build()
}
