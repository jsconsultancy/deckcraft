//! Keeps the display awake while a slide show runs.
//!
//! Rendering does not reset the OS display-idle timer (on Windows only user input or an explicit
//! `SetThreadExecutionState` call does; winit and egui deliberately leave power management to the
//! app), so a presenter talking over a static slide would otherwise let the screen saver start or
//! the display turn off mid-show. `start_show` takes a [`Guard`]; dropping it (the show ends, the
//! document closes or the app quits) releases the inhibition. The guard is taken at show creation,
//! not the first painted frame, so a show whose window starts minimised or hidden still holds it.
//!
//! The FFI behind [`Guard`] (Windows `SetThreadExecutionState`, macOS IOKit power assertions,
//! Linux `org.freedesktop.ScreenSaver` inhibit) lives in the audited `keepawake` crate — this
//! workspace forbids `unsafe` of its own. Acquiring can fail (no D-Bus session on a headless
//! Linux CI, for instance); that logs a warning and runs the show without an inhibition rather
//! than failing the show.

/// Holds the show's display keep-awake; drop it to release. An empty unit on the web build,
/// which has no power-management API.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Guard = keepawake::KeepAwake;
#[cfg(target_arch = "wasm32")]
pub(crate) type Guard = ();

/// Take a display keep-awake for the duration of a slide show. Returns `None` when the platform
/// refuses one; the show runs on regardless.
pub(crate) fn acquire() -> Option<Guard> {
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match keepawake::Builder::default()
            .display(true)
            // A static show must not idle-sleep the system either, but a user who closes the lid
            // or picks Sleep still gets it (`sleep` stays false), like PowerPoint.
            .idle(true)
            .reason("Slide show")
            .app_name("DeckCraft")
            .app_reverse_domain("ai.storyteller.deckcraft")
            .create()
        {
            Ok(guard) => Some(guard),
            Err(e) => {
                log::warn!("slide show keeps the display awake only on some platforms: {e}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn acquire_never_panics_and_releases_on_drop() {
        // Platform-dependent outcome: Windows succeeds; headless Linux without a D-Bus session
        // returns None. Either way taking and dropping a guard must be quiet and safe.
        let guard = super::acquire();
        drop(guard);
    }
}
