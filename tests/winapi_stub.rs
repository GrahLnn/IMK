use imk::platform_win::winapi::get_foreground_process_name;

#[test]
fn foreground_process_name_compiles() {
    let _ = get_foreground_process_name();
}
