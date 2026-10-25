//! pa3ds's own window (`pa3ds.ux.window`), read back from its template: no test opens it.

use super::test_rename::read_template;
use crate::ux::window::{
    IDC_EXIT, IDC_INSTALLER, IDC_ISSUES, IDC_PORTABLE, IDC_UPDATES, INSTALLER, PORTABLE,
    WS_EX_APPWINDOW, download, menus, template,
};

/// A window left in sight for OBS takes as little of the screen as a window can (Orion,
/// 2026-10-25): what were its buttons are the items of its menus.
#[test]
fn test_the_window_is_pa3dss_by_its_title_and_holds_nothing_but_its_menu_bar() {
    let (title, items) = read_template(&template().unwrap());
    assert_eq!(title, "Pokémon Access 3DS");
    assert!(items.is_empty());
    let owned = |items: &[(i64, &str)]| -> Vec<(i64, String)> {
        items
            .iter()
            .map(|(id, said)| (*id, said.to_string()))
            .collect()
    };
    assert_eq!(
        menus(),
        vec![
            (
                "&pa3ds".to_string(),
                owned(&[(IDC_UPDATES, "Check for &updates"), (IDC_EXIT, "E&xit")])
            ),
            (
                "&Azahar".to_string(),
                owned(&[
                    (IDC_PORTABLE, "Download Azahar (&portable)"),
                    (IDC_INSTALLER, "Download Azahar (&installer)"),
                ])
            ),
            (
                "&Help".to_string(),
                owned(&[(IDC_ISSUES, "&Report an issue on GitHub")])
            ),
        ]
    );
}

/// A dialog with no owner is left off the taskbar and out of Alt+Tab unless it says
/// otherwise, and a double-clicked pa3ds has nothing else there to find it by.
#[test]
fn test_the_window_is_on_the_taskbar_though_it_has_no_owner() {
    let built = template().unwrap();
    let ex_style = u32::from_le_bytes(built[4..8].try_into().unwrap()) as i64;
    assert!(ex_style & WS_EX_APPWINDOW != 0);
}

fn release() -> String {
    let mut assets = Vec::new();
    for build in ["libretro-windows-x86_64", "windows-msvc", "windows-msys2"] {
        for end in [".zip", "-installer.exe"] {
            assets.push(format!(
                "{{\"name\": \"azahar-{build}-2126.2{end}\", \"browser_download_url\": \
                 \"https://example.org/azahar-{build}-2126.2{end}\"}}"
            ));
        }
    }
    format!(
        "{{\"tag_name\": \"2126.2\", \"assets\": [{}]}}",
        assets.join(", ")
    )
}

/// Azahar's 2126.2 offered three Windows builds, each a zip and an installer, every name
/// carrying the version (2026-10-24).
#[test]
fn test_each_download_is_the_latest_releases_own_file_of_the_build_made_with_msvc() {
    assert_eq!(
        download(&release(), &PORTABLE).as_deref(),
        Some("https://example.org/azahar-windows-msvc-2126.2.zip")
    );
    assert_eq!(
        download(&release(), &INSTALLER).as_deref(),
        Some("https://example.org/azahar-windows-msvc-2126.2-installer.exe")
    );
    assert_eq!(download("{\"assets\": []}", &PORTABLE), None);
    assert_eq!(download("not json", &PORTABLE), None);
}
