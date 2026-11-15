use scrollpilot::layout::Shell;

#[test]
fn custom_frame_keeps_drag_resize_and_buttons_separate() {
    use scrollpilot::layout::{frame_hit, FrameHit::*};
    assert_eq!(frame_hit(2, 2, 1280, 920, false), TopLeft);
    assert_eq!(frame_hit(1278, 2, 1280, 920, false), TopRight);
    assert_eq!(frame_hit(2, 918, 1280, 920, false), BottomLeft);
    assert_eq!(frame_hit(1278, 918, 1280, 920, false), BottomRight);
    assert_eq!(frame_hit(500, 20, 1280, 920, false), Caption);
    assert_eq!(frame_hit(1160, 20, 1280, 920, false), Client);
    assert_eq!(frame_hit(500, 100, 1280, 920, false), Client);
    assert_eq!(frame_hit(2, 2, 1280, 920, true), Caption);
    assert_eq!(frame_hit(1278, 918, 1280, 920, true), Client);
}

#[test]
fn resized_shell_keeps_actions_visible_and_cards_reachable() {
    for (w, h) in [(940, 680), (1100, 760), (1280, 920), (1600, 1000)] {
        let shell = Shell::new(w, h);
        assert!(shell.footer.y + shell.footer.h <= h);
        assert!(shell.body.y + shell.body.h <= shell.footer.y);
        assert!(shell.body.x > shell.rail);
        let cards = shell.cards();
        for card in &cards {
            assert!(card.w > 300 && card.h > 100);
        }
        for (i, a) in cards.iter().enumerate() {
            for b in &cards[i + 1..] {
                assert!(!a.overlaps(*b));
            }
        }
        assert!(shell.content_height() >= cards.iter().map(|r| r.y + r.h).max().unwrap());
    }
}

#[test]
fn narrow_windows_stack_cards_in_reading_order() {
    let shell = Shell::new(940, 680);
    let cards = shell.cards();
    assert!(cards
        .windows(2)
        .all(|pair| pair[0].y + pair[0].h <= pair[1].y));
}

#[test]
fn scroll_test_uses_the_removed_profile_space_and_available_height() {
    let shell = Shell::with_exclusions(1280, 920, 4);
    let [feel, options, excluded, test] = shell.cards();
    assert_eq!(shell.body.y, 64);
    assert_eq!(test.y, excluded.y + excluded.h + 16);
    assert_eq!(test.x, excluded.x);
    assert_eq!(test.w, excluded.w);
    assert!(test.h >= 450);
    assert!(test.y + test.h >= options.y + options.h);
    assert!(!feel.overlaps(test));
}

#[test]
fn compact_default_fits_all_normal_cards_without_a_vertical_scrollbar() {
    use scrollpilot::layout::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
    for exclusions in [0, 1, 2, 3, 4] {
        let shell = Shell::with_exclusions(DEFAULT_WIDTH, DEFAULT_HEIGHT, exclusions);
        assert!(shell.content_height() <= shell.body.h);
        let cards = shell.cards();
        assert!(cards[3].x > cards[0].x);
        assert!(cards[3].h >= 380);
        assert_eq!(cards[3].x + cards[3].w, shell.body.w);
        assert!(shell.body.y + shell.body.h < shell.footer.y);
        assert!(shell.footer.y + shell.footer.h <= DEFAULT_HEIGHT - 16);
    }
}

#[test]
fn every_excluded_app_card_stays_reachable_without_overlapping_test() {
    for count in [0, 1, 2, 3, 4, 16, 128] {
        for (w, h) in [(940, 680), (1100, 760), (1280, 920), (1600, 1000)] {
            let shell = Shell::with_exclusions(w, h, count);
            let cards = shell.cards();
            for i in 0..count {
                let row = shell.exclusion(i);
                let viewport = shell.exclusion_viewport();
                assert!(row.x >= 0 && row.x + row.w <= viewport.w);
                assert!(row.y >= 0 && row.y + row.h <= shell.exclusion_content_height());
                let maximum_scroll = (shell.exclusion_content_height() - viewport.h).max(0);
                if i == count - 1 {
                    assert!(row.y + row.h - maximum_scroll <= viewport.h);
                }
                for j in 0..i {
                    assert!(!row.overlaps(shell.exclusion(j)));
                }
            }
            for (i, a) in cards.iter().enumerate() {
                for b in &cards[i + 1..] {
                    assert!(!a.overlaps(*b));
                }
            }
            assert!(shell.content_height() >= cards[3].y + cards[3].h);
        }
    }
}

#[test]
fn large_exclusion_lists_leave_the_test_card_room() {
    let shell = Shell::with_exclusions(1140, 768, 128);
    let cards = shell.cards();
    assert!(
        cards[2].h <= 200,
        "Exclusions must scroll inside a bounded card"
    );
    assert!(cards[3].h >= 390);
    assert!(shell.content_height() <= shell.body.h);
    assert!(
        shell.exclusion(0).h <= 26,
        "Compact cards should be about half their old height"
    );
}

#[test]
fn compact_exclusion_entry_is_right_aligned_with_room_for_three_rows() {
    for w in [940, 1100, 1140, 1280] {
        let shell = Shell::with_exclusions(w, 768, 128);
        let card = shell.cards()[2];
        let (entry, add) = shell.exclusion_entry();
        assert_eq!(entry.w + 8 + add.w, (card.w - 40) * 3 / 4);
        assert_eq!(add.x + add.w, card.x + card.w - 20);
        assert_eq!(entry.y, add.y);
        assert!(entry.y + entry.h < shell.exclusion_viewport().y);
        assert!(shell.exclusion_viewport().h >= 3 * 32 - 6);
    }
}

#[test]
fn collapsed_settings_card_pulls_the_following_cards_up() {
    use scrollpilot::layout::{form_row_visible, SETTINGS_COLLAPSED, SETTINGS_EXPANDED};
    for width in [700, 1140] {
        let open = Shell::new(width, 768).cards();
        let shut = Shell::new(width, 768)
            .with_settings_height(SETTINGS_COLLAPSED)
            .cards();
        assert_eq!(open[0].h, SETTINGS_EXPANDED);
        assert_eq!(shut[0].h, SETTINGS_COLLAPSED);
        let lift = SETTINGS_EXPANDED - SETTINGS_COLLAPSED;
        assert_eq!(shut[1].y, open[1].y - lift, "Options move up with the card");
        assert_eq!(shut[1].y, shut[0].y + shut[0].h + 16);
    }
    assert!((0..6).all(|i| !form_row_visible(i, SETTINGS_COLLAPSED)));
    assert!((0..6).all(|i| form_row_visible(i, SETTINGS_EXPANDED)));
    let partly = 160;
    assert!(form_row_visible(0, partly) && !form_row_visible(5, partly));
}
