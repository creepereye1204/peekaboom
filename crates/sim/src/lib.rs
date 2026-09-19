//! `peekaboom-sim` — 순수 게임 로직. 의존성 0, no_std 가능(현재는 std 사용, WASM 아님).
//! 네트워크·렌더링을 모른다. 결정적(CB-6): 같은 상태 + 같은 입력 → 같은 결과.
//!
//! 001(연결 수직 슬라이스) 범위: 플레이어 하나를 입력으로 미는 것뿐이다.
//! 지도, 시야, 라운드 규칙은 M2 이후(vis.rs, map.rs는 아직 없음 — tech-debt-tracker TD-012 참조).

pub mod fixed;
pub mod intent;

pub use fixed::{Fx, V2};
pub use intent::{action_flags, InputIntent};

pub type PlayerId = u8;
pub const MAX_PLAYERS: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Player {
    pub pos: V2,
    pub alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Lobby,
}

pub struct World {
    pub tick: u32,
    players: [Player; MAX_PLAYERS],
    pub phase: Phase,
    /// 틱당 이동 거리 (화면 단위, Q16.16). 001에서는 게임 규칙이 없으니 임의 상수.
    pub move_speed: Fx,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> World {
        World {
            tick: 0,
            players: [Player::default(); MAX_PLAYERS],
            phase: Phase::Lobby,
            move_speed: Fx::from_f32(6.0),
        }
    }

    pub fn spawn(&mut self, id: PlayerId, pos: V2) {
        if (id as usize) < MAX_PLAYERS {
            self.players[id as usize] = Player { pos, alive: true };
        }
    }

    pub fn despawn(&mut self, id: PlayerId) {
        if let Some(p) = self.players.get_mut(id as usize) {
            p.alive = false;
        }
    }

    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        self.players.get(id as usize).filter(|p| p.alive)
    }

    /// 한 플레이어를 한 틱 진행시킨다. 결정적: 같은 pos+input → 같은 결과.
    ///
    /// 001 슬라이스에서는 호스트 권위가 없으므로 각 피어가 **자기 자신**에게만 호출한다.
    /// M2에서는 host.rs가 매 틱 전체 피어에 대해 이걸 돌리는 것으로 확장된다.
    pub fn step(&mut self, id: PlayerId, input: &InputIntent) {
        if let Some(p) = self.players.get_mut(id as usize) {
            if p.alive {
                p.pos = p.pos + input.move_dir * self.move_speed;
            }
        }
    }

    /// 로컬 시뮬레이션과 무관하게 원격에서 수신한 좌표로 직접 덮어쓴다.
    /// 001 한정: 원격 피어는 보간 없이 수신값을 그대로 반영한다 (체크리스트 "보간 없음").
    pub fn set_remote_pos(&mut self, id: PlayerId, pos: V2) {
        if let Some(p) = self.players.get_mut(id as usize) {
            if p.alive {
                p.pos = pos;
            }
        }
    }

    pub fn advance_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn iter_players(&self) -> impl Iterator<Item = (PlayerId, &Player)> {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive)
            .map(|(i, p)| (i as PlayerId, p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 같은_입력을_두번_돌리면_같은_결과() {
        let mut a = World::new();
        let mut b = World::new();
        a.spawn(0, V2::ZERO);
        b.spawn(0, V2::ZERO);
        let input = InputIntent {
            move_dir: V2::from_f32(1.0, 0.0),
            action: 0,
            look: None,
        };
        for _ in 0..10 {
            a.step(0, &input);
            b.step(0, &input);
        }
        assert_eq!(a.player(0).unwrap().pos, b.player(0).unwrap().pos);
    }

    #[test]
    fn 이동_없으면_제자리() {
        let mut w = World::new();
        w.spawn(0, V2::from_f32(3.0, 3.0));
        w.step(0, &InputIntent::NONE);
        assert_eq!(w.player(0).unwrap().pos, V2::from_f32(3.0, 3.0));
    }

    #[test]
    fn despawn된_플레이어는_안_보인다() {
        let mut w = World::new();
        w.spawn(0, V2::ZERO);
        w.despawn(0);
        assert!(w.player(0).is_none());
        assert_eq!(w.iter_players().count(), 0);
    }

    #[test]
    fn 원격_좌표는_그대로_덮어써진다() {
        let mut w = World::new();
        w.spawn(1, V2::ZERO);
        w.set_remote_pos(1, V2::from_f32(42.0, -7.0));
        assert_eq!(w.player(1).unwrap().pos, V2::from_f32(42.0, -7.0));
    }
}
