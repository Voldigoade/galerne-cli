//! In-run leveling upgrades and selection draft.

use serde::{Deserialize, Serialize};

use crate::game::state::FastRng;
use crate::terminal::buffer::{Buffer, Color, Style};

/// Available in-run enhancement types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeKind {
    DamageBoost,
    FireRateBoost,
    MoveSpeed,
    Multishot,
    Piercing,
    MaxHealth,
    MagnetRange,
}

impl UpgradeKind {
    pub const ALL: [UpgradeKind; 7] = [
        UpgradeKind::DamageBoost,
        UpgradeKind::FireRateBoost,
        UpgradeKind::MoveSpeed,
        UpgradeKind::Multishot,
        UpgradeKind::Piercing,
        UpgradeKind::MaxHealth,
        UpgradeKind::MagnetRange,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            UpgradeKind::DamageBoost => "Overclock Projectiles",
            UpgradeKind::FireRateBoost => "Accelerated Feed",
            UpgradeKind::MoveSpeed => "Thermal Thrusters",
            UpgradeKind::Multishot => "Twin Barrel",
            UpgradeKind::Piercing => "Depleted Penetrator",
            UpgradeKind::MaxHealth => "Reinforced Bulkhead",
            UpgradeKind::MagnetRange => "Magnetic Array",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            UpgradeKind::DamageBoost => "+25% Projectile Damage",
            UpgradeKind::FireRateBoost => "-20% Firing Cooldown",
            UpgradeKind::MoveSpeed => "+20% Movement Speed",
            UpgradeKind::Multishot => "+1 Projectile per Salvo",
            UpgradeKind::Piercing => "+1 Pierce Count",
            UpgradeKind::MaxHealth => "+30 Max HP & Immediate Heal",
            UpgradeKind::MagnetRange => "+50% Magnet Pickup Range",
        }
    }
}

/// A 3-choice upgrade draft presented on level up.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeDraft {
    pub options: [UpgradeKind; 3],
    pub selected: usize,
}

impl UpgradeDraft {
    /// Generates 3 unique upgrade options using the game's deterministic RNG.
    pub fn generate(rng: &mut FastRng) -> Self {
        let mut pool = UpgradeKind::ALL.to_vec();

        // Fisher-Yates shuffle first 3 elements
        for i in 0..3 {
            let j = rng.range_u32(i as u32, (pool.len() - 1) as u32) as usize;
            pool.swap(i, j);
        }

        Self {
            options: [pool[0], pool[1], pool[2]],
            selected: 0,
        }
    }

    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % 3;
    }

    pub fn select_prev(&mut self) {
        self.selected = if self.selected == 0 {
            2
        } else {
            self.selected - 1
        };
    }

    pub fn current_selection(&self) -> UpgradeKind {
        self.options[self.selected]
    }

    /// Renders a centered upgrade modal over the paused game scene.
    pub fn render_modal(&self, buf: &mut Buffer) {
        let w = buf.width();
        let h = buf.height();

        let box_w = 46u16.min(w.saturating_sub(4));
        let box_h = 11u16.min(h.saturating_sub(2));

        let start_x = (w.saturating_sub(box_w)) / 2;
        let start_y = (h.saturating_sub(box_h)) / 2;

        let frame_fg = Color::Cyan;
        let bg = Color::Reset;

        // Draw box frame
        buf.set_char(start_x, start_y, '╔', frame_fg, bg);
        for x in (start_x + 1)..(start_x + box_w - 1) {
            buf.set_char(x, start_y, '═', frame_fg, bg);
        }
        buf.set_char(start_x + box_w - 1, start_y, '╗', frame_fg, bg);

        for y in (start_y + 1)..(start_y + box_h - 1) {
            buf.set_char(start_x, y, '║', frame_fg, bg);
            for x in (start_x + 1)..(start_x + box_w - 1) {
                buf.set_char(x, y, ' ', Color::Reset, bg);
            }
            buf.set_char(start_x + box_w - 1, y, '║', frame_fg, bg);
        }

        buf.set_char(start_x, start_y + box_h - 1, '╚', frame_fg, bg);
        for x in (start_x + 1)..(start_x + box_w - 1) {
            buf.set_char(x, start_y + box_h - 1, '═', frame_fg, bg);
        }
        buf.set_char(start_x + box_w - 1, start_y + box_h - 1, '╝', frame_fg, bg);

        // Header
        let title = "◆ LEVEL UP — SELECT UPGRADE ◆";
        let tx = start_x + (box_w.saturating_sub(title.len() as u16)) / 2;
        buf.draw_str_styled(tx, start_y + 1, title, Style::new(Color::Yellow, bg).bold());

        // Options
        for (idx, opt) in self.options.iter().enumerate() {
            let row_y = start_y + 3 + (idx as u16 * 2);
            let is_sel = idx == self.selected;

            let prefix = if is_sel { ">" } else { " " };
            let line = format!("{} {:<20} {}", prefix, opt.title(), opt.description());

            let style = if is_sel {
                Style::new(Color::Black, Color::Cyan).bold()
            } else {
                Style::new(Color::White, bg)
            };

            buf.draw_str_styled(start_x + 2, row_y, &line, style);
        }

        // Footer prompt
        let prompt = "↑/↓: choose   Enter: confirm";
        let px = start_x + (box_w.saturating_sub(prompt.len() as u16)) / 2;
        buf.draw_str_styled(
            px,
            start_y + box_h - 2,
            prompt,
            Style::new(Color::DarkGrey, bg),
        );
    }
}
