use std::io::{self, stdout};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{
    Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal;

use crate::game::pixel::ColorTier;
use crate::game::prototype::{InputState, PrototypeGame};
use crate::game::prototype_scene;
use crate::terminal::{Engine, TerminalGuard};
use crate::ui::i18n::Locale;

const FIXED_STEP: Duration = Duration::from_nanos(16_666_667);
const FRAME_STEP: Duration = Duration::from_millis(33);

pub fn run_pixel_prototype() -> io::Result<()> {
    let _terminal = TerminalGuard::enter()?;
    let _keyboard_protocol = KeyboardProtocolGuard::enable()?;
    let mut engine = Engine::new()?;
    let locale = Locale::detect();
    let color_tier = ColorTier::from_environment();
    let mut game = PrototypeGame::new(engine.width(), engine.height());
    let mut input = InputState::default();
    let started = Instant::now();
    let mut previous_update = started;
    let mut last_frame = started;
    let mut accumulator = Duration::ZERO;
    let mut quitting = false;

    while !quitting {
        let mut event = Engine::poll_event(Duration::from_millis(4))?;
        while let Some(current) = event {
            match current {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    if key.code == KeyCode::Esc
                        || matches!(key.code, KeyCode::Char('q' | 'Q'))
                        || (key.modifiers.contains(KeyModifiers::CONTROL)
                            && matches!(key.code, KeyCode::Char('c' | 'C')))
                    {
                        quitting = true;
                        break;
                    }
                    input.key_event(
                        key.code,
                        key.kind,
                        _keyboard_protocol.enabled,
                        started.elapsed(),
                    );
                }
                Event::Key(key) => input.key_event(
                    key.code,
                    key.kind,
                    _keyboard_protocol.enabled,
                    started.elapsed(),
                ),
                Event::Resize(_, _) => {
                    engine.check_resize()?;
                    game.resize(engine.width(), engine.height());
                }
                _ => {}
            }
            event = Engine::poll_event(Duration::ZERO)?;
        }
        if quitting {
            break;
        }

        let now = Instant::now();
        accumulator += (now - previous_update).min(Duration::from_millis(100));
        previous_update = now;
        while accumulator >= FIXED_STEP {
            game.update(FIXED_STEP.as_secs_f32(), &mut input, started.elapsed());
            accumulator -= FIXED_STEP;
        }

        if last_frame.elapsed() >= FRAME_STEP {
            if engine.is_too_small() {
                engine.render_too_small_notice(locale);
            } else {
                let frame = prototype_scene::render(
                    &game,
                    engine.width(),
                    engine.height(),
                    locale,
                    color_tier,
                );
                engine.buffer_mut().copy_from(&frame);
            }
            engine.flush()?;
            last_frame = Instant::now();
        }
        thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

struct KeyboardProtocolGuard {
    enabled: bool,
    pushed: bool,
}

impl KeyboardProtocolGuard {
    fn enable() -> io::Result<Self> {
        #[cfg(windows)]
        let (enabled, pushed) = (true, false);
        #[cfg(unix)]
        let (enabled, pushed) = {
            let known_support = std::env::var_os("KITTY_WINDOW_ID").is_some()
                || std::env::var_os("WEZTERM_PANE").is_some()
                || std::env::var("TERM")
                    .is_ok_and(|term| term.contains("kitty") || term.contains("ghostty"))
                || std::env::var("TERM_PROGRAM").is_ok_and(|program| {
                    matches!(
                        program.to_ascii_lowercase().as_str(),
                        "wezterm" | "ghostty" | "iterm.app"
                    )
                });
            let enabled =
                known_support && terminal::supports_keyboard_enhancement().unwrap_or(false);
            (enabled, enabled)
        };
        #[cfg(not(any(unix, windows)))]
        let (enabled, pushed) = (false, false);

        if pushed {
            execute!(
                stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                )
            )?;
        }
        Ok(Self { enabled, pushed })
    }
}

impl Drop for KeyboardProtocolGuard {
    fn drop(&mut self) {
        if self.pushed {
            let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
        }
    }
}
