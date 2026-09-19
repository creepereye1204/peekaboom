//! 인스턴스 배칭용 데이터 레이아웃. GL을 몰라도 되는 순수 부분만 여기 둔다.

/// 사각형 하나를 그리기 위한 인스턴스 데이터.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    /// 화면 픽셀 좌표 (좌상단 원점, 카메라 없음 — 001 슬라이스).
    pub x: f32,
    pub y: f32,
    /// 정사각형 한 변의 픽셀 크기.
    pub size: f32,
    pub color: [f32; 4],
}

/// 인스턴스당 float 개수: x, y, size, r, g, b, a.
pub const FLOATS_PER_INSTANCE: usize = 7;

/// 인스턴스 목록을 GPU에 업로드할 평평한 f32 버퍼로 직렬화한다.
pub fn pack(instances: &[Instance]) -> Vec<f32> {
    let mut buf = Vec::with_capacity(instances.len() * FLOATS_PER_INSTANCE);
    for i in instances {
        buf.push(i.x);
        buf.push(i.y);
        buf.push(i.size);
        buf.extend_from_slice(&i.color);
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 팩킹된_길이는_인스턴스수_곱하기_7() {
        let instances = vec![
            Instance {
                x: 1.0,
                y: 2.0,
                size: 10.0,
                color: [1.0, 0.0, 0.0, 1.0],
            },
            Instance {
                x: 3.0,
                y: 4.0,
                size: 12.0,
                color: [0.0, 1.0, 0.0, 1.0],
            },
        ];
        let packed = pack(&instances);
        assert_eq!(packed.len(), 2 * FLOATS_PER_INSTANCE);
        assert_eq!(packed[0], 1.0);
        assert_eq!(packed[7], 3.0);
    }
}
