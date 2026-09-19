//! Q16.16 고정소수점. 결정적 시뮬레이션(CB-6)의 기반.

use std::ops::{Add, Div, Mul, Neg, Sub};

const FRAC_BITS: i32 = 16;

/// Q16.16 고정소수점 스칼라.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Fx(pub i32);

impl Fx {
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(1 << FRAC_BITS);

    pub const fn from_raw(raw: i32) -> Fx {
        Fx(raw)
    }

    pub fn from_int(v: i32) -> Fx {
        Fx(v << FRAC_BITS)
    }

    pub fn from_f32(v: f32) -> Fx {
        Fx((v * (1i64 << FRAC_BITS) as f32) as i32)
    }

    pub fn to_f32(self) -> f32 {
        self.0 as f32 / (1i64 << FRAC_BITS) as f32
    }

    pub fn abs(self) -> Fx {
        Fx(self.0.abs())
    }

    /// 정수 제곱근 기반 sqrt (뉴턴법, 결정적).
    ///
    /// `raw`(i32)를 그대로 제곱하면(Fx::mul 경유) 값이 약 181을 넘는 순간
    /// i32 범위를 넘겨 오버플로한다. 여기서는 i128로 승격해서 계산하므로
    /// 화면 좌표 규모(수천 px)에서도 안전하다.
    pub fn sqrt(self) -> Fx {
        if self.0 <= 0 {
            return Fx::ZERO;
        }
        // result_raw = sqrt(V) * 2^16 = sqrt(V * 2^32 / 2^16) = sqrt(raw * 2^16)
        let target = (self.0 as i128) << FRAC_BITS;
        Fx(isqrt_i128(target) as i32)
    }
}

/// 128비트 정수 제곱근 (뉴턴법, 결정적 — 부동소수점 없음).
fn isqrt_i128(n: i128) -> i128 {
    if n <= 0 {
        return 0;
    }
    let mut x = 1i128 << ((128 - n.leading_zeros() as i128) / 2 + 1);
    loop {
        let next = (x + n / x) / 2;
        if next >= x {
            break;
        }
        x = next;
    }
    x
}

impl Add for Fx {
    type Output = Fx;
    fn add(self, rhs: Fx) -> Fx {
        Fx(self.0 + rhs.0)
    }
}

impl Sub for Fx {
    type Output = Fx;
    fn sub(self, rhs: Fx) -> Fx {
        Fx(self.0 - rhs.0)
    }
}

impl Neg for Fx {
    type Output = Fx;
    fn neg(self) -> Fx {
        Fx(-self.0)
    }
}

impl Mul for Fx {
    type Output = Fx;
    fn mul(self, rhs: Fx) -> Fx {
        let v = (self.0 as i64 * rhs.0 as i64) >> FRAC_BITS;
        Fx(v as i32)
    }
}

impl Div for Fx {
    type Output = Fx;
    fn div(self, rhs: Fx) -> Fx {
        let v = ((self.0 as i64) << FRAC_BITS) / rhs.0 as i64;
        Fx(v as i32)
    }
}

/// 2D 벡터, Q16.16 성분.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct V2 {
    pub x: Fx,
    pub y: Fx,
}

impl V2 {
    pub const ZERO: V2 = V2 {
        x: Fx::ZERO,
        y: Fx::ZERO,
    };

    pub fn new(x: Fx, y: Fx) -> V2 {
        V2 { x, y }
    }

    pub fn from_f32(x: f32, y: f32) -> V2 {
        V2 {
            x: Fx::from_f32(x),
            y: Fx::from_f32(y),
        }
    }

    /// `Fx::mul`을 거치지 않고 raw 성분에서 직접 계산한다 — 화면 좌표 규모의
    /// 델타(수천 px)를 제곱하면 `Fx`(i32) 자체가 못 담는 값이 나오기 때문이다.
    pub fn len(self) -> Fx {
        let sx = self.x.0 as i128;
        let sy = self.y.0 as i128;
        let sum_sq = sx * sx + sy * sy; // raw^2 단위 = (V*2^16)^2
        Fx(isqrt_i128(sum_sq) as i32)
    }

    /// 길이가 0이면 ZERO, 아니면 길이 ONE로 정규화.
    pub fn normalized(self) -> V2 {
        let l = self.len();
        if l == Fx::ZERO {
            V2::ZERO
        } else {
            V2::new(self.x / l, self.y / l)
        }
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, rhs: V2) -> V2 {
        V2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, rhs: V2) -> V2 {
        V2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<Fx> for V2 {
    type Output = V2;
    fn mul(self, rhs: Fx) -> V2 {
        V2::new(self.x * rhs, self.y * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 정수_왕복() {
        assert_eq!(Fx::from_int(5).to_f32(), 5.0);
        assert_eq!(Fx::from_int(-3).to_f32(), -3.0);
    }

    #[test]
    fn 덧셈_뺄셈() {
        let a = Fx::from_f32(1.5);
        let b = Fx::from_f32(2.25);
        assert!((a + b).to_f32() - 3.75 < 0.001);
        assert!((b - a).to_f32() - 0.75 < 0.001);
    }

    #[test]
    fn 곱셈_나눗셈() {
        let a = Fx::from_f32(2.0);
        let b = Fx::from_f32(3.0);
        assert!((a * b).to_f32() - 6.0 < 0.001);
        assert!((b / a).to_f32() - 1.5 < 0.001);
    }

    #[test]
    fn 정규화된_벡터의_길이는_1이다() {
        let v = V2::from_f32(3.0, 4.0).normalized();
        let l = v.len().to_f32();
        assert!((l - 1.0).abs() < 0.01, "len={l}");
    }

    #[test]
    fn 대각선_이동은_정규화된다() {
        // W+D 동시입력: (0,-1) + (1,0) = (1,-1), 정규화하면 길이 1
        let raw = V2::from_f32(1.0, -1.0);
        let n = raw.normalized();
        assert!((n.len().to_f32() - 1.0).abs() < 0.01);
    }

    #[test]
    fn 영벡터_정규화는_영벡터() {
        assert_eq!(V2::ZERO.normalized(), V2::ZERO);
    }
}
