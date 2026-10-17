//! Opening links and reading desktop settings: through the XDG desktop
//! portal on Linux, and through the system itself elsewhere. Errors are
//! the system's own text, for the app to put in its own words.

#[cfg(target_os = "linux")]
pub use linux::*;

use std::sync::OnceLock;

/// Whether the window manager draws a title bar, with its close button,
/// above the app's windows. Windows and macOS do, and so do most Linux
/// desktops; tiling Wayland compositors such as Hyprland, niri, river and
/// dwl agree to draw a window's decorations and then draw only a border,
/// which Wayland gives an app no way to tell from a desktop that draws a
/// whole title bar, so they are known by name. Where this is false, an app
/// shows a close button of its own. `SCRAMBLE_UI_TITLE_BARS=0` or `1`
/// decides it on a desktop this does not know, such as sway set up without
/// title bars.
pub fn draws_title_bars() -> bool {
    static DRAWS: OnceLock<bool> = OnceLock::new();
    *DRAWS.get_or_init(|| title_bars_from(|name| std::env::var(name).ok()))
}

/// The compositors that draw no title bars, as `XDG_CURRENT_DESKTOP`
/// names them.
const BARE: [&str; 4] = ["hyprland", "niri", "river", "dwl"];

fn title_bars_from(var: impl Fn(&str) -> Option<String>) -> bool {
    if let Some(choice) = var("SCRAMBLE_UI_TITLE_BARS") {
        return choice.trim() != "0";
    }
    if cfg!(any(windows, target_os = "macos")) {
        return true;
    }
    // Without Wayland, an X11 window manager places the window, and it
    // decorates it.
    if var("WAYLAND_DISPLAY").is_none() {
        return true;
    }
    if var("HYPRLAND_INSTANCE_SIGNATURE").is_some() || var("NIRI_SOCKET").is_some() {
        return false;
    }
    let desktops = var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase();
    !desktops.split(':').any(|desktop| BARE.contains(&desktop))
}

#[cfg(not(target_os = "linux"))]
pub use other::*;

#[cfg(target_os = "linux")]
mod linux {
    pub async fn open_uri(uri: String) -> Result<(), String> {
        let parsed = ashpd::Uri::parse(&uri).map_err(|error| error.to_string())?;
        ashpd::desktop::open_uri::OpenFileRequest::default()
            .send_uri(&parsed)
            .await
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    /// Whether the desktop wants animations, from the GNOME interface setting
    /// the settings portal passes on. `None` when the portal cannot tell.
    pub async fn animations_enabled() -> Option<bool> {
        let settings = ashpd::desktop::settings::Settings::new().await.ok()?;
        settings
            .read::<bool>("org.gnome.desktop.interface", "enable-animations")
            .await
            .ok()
    }

    /// The desktop's accent color, through the settings portal, when the
    /// desktop sets one (GNOME, KDE).
    pub async fn accent_color() -> Option<(u8, u8, u8)> {
        let settings = ashpd::desktop::settings::Settings::new().await.ok()?;
        rgb(settings.accent_color().await.ok()?)
    }

    /// The desktop's light or dark preference, through the settings
    /// portal. iced reads it at startup too, but gives the portal only a
    /// fifth of a second, which a busy desktop can miss.
    pub async fn color_scheme() -> Option<iced::theme::Mode> {
        use ashpd::desktop::settings::ColorScheme;
        let settings = ashpd::desktop::settings::Settings::new().await.ok()?;
        match settings.color_scheme().await.ok()? {
            ColorScheme::PreferDark => Some(iced::theme::Mode::Dark),
            ColorScheme::PreferLight => Some(iced::theme::Mode::Light),
            ColorScheme::NoPreference => None,
        }
    }

    /// The accent color each time the desktop changes it.
    pub fn accent_changes() -> impl iced::futures::Stream<Item = Option<(u8, u8, u8)>> {
        use iced::futures::{SinkExt, StreamExt};
        iced::stream::channel(1, async |mut output| {
            let Ok(settings) = ashpd::desktop::settings::Settings::new().await else {
                return;
            };
            let Ok(changes) = settings.receive_accent_color_changed().await else {
                return;
            };
            let mut changes = std::pin::pin!(changes);
            while let Some(color) = changes.next().await {
                if output.send(rgb(color)).await.is_err() {
                    return;
                }
            }
        })
    }

    /// A portal color as 8-bit channels, or `None` for the color outside 0
    /// to 1 the portal sends when none is set.
    fn rgb(color: ashpd::desktop::Color) -> Option<(u8, u8, u8)> {
        let channel = |value: f64| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        [color.red(), color.green(), color.blue()]
            .iter()
            .all(|value| (0.0..=1.0).contains(value))
            .then(|| {
                (
                    channel(color.red()),
                    channel(color.green()),
                    channel(color.blue()),
                )
            })
    }
}

#[cfg(not(target_os = "linux"))]
mod other {
    /// Opens `uri` in the default app: through the URL handler every
    /// Windows version has, or `open` on macOS.
    pub async fn open_uri(uri: String) -> Result<(), String> {
        #[cfg(windows)]
        let mut command = std::process::Command::new("rundll32");
        #[cfg(windows)]
        command.args(["url.dll,FileProtocolHandler", &uri]);
        #[cfg(not(windows))]
        let mut command = std::process::Command::new("open");
        #[cfg(not(windows))]
        command.arg(&uri);
        command
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    /// Whether Windows animates controls and elements, its "Animation
    /// effects" setting.
    #[cfg(windows)]
    #[allow(unsafe_code)]
    pub async fn animations_enabled() -> Option<bool> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SPI_GETCLIENTAREAANIMATION, SystemParametersInfoW,
        };
        let mut enabled: i32 = 1;
        // SAFETY: the setting is a BOOL written to `enabled`, which lives
        // through the call.
        let read = unsafe {
            SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, (&raw mut enabled).cast(), 0)
        };
        (read != 0).then_some(enabled != 0)
    }

    /// Whether macOS animates, the opposite of its Reduce Motion setting.
    #[cfg(target_os = "macos")]
    pub async fn animations_enabled() -> Option<bool> {
        let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
        Some(!workspace.accessibilityDisplayShouldReduceMotion())
    }

    /// Not read from the system here; animations stay on unless the
    /// settings turn them off.
    #[cfg(not(any(windows, target_os = "macos")))]
    pub async fn animations_enabled() -> Option<bool> {
        None
    }

    /// Windows' accent color, as Settings, Personalization, Colors sets it:
    /// the DWM's `AccentColor`, stored as 0xAABBGGRR.
    #[cfg(windows)]
    #[allow(unsafe_code)]
    fn read_accent() -> Option<(u8, u8, u8)> {
        use windows_sys::Win32::System::Registry::{
            HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW,
        };
        let wide = |text: &str| text.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let (key, value) = (
            wide("Software\\Microsoft\\Windows\\DWM"),
            wide("AccentColor"),
        );
        let mut color: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        // SAFETY: the strings end in nul and live through the call, and
        // the DWORD read goes to `color`, `size` bytes long.
        let read = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&raw mut color).cast(),
                &raw mut size,
            )
        };
        (read == 0).then_some((
            (color & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            ((color >> 16) & 0xff) as u8,
        ))
    }

    /// macOS's accent color, as System Settings, Appearance sets it.
    #[cfg(target_os = "macos")]
    fn read_accent() -> Option<(u8, u8, u8)> {
        use objc2_app_kit::{NSColor, NSColorSpace};
        let color =
            NSColor::controlAccentColor().colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
        let channel = |value: f64| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        Some((
            channel(color.redComponent()),
            channel(color.greenComponent()),
            channel(color.blueComponent()),
        ))
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    fn read_accent() -> Option<(u8, u8, u8)> {
        None
    }

    /// The system's accent color.
    pub async fn accent_color() -> Option<(u8, u8, u8)> {
        read_accent()
    }

    /// How often the accent color is looked at again: neither Windows nor
    /// macOS tells iced's apps when it changes.
    const ACCENT_POLL: std::time::Duration = std::time::Duration::from_secs(2);

    /// The accent color each time it changes, from a thread that looks at
    /// it every [`ACCENT_POLL`].
    pub fn accent_changes() -> impl iced::futures::Stream<Item = Option<(u8, u8, u8)>> {
        let (mut sender, receiver) = iced::futures::channel::mpsc::channel(1);
        let _ = std::thread::Builder::new()
            .name("accent-watch".into())
            .spawn(move || {
                let mut last = read_accent();
                loop {
                    std::thread::sleep(ACCENT_POLL);
                    if sender.is_closed() {
                        return;
                    }
                    let now = read_accent();
                    if now != last {
                        last = now;
                        let _ = sender.try_send(now);
                    }
                }
            });
        receiver
    }
}

#[cfg(test)]
mod tests {
    use super::title_bars_from;

    fn with(vars: &[(&str, &str)]) -> bool {
        title_bars_from(|name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    #[cfg(not(any(windows, target_os = "macos")))]
    fn tiling_compositors_draw_no_title_bars() {
        let wayland = ("WAYLAND_DISPLAY", "wayland-1");
        assert!(!with(&[wayland, ("XDG_CURRENT_DESKTOP", "Hyprland")]));
        assert!(!with(&[wayland, ("HYPRLAND_INSTANCE_SIGNATURE", "abc")]));
        assert!(!with(&[wayland, ("NIRI_SOCKET", "/run/niri.sock")]));
        assert!(!with(&[wayland, ("XDG_CURRENT_DESKTOP", "river")]));
        assert!(with(&[wayland, ("XDG_CURRENT_DESKTOP", "GNOME")]));
        assert!(with(&[wayland, ("XDG_CURRENT_DESKTOP", "KDE")]));
        assert!(with(&[wayland, ("XDG_CURRENT_DESKTOP", "sway")]));
        // Under X11 the window manager decorates, whatever the desktop.
        assert!(with(&[("XDG_CURRENT_DESKTOP", "Hyprland")]));
    }

    #[test]
    fn the_variable_decides_first() {
        let wayland = ("WAYLAND_DISPLAY", "wayland-1");
        assert!(with(&[
            wayland,
            ("XDG_CURRENT_DESKTOP", "Hyprland"),
            ("SCRAMBLE_UI_TITLE_BARS", "1")
        ]));
        assert!(!with(&[("SCRAMBLE_UI_TITLE_BARS", "0")]));
    }
}
