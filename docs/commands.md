# ESP32 명령 메모

## 

```
. ~/export-esp.sh

```
```
cargo build --example oled
```

```
espflash flash \
  --monitor \
  --port /dev/cu.usbserial-A5069RR4 \
  target/xtensa-esp32-none-elf/debug/examples/led

```

