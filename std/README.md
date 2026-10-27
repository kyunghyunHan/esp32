# ESP32 ESP-IDF HTTP 서버

ESP-IDF의 표준 라이브러리를 사용하는 별도 Rust 프로젝트다.
루트의 `no_std` 예제와 타깃·링커 설정이 다르므로 이 디렉터리에서 Cargo를 실행한다.

## 빌드와 실행

ESP 툴체인과 `ldproxy`, `espflash`가 필요하다.

```bash
. ~/export-esp.sh
cd std
cargo check --locked
cargo build --locked
cargo run --locked
```

저장소 루트에서는 `make espidf-check`, `make espidf-build`를 사용할 수 있다.
최초 빌드는 `.cargo/config.toml`에 지정한 ESP-IDF SDK와 도구를 준비하므로 시간이 걸린다.

`src/main.rs`의 Wi-Fi 설정을 수정한 뒤 실행한다.
연결되면 시리얼 로그에 표시되는 `http://<보드 IP>/`로 접속한다.
서버는 80번 포트의 `/` 경로에서 HTML을 반환한다.

포맷 검사: `cargo fmt --all -- --check`.
