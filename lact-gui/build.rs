fn main() {
    glib_build_tools::compile_resources(&["ui"], "ui/resources.gresource.xml", "ui.gresource");
}
