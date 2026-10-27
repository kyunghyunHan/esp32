# Esp32

# ESP32 핀맵과 개발 도구
> 기준 보드: ESP32 DevKit / ESP32-WROOM

## 전원 & 기준 핀
- **GND**: 모든 전기의 기준점 (공통 접지)
- **3V3**: ESP32 동작 전압 (센서 전원용)
- **VIN / 5V**: USB 또는 외부 5V 입력

## I2C
- **SDA**: GPIO21
- **SCL**: GPIO22

## SPI
- **MOSI**: GPIO23
- **MISO**: GPIO19
- **SCK** : GPIO18
- **CS**  : GPIO5

## UART
- **UART0**: GPIO1(TX), GPIO3(RX) – 업로드/로그
- **UART2**: GPIO17(TX), GPIO16(RX) – 외부 장치

## 안전한 GPIO
GPIO 4,5,13,14,16,17,18,19,21,22,23,25,26,27,32,33

## 입력 전용 GPIO
GPIO 34,35,36,39

## 부팅 주의 GPIO
GPIO 0,2,12,15

## ADC
- ADC1: GPIO32~39 (Wi-Fi OK)
- ADC2: Wi-Fi 사용 시 불가

## Deep Sleep Wakeup GPIO
GPIO 0,2,4,12~15,25~27,32~39

## 실전 추천
- LED: GPIO 2,4,16,17
- 버튼: GPIO 4,27,32~39
- 아날로그 센서: GPIO 32~36

## Rust 도구와 시리얼 포트

포트 목록 확인:

```bash
ls /dev/cu.usb*
```

확인한 포트를 지정해 보드 정보를 조회한다. 아래 포트 이름은 기존 환경의 예시다.

```bash
espflash board-info --port /dev/cu.usbserial-A5069RR4
```

포트를 사용 중인 프로세스 확인:

```bash
lsof | grep usbserial
```

OLED·센서 연결 메모는 [부품 배선](docs/peripherals/wiring.md)을 참고한다.


```bash
. ~/export-esp.sh
cd no_std
cargo run --example led_blink
cargo run --example oled
```

기존 예제 이름은 유지했다. [ESP32 예제 목록](no_std/examples/README.md)에서 소스 위치를 확인한다.

## Rust 프로젝트

- [no_std 예제](no_std/README.md): ESP HAL 기반 GPIO·디스플레이·센서·오디오·Wi-Fi
- [ESP-IDF](std/README.md): Wi-Fi·HTTP 서버
- [학습 코드 조각](snippets/README.md)
- [배선](docs/peripherals/wiring.md), [명령 메모](docs/commands.md), [Zephyr](docs/rtos/zephyr/setup.md)

루트에서 `make esp32-check`, `make esp32-build`, `make espidf-check`, `make espidf-build`를 실행한다.
먼저 `. ~/export-esp.sh`를 적용한다. 기존 루트 Rust 프로젝트는 유지한다.
