//! InputIntent — 모든 입력 장치가 수렴하는 단일 타입 (DD-005, H6).
//! `sim`은 이게 키보드에서 왔는지 터치에서 왔는지 모른다.

use crate::fixed::V2;

/// 비트플래그 상수. `bitflags` 크레이트를 끌어오지 않는다 —
/// `sim`은 의존성 0이어야 한다 (repo-layout.md DD-006 검증 규칙).
pub mod action_flags {
    pub const INTERACT: u8 = 0b0000_0001;
    pub const DASH: u8 = 0b0000_0010;
    pub const EMOTE_1: u8 = 0b0000_0100;
    pub const EMOTE_2: u8 = 0b0000_1000;
    pub const EMOTE_3: u8 = 0b0001_0000;
    pub const EMOTE_4: u8 = 0b0010_0000;
}

/// 이동/조준/액션을 담는 정규화된 입력 프레임. 10바이트로 네트워크 전송된다
/// (실제 Q8.8 압축은 `net::wire`가 담당 — 여기서는 sim 연산 편의를 위해 V2(Q16.16)를 쓴다).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct InputIntent {
    /// 이동 방향. 길이 0 또는 ≈1.0.
    pub move_dir: V2,
    /// action_flags 비트플래그.
    pub action: u8,
    /// 조준 방향. None이면 move_dir을 따른다 (터치 기본 동작).
    pub look: Option<V2>,
}

impl InputIntent {
    pub const NONE: InputIntent = InputIntent {
        move_dir: V2::ZERO,
        action: 0,
        look: None,
    };

    pub fn has_action(&self, flag: u8) -> bool {
        self.action & flag != 0
    }

    /// 조준 방향: look이 있으면 그것, 없으면 이동 방향.
    pub fn look_or_move(&self) -> V2 {
        self.look.unwrap_or(self.move_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_없으면_이동방향을_따른다() {
        let i = InputIntent {
            move_dir: V2::from_f32(1.0, 0.0),
            action: 0,
            look: None,
        };
        assert_eq!(i.look_or_move(), V2::from_f32(1.0, 0.0));
    }

    #[test]
    fn look_있으면_그걸_쓴다() {
        let i = InputIntent {
            move_dir: V2::from_f32(1.0, 0.0),
            action: 0,
            look: Some(V2::from_f32(0.0, 1.0)),
        };
        assert_eq!(i.look_or_move(), V2::from_f32(0.0, 1.0));
    }

    #[test]
    fn action_플래그_확인() {
        let i = InputIntent {
            move_dir: V2::ZERO,
            action: action_flags::DASH,
            look: None,
        };
        assert!(i.has_action(action_flags::DASH));
        assert!(!i.has_action(action_flags::INTERACT));
    }
}
