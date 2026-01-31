use std::sync::LazyLock;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    unic_langid::LanguageIdentifier,
    DesktopLanguageRequester, I18nAssets,
};
use rust_embed::RustEmbed;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> =
    LazyLock::new(|| loader(fluent_language_loader!(), &Localizations, None));

pub fn loader(
    loader: FluentLanguageLoader,
    assets: &dyn I18nAssets,
    languages: Option<Vec<LanguageIdentifier>>,
) -> FluentLanguageLoader {
    let requested_languages = languages.unwrap_or_else(DesktopLanguageRequester::requested_languages);
    i18n_embed::select(&loader, assets, &requested_languages)
        .expect("Failed to select localization");
    loader
}

#[derive(RustEmbed)]
#[folder = "i18n"]
pub struct Localizations;
