use galerne::terminal::buffer::{Buffer, Cell, Color, Style};

#[test]
fn test_buffer_initialization_and_indexing() {
    let mut buf = Buffer::new(80, 24);
    assert_eq!(buf.width(), 80);
    assert_eq!(buf.height(), 24);

    let cell = buf.get(10, 5).expect("valid cell coordinates");
    assert_eq!(cell.ch, ' ');

    buf.set_char(10, 5, '@', Color::Cyan, Color::Reset);
    let updated = buf.get(10, 5).expect("valid cell coordinates");
    assert_eq!(updated.ch, '@');
    assert_eq!(updated.fg, Color::Cyan);
}

#[test]
fn test_buffer_draw_string_and_clipping() {
    let mut buf = Buffer::new(20, 10);
    buf.draw_str(15, 2, "GALERNE", Color::Yellow, Color::Reset);

    // Characters within bounds
    assert_eq!(buf.get(15, 2).unwrap().ch, 'G');
    assert_eq!(buf.get(16, 2).unwrap().ch, 'A');
    assert_eq!(buf.get(17, 2).unwrap().ch, 'L');
    assert_eq!(buf.get(18, 2).unwrap().ch, 'E');
    assert_eq!(buf.get(19, 2).unwrap().ch, 'R');
    // Characters beyond boundary (x >= 20) are clipped without panicking
    assert!(buf.get(20, 2).is_none());
}

#[test]
fn test_buffer_diff_detection() {
    let prev = Buffer::new(10, 5);
    let mut curr = Buffer::new(10, 5);

    // Identical buffers produce empty diff
    let diffs: Vec<_> = curr.diff(&prev).collect();
    assert!(
        diffs.is_empty(),
        "Identical buffers should produce no diffs"
    );

    // Mutate one cell
    curr.set_char(3, 2, 'X', Color::Red, Color::Reset);
    let diffs: Vec<_> = curr.diff(&prev).collect();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].0, 3);
    assert_eq!(diffs[0].1, 2);
    assert_eq!(diffs[0].2.ch, 'X');
    assert_eq!(diffs[0].2.fg, Color::Red);
}

#[test]
fn test_buffer_resize_preserves_or_resets() {
    let mut buf = Buffer::new(10, 10);
    buf.set_char(2, 2, 'P', Color::Green, Color::Reset);
    buf.resize(20, 15);
    assert_eq!(buf.width(), 20);
    assert_eq!(buf.height(), 15);
}

#[test]
fn test_draw_string_advances_by_terminal_columns_for_wide_glyphs() {
    let mut buf = Buffer::new(8, 1);
    buf.draw_str(0, 0, "A中B", Color::White, Color::Reset);

    assert_eq!(buf.get(0, 0).unwrap().ch, 'A');
    assert_eq!(buf.get(1, 0).unwrap().ch, '中');
    assert_eq!(buf.get(1, 0).unwrap().width, 2);
    assert_eq!(buf.get(2, 0).unwrap().width, 0);
    assert_eq!(buf.get(3, 0).unwrap().ch, 'B');
}

#[test]
fn test_wide_glyphs_are_clipped_without_partial_output() {
    let mut buf = Buffer::new(3, 1);
    buf.draw_str(2, 0, "中", Color::White, Color::Reset);

    assert_eq!(buf.get(2, 0).unwrap().ch, ' ');
    assert_eq!(buf.get(2, 0).unwrap().width, 1);
}

#[test]
fn test_overwriting_a_wide_glyph_continuation_clears_its_lead_cell() {
    let mut buf = Buffer::new(5, 1);
    buf.draw_str(0, 0, "A中B", Color::White, Color::Reset);

    buf.set_char(2, 0, 'X', Color::Yellow, Color::Reset);

    assert_eq!(buf.get(1, 0).unwrap().ch, ' ');
    assert_eq!(buf.get(1, 0).unwrap().width, 1);
    assert_eq!(buf.get(2, 0).unwrap().ch, 'X');
    assert_eq!(buf.get(3, 0).unwrap().ch, 'B');
}

#[test]
fn test_setting_a_wide_cell_reserves_its_second_column() {
    let mut buf = Buffer::new(4, 1);
    buf.set(1, 0, Cell::new('中', Style::new(Color::Cyan, Color::Reset)));

    assert_eq!(buf.get(1, 0).unwrap().ch, '中');
    assert_eq!(buf.get(1, 0).unwrap().width, 2);
    assert_eq!(buf.get(2, 0).unwrap().width, 0);
}
