//! `peekaboom-net` — 피어 테이블 + 결정적 호스트 선출.
//!
//! 001(연결 수직 슬라이스) 범위: 호스트 권위 없음, 각자 자기 위치를 브로드캐스트한다
//! (exec-plans/active/001-connection-slice.md "하지 않는 것"). 그래서 이 크레이트는
//! 아직 프로토콜 직렬화(wire.rs)도, 호스트/클라이언트 상태기계(host.rs/client.rs)도
//! 담지 않는다 — M2에서 추가된다 (tech-debt-tracker TD-012).
//!
//! 지금 필요한 건 "누가 호스트가 될지"를 결정적으로 정하는 것뿐이다. 호스트
//! 이양(ARCHITECTURE.md §4)이 M2부터 의미를 가지려면 이 선출 규칙이 처음부터
//! 고정돼 있어야 한다.

/// Trystero의 selfId는 문자열이다.
pub type PeerId = String;

/// 접속 중인 피어 목록.
#[derive(Default, Debug, Clone)]
pub struct PeerTable {
    ids: Vec<PeerId>,
}

impl PeerTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: PeerId) {
        if !self.ids.iter().any(|x| x == &id) {
            self.ids.push(id);
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.ids.retain(|x| x != id);
    }

    pub fn ids(&self) -> &[PeerId] {
        &self.ids
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// 결정적 호스트 선출: 사전순 최소 selfId (ARCHITECTURE.md §4).
    /// 합의 프로토콜이 필요 없다 — 모든 피어가 같은 입력(현재 피어 목록)에서
    /// 같은 결과를 계산하기만 하면 된다.
    pub fn elect_host(&self) -> Option<&PeerId> {
        self.ids.iter().min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 사전순_최소값이_호스트() {
        let mut t = PeerTable::new();
        t.insert("zeta".into());
        t.insert("alpha".into());
        t.insert("mid".into());
        assert_eq!(t.elect_host(), Some(&"alpha".to_string()));
    }

    #[test]
    fn 중복_삽입은_무시된다() {
        let mut t = PeerTable::new();
        t.insert("a".into());
        t.insert("a".into());
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn 제거하면_선출에서_빠진다() {
        let mut t = PeerTable::new();
        t.insert("alpha".into());
        t.insert("beta".into());
        t.remove("alpha");
        assert_eq!(t.elect_host(), Some(&"beta".to_string()));
    }

    #[test]
    fn 비어있으면_호스트_없음() {
        let t = PeerTable::new();
        assert_eq!(t.elect_host(), None);
    }
}
