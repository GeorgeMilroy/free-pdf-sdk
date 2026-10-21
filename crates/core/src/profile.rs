//! pdfcraft ids hidden in every configuration: what makes no sense in a viewer embedded in a
//! page. Opening, creating and comparing files, tabs and update checks are pdfcraft's `app`
//! feature, which the SDK never turns on; these are listed too, so they stay hidden even if a
//! pdfcraft update moves one of them.

/// Hidden in every configuration.
pub const ALWAYS_HIDDEN: &[&str] = &[
    "help.check_updates",
    "help.discord",
    "help.app_page",
    "help.github",
    "help.website",
    "file.open",
    "file.open_recent",
    "file.pin_folder",
    "file.close",
    "file.close_all",
    "file.revert",
    "create.blank",
    "group:create",
    "group:combine",
    "group:compare",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_and_update_checks_are_always_hidden() {
        for id in ["file.open", "help.check_updates", "group:compare"] {
            assert!(ALWAYS_HIDDEN.contains(&id), "{id}");
        }
        let resolved = crate::features::FeatureConfig { mode: crate::features::Mode::Full, ..Default::default() }.resolve();
        assert!(resolved.hidden.contains("file.open") && !resolved.features.contains("app"));
    }
}
