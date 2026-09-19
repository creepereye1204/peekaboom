//! 플로팅 가상 조이스틱 → 방향 벡터 (DD-005).
//! 터치 시작 지점이 중심. 데드존 12%, 최대 반경 64 CSS px.

use peekaboom_sim::V2;

pub const MAX_RADIUS_PX: f32 = 64.0;
pub const DEADZONE_RATIO: f32 = 0.12;

/// `origin`(터치 시작점)과 `current`(현재 포인터 위치, 둘 다 CSS px)에서
/// 이동 방향을 계산한다. 데드존 안이면 ZERO, 밖이면 정규화된 방향
/// (반경 캡: 최대 반경 밖으로 끌어도 방향만 갱신되고 크기는 1.0 고정).
pub fn direction(origin: (f32, f32), current: (f32, f32)) -> V2 {
    let dx = current.0 - origin.0;
    let dy = current.1 - origin.1;
    let dist = (dx * dx + dy * dy).sqrt();

    let deadzone_px = MAX_RADIUS_PX * DEADZONE_RATIO;
    if dist < deadzone_px {
        return V2::ZERO;
    }

    V2::from_f32(dx, dy).normalized()
}

/// 시각 표시용: 손잡이(knob)가 그려질 오프셋. 최대 반경 밖으로 끌어도
/// 손잡이 자체는 반경 안에 머문다 (`direction()`은 방향만 정규화하고 이건
/// 렌더링 위치를 위해 크기까지 클램프한다는 점이 다르다).
pub fn knob_offset(origin: (f32, f32), current: (f32, f32)) -> (f32, f32) {
    let dx = current.0 - origin.0;
    let dy = current.1 - origin.1;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= MAX_RADIUS_PX || dist == 0.0 {
        (dx, dy)
    } else {
        let scale = MAX_RADIUS_PX / dist;
        (dx * scale, dy * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 데드존_안이면_영벡터() {
        let d = direction((100.0, 100.0), (102.0, 101.0));
        assert_eq!(d, V2::ZERO);
    }

    #[test]
    fn 데드존_밖이면_정규화된_방향() {
        let d = direction((100.0, 100.0), (100.0, 200.0));
        assert!((d.len().to_f32() - 1.0).abs() < 0.01);
        assert!(d.y.to_f32() > 0.0);
    }

    #[test]
    fn 최대반경_넘어도_크기는_1로_고정() {
        let near = direction((0.0, 0.0), (0.0, 64.0));
        let far = direction((0.0, 0.0), (0.0, 6400.0));
        assert!((near.len().to_f32() - far.len().to_f32()).abs() < 0.01);
    }

    #[test]
    fn 손잡이_오프셋은_최대반경으로_클램프된다() {
        let (dx, dy) = knob_offset((0.0, 0.0), (0.0, 6400.0));
        assert!((dy - MAX_RADIUS_PX).abs() < 0.01, "dy={dy}");
        assert!(dx.abs() < 0.01);
    }

    #[test]
    fn 손잡이_오프셋은_반경_안에서_그대로() {
        let (dx, dy) = knob_offset((0.0, 0.0), (10.0, 20.0));
        assert_eq!((dx, dy), (10.0, 20.0));
    }
}
