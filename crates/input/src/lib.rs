//! `peekaboom-input` — 장치별 입력을 `InputIntent`로 수렴시킨다 (DD-005, H6).
//! `sim`에만 의존한다. DOM을 직접 만지지 않는다 — 그건 `app`의 얇은 껍데기가 한다.

pub mod joystick;
pub mod keyboard;

use peekaboom_sim::{InputIntent, V2};

/// 키보드/터치 입력을 하나의 `InputIntent`로 합친다.
/// 터치 조이스틱이 활성 상태(Some)면 그걸 우선한다 — 같은 프레임에 키보드와
/// 터치가 동시에 들어올 일은 실제로는 없지만(장치가 다르므로), 우선순위를 정해둔다.
pub fn combine(keyboard_move: V2, keyboard_action: u8, joystick: Option<V2>) -> InputIntent {
    let move_dir = joystick.unwrap_or(keyboard_move);
    InputIntent {
        move_dir,
        action: keyboard_action,
        look: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 조이스틱이_있으면_그걸_쓴다() {
        let kb = V2::from_f32(1.0, 0.0);
        let js = Some(V2::from_f32(0.0, 1.0));
        let i = combine(kb, 0, js);
        assert_eq!(i.move_dir, V2::from_f32(0.0, 1.0));
    }

    #[test]
    fn 조이스틱_없으면_키보드를_쓴다() {
        let kb = V2::from_f32(1.0, 0.0);
        let i = combine(kb, 0, None);
        assert_eq!(i.move_dir, kb);
    }
}
