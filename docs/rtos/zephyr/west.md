# west 빌드·업로드·모니터링

개발 환경 준비는 [Zephyr 설치](setup.md)를 참고한다.

## 환경 변수

```bash
export ZEPHYR_BASE="$HOME/zephyrproject/zephyr"
```

## ESP32 보드 확인

```bash
cd ~/zephyrproject/zephyr
west boards | grep -i esp32
```

## ESP32 Blinky 예제

Zephyr 소스 디렉터리에서 실행한다.

```bash
west build -p always -b doit_esp32_devkit_v1/esp32/procpu samples/basic/blinky
```

## 애플리케이션 빌드

각 애플리케이션의 `CMakeLists.txt`가 있는 디렉터리에서 실행한다.

ESP32:

```bash
west build -p always -b doit_esp32_devkit_v1/esp32/procpu .
```

NXP FRDM-MCXN947:

```bash
west build -p always -b frdm_mcxn947/mcxn947/cpu0 .
```

## 업로드

```bash
west flash
```

NXP 디버그 프로브 메모: LinkServer probes.

## ESP32 모니터

```bash
west espressif monitor
```

## NXP 시리얼 연결

NXP 관련 메모는 기존 embedded 저장소의 문서를 참고한다.
