//! 워크스페이스 통합 테스트: 결정적 시뮬레이션 (CB-6).
//! CI에서 별도로 확인한다 (repo-layout.md CI 7단계).
//!
//! 001 범위: `World::step`이 하는 일은 플레이어 하나를 미는 것뿐이지만,
//! 그 성질(같은 입력 시퀀스 → 같은 결과)은 호스트 이양·재조정·리플레이가
//! 전부 이 위에 올라가는 만큼 지금부터 틀어지면 안 된다.

use peekaboom_sim::{InputIntent, PlayerId, World, V2};

/// 긴 랜덤 입력 시퀀스를 두 개의 독립된 `World`에 똑같이 먹여서 최종 상태가
/// 완전히 같은지 확인한다. 부동소수점이었다면 컴파일러/CPU 최적화 차이로
/// 어긋날 수 있는 지점이다.
#[test]
fn 긴_입력_시퀀스_후에도_두_월드는_같은_상태다() {
    const PLAYER: PlayerId = 0;
    let inputs = synthetic_input_sequence(500);

    let mut a = World::new();
    let mut b = World::new();
    a.spawn(PLAYER, V2::ZERO);
    b.spawn(PLAYER, V2::ZERO);

    for input in &inputs {
        a.step(PLAYER, input);
        a.advance_tick();
        b.step(PLAYER, input);
        b.advance_tick();
    }

    assert_eq!(a.tick, b.tick);
    assert_eq!(a.player(PLAYER).unwrap().pos, b.player(PLAYER).unwrap().pos);
}

/// 선형합동생성기(LCG)로 결정적 의사난수 입력을 만든다. `rand` 크레이트를
/// 끌어오지 않는다 — 이 테스트 파일 자체도 재현 가능해야 한다.
fn synthetic_input_sequence(n: usize) -> Vec<InputIntent> {
    let mut seed: u32 = 0x1234_5678;
    let mut next = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed
    };

    (0..n)
        .map(|_| {
            let r = next();
            let x = ((r & 0xFF) as f32 / 255.0) * 2.0 - 1.0;
            let y = (((r >> 8) & 0xFF) as f32 / 255.0) * 2.0 - 1.0;
            InputIntent {
                move_dir: V2::from_f32(x, y).normalized(),
                action: 0,
                look: None,
            }
        })
        .collect()
}
