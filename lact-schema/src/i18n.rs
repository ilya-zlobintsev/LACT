use std::sync::LazyLock;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester, I18nAssets,
};
use rust_embed::RustEmbed;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> =
    LazyLock::new(|| loader(fluent_language_loader!(), &Localizations));

pub fn loader(loader: FluentLanguageLoader, assets: &dyn I18nAssets) -> FluentLanguageLoader {
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _selected_languages = i18n_embed::select(&loader, assets, &requested_languages)
        .expect("Failed to select localization");

    loader
}

#[derive(RustEmbed)]
#[folder = "i18n"]
pub struct Localizations;
