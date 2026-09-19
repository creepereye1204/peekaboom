//! 키보드 → InputIntent. 순수 함수: 브라우저 이벤트를 직접 만지지 않는다 (DD-005).

use peekaboom_sim::{action_flags, V2};

/// 눌린 키의 `KeyboardEvent.code` 값들을 보고 이동 방향을 계산한다.
/// 대각선 입력은 정규화된다.
pub fn move_dir(pressed: &[&str]) -> V2 {
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let has = |code: &str| pressed.iter().any(|k| *k == code);

    if has("KeyW") || has("ArrowUp") {
        y -= 1.0;
    }
    if has("KeyS") || has("ArrowDown") {
        y += 1.0;
    }
    if has("KeyA") || has("ArrowLeft") {
        x -= 1.0;
    }
    if has("KeyD") || has("ArrowRight") {
        x += 1.0;
    }

    V2::from_f32(x, y).normalized()
}

/// 눌린 키에서 액션 플래그를 뽑는다.
pub fn action_flags_from_keys(pressed: &[&str]) -> u8 {
    let has = |code: &str| pressed.iter().any(|k| *k == code);
    let mut flags = 0u8;
    if has("Space") || has("KeyE") {
        flags |= action_flags::INTERACT;
    }
    if has("ShiftLeft") || has("ShiftRight") {
        flags |= action_flags::DASH;
    }
    if has("Digit1") {
        flags |= action_flags::EMOTE_1;
    }
    if has("Digit2") {
        flags |= action_flags::EMOTE_2;
    }
    if has("Digit3") {
        flags |= action_flags::EMOTE_3;
    }
    if has("Digit4") {
        flags |= action_flags::EMOTE_4;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 대각선_이동은_정규화된다() {
        let d = move_dir(&["KeyW", "KeyD"]);
        assert!((d.len().to_f32() - 1.0).abs() < 0.01);
    }

    #[test]
    fn 반대_키_동시입력은_상쇄된다() {
        let d = move_dir(&["KeyW", "KeyS"]);
        assert_eq!(d, V2::ZERO);
    }

    #[test]
    fn 스페이스는_interact() {
        assert_eq!(action_flags_from_keys(&["Space"]), action_flags::INTERACT);
    }

    #[test]
    fn 여러_액션_동시입력() {
        let f = action_flags_from_keys(&["ShiftLeft", "Digit1"]);
        assert!(f & action_flags::DASH != 0);
        assert!(f & action_flags::EMOTE_1 != 0);
    }
}
