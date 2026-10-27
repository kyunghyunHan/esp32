# Zephyr 개발 환경 설치 (macOS)

> 목표: ESP32 / NXP 등에서 Zephyr RTOS 개발 환경 구축

## Homebrew 패키지 설치

```bash
brew update

brew install python
brew install cmake
brew install ninja
brew install gperf
brew install dtc
brew install ccache
brew install wget
brew install openocd
brew install pipx
```

---

## west 설치

```bash
pipx ensurepath
```

터미널을 재시작

```bash
pipx install west
```

확인

```bash
west --version
```

예시

```text
West version: v1.5.0
```

---

## Zephyr Workspace 생성

```bash
west init -m https://github.com/zephyrproject-rtos/zephyr ~/zephyrproject
```

```bash
cd ~/zephyrproject
```

---

## Zephyr 다운로드

```bash
west update
```

설치 시간이 꽤 오래 걸릴 수 있다.

---

## Python 의존성 설치

```bash
west packages pip --install
```

---

## CMake 등록

```bash
west zephyr-export
```

---

## Zephyr SDK 설치

```bash
cd ~/zephyrproject/zephyr
```

```bash
west sdk install
```

설치 시간이 꽤 오래 걸린다.

## 완료 확인

```bash
west --version
```

```bash
cmake --version
```

```bash
ninja --version
```

```bash
python3 --version
```

모두 정상 출력되면 개발 환경 구축 완료.

다음 단계: [보드 확인·빌드·업로드](west.md), [프로젝트 구조와 쓰레드](application.md)
