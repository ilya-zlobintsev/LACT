use gtk::{
    gdk::{self, prelude::TextureExt as _},
    gio::{self, prelude::ApplicationExt as _},
    glib::object::IsA,
    gsk::{self, prelude::GskRendererExt as _},
    prelude::{
        Cast as _, GtkApplicationExt as _, GtkWindowExt, PaintableExt as _, SnapshotExt as _,
        WidgetExt as _,
    },
};
use lact_schema::args::GuiArgs;
use relm4::{
    component::{AsyncComponent, AsyncComponentController},
    tokio,
};
use std::{cell::OnceCell, path::PathBuf, sync::LazyLock, time::Duration};

use crate::{
    APP_ID,
    app::{AppModel, msg::AppMsg},
};

static TOKIO_RT: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

fn init_test_app() -> gtk::Application {
    thread_local! {
        static MAIN_APPLICATION: OnceCell<gtk::Application> = const { OnceCell::new() };
    }

    MAIN_APPLICATION.with(|cell| {
        cell.get_or_init(move || {
            let app = gtk::Application::new(
                Some(format!("{APP_ID}-test")),
                gio::ApplicationFlags::default(),
            );
            app.register(gio::Cancellable::NONE).unwrap();
            app
        })
        .clone()
    })
}

#[gtk::test]
async fn snapshot_app() {
    let _guard = TOKIO_RT.enter();
    let app = init_test_app();

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .parse("trace")
        .unwrap();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let mut app_controller = AppModel::builder().launch(GuiArgs::default()).detach();
    app_controller.detach_runtime();
    let widget = app_controller.widget();
    app.add_window(widget);
    widget.present();

    let test_data_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/data/amd/rx6900xt");

    unsafe {
        std::env::set_var("_LACT_DRM_SYSFS_PATH", test_data_dir.to_str().unwrap());
    }

    tokio::time::sleep(Duration::from_millis(150)).await;

    insta::assert_binary_snapshot!("screenshot-main.png", widget_to_png(widget));

    app_controller.emit(AppMsg::SelectPage("oc_page".to_owned()));
    tokio::time::sleep(Duration::from_millis(20)).await;
    insta::assert_binary_snapshot!("screenshot-oc.png", widget_to_png(widget));

    app_controller.emit(AppMsg::SelectPage("thermals_page".to_owned()));
    tokio::time::sleep(Duration::from_millis(20)).await;
    insta::assert_binary_snapshot!("screenshot-thermals.png", widget_to_png(widget));
}

fn widget_to_png(widget: &impl IsA<gtk::Widget>) -> Vec<u8> {
    let widget = widget.upcast_ref();
    let paintable = gtk::WidgetPaintable::new(Some(widget));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(
        snapshot.upcast_ref::<gdk::Snapshot>(),
        widget.width().into(),
        widget.height().into(),
    );
    let node = snapshot.to_node().expect("empty render tree");

    let renderer = gsk::CairoRenderer::new();
    renderer.realize(gdk::Surface::NONE).unwrap();
    let texture = renderer.render_texture(&node, None);

    texture.save_to_png_bytes().to_vec()
}
