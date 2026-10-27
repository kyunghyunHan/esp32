# ESP32 예제

루트 프로젝트는 ESP32 no_std (`xtensa-esp32-none-elf`, esp 툴체인) 전용이다.
소스는 주제별로 이동했으며 기존 `cargo run --example <이름>` 명령을 유지한다.

```bash
. ~/export-esp.sh
cargo run --example led_blink
```

| 예제 이름 | 소스 |
|---|---|
| `button_piezo_buzzer` | [esp32/gpio/button_piezo_buzzer.rs ](esp32/gpio/button_piezo_buzzer.rs) |
| `button_polling` | [esp32/gpio/button_polling.rs ](esp32/gpio/button_polling.rs) |
| `interrupt` | [esp32/gpio/interrupt.rs ](esp32/gpio/interrupt.rs) |
| `ioa` | [esp32/gpio/ioa.rs ](esp32/gpio/ioa.rs) |
| `led` | [esp32/gpio/led.rs ](esp32/gpio/led.rs) |
| `led_blink` | [esp32/gpio/led_blink.rs ](esp32/gpio/led_blink.rs) |
| `rotary_encode` | [esp32/gpio/rotary_encode.rs ](esp32/gpio/rotary_encode.rs) |
| `oled` | [esp32/display/oled.rs ](esp32/display/oled.rs) |
| `pir` | [esp32/sensors/pir.rs ](esp32/sensors/pir.rs) |
| `time` | [esp32/sensors/time.rs ](esp32/sensors/time.rs) |
| `sound` | [esp32/audio/sound.rs ](esp32/audio/sound.rs) |
| `sound2` | [esp32/audio/sound2.rs ](esp32/audio/sound2.rs) |
| `sound3` | [esp32/audio/sound3.rs ](esp32/audio/sound3.rs) |
| `wifi` | [esp32/network/wifi.rs ](esp32/network/wifi.rs) |
| `ws` | [esp32/network/ws.rs ](esp32/network/ws.rs) |
| `test` | [esp32/experiments/test.rs ](esp32/experiments/test.rs) |
| `test2` | [esp32/experiments/test2.rs ](esp32/experiments/test2.rs) |
| `test3` | [esp32/experiments/test3.rs ](esp32/experiments/test3.rs) |
| `t4` | [esp32/experiments/t4.rs ](esp32/experiments/t4.rs) |
| `timer` | [esp32/experiments/timer.rs ](esp32/experiments/timer.rs) |
| `test4` | [esp32/async/test4.rs ](esp32/async/test4.rs) |

`experiments/`는 기존 번호별 실험 코드다. 핀과 연결 장치는 각 소스를 확인한다.
모든 예제의 실물 동작을 일괄 검증한 것은 아니다.

미완성 RTOS/시리얼/커널 예시는 [snippets](../../snippets/README.md)에 보존했다.
