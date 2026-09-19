//! `peekaboom-render` — WebGL2 인스턴스 배칭 렌더러.
//! `peekaboom-sim`의 상태를 **읽기만** 한다. 렌더러가 상태를 바꾸면 결정성이 깨진다.
//!
//! 001(연결 수직 슬라이스) 범위: 단색 사각형 인스턴스 배칭뿐. 텍스처 아틀라스는
//! M2 이후 (`atlas.rs`는 아직 없음 — tech-debt-tracker TD-012).
//!
//! GL을 실제로 다루는 부분은 wasm32 빌드에서만 컴파일된다. 네이티브에서는
//! `batch` 모듈(순수 데이터 변환)만 컴파일 대상이라 `cargo test --workspace`가
//! 통과한다 — repo-layout.md DD-006 "render: 타입체크만" 규칙.

pub mod batch;

#[cfg(target_arch = "wasm32")]
mod gl;

#[cfg(target_arch = "wasm32")]
pub use gl::{Renderer, MAX_DEVICE_PIXEL_RATIO};
