# Zephyr 프로젝트와 쓰레드

개발 환경은 [설치 문서](setup.md), 빌드와 업로드는 [west 명령어](west.md)를 참고한다.

## 프로젝트 구조

    zephyrproject
    │
    ├── zephyr
    ├── 01_blinky
    ├── 02_button
    ├── 03_thread
    ├── 04_uart
    ├── 05_spi
    └── ...

---

## Zephyr 프로젝트

    test
    ├── CMakeLists.txt
    ├── prj.conf
    └── src
        └── main.c

CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})

project(test)

target_sources(app PRIVATE src/main.c)
```

prj.conf

```conf
CONFIG_GPIO=y
CONFIG_SERIAL=y
CONFIG_CONSOLE=y
CONFIG_UART_CONSOLE=y
CONFIG_PRINTK=y
```

---

## RTOS 구조

    Application(Task)

            │

    Zephyr Kernel

            │

    Driver

            │

    Hardware

---

## main()은 누가 실행하는가?

전원 ON

↓

Bootloader

↓

Zephyr Kernel

↓

Scheduler 시작

↓

main()를 첫 번째 Thread로 생성

↓

main() 실행

즉 main()도 커널이 실행하는 Thread이다.

---

## Thread 생성

```c
k_thread_create(...);
```

이 함수는 내가 만드는 것이 아니라

Zephyr Kernel API이다.

Application

↓

k_thread_create()

↓

Zephyr Kernel

↓

Thread 생성

↓

Ready Queue 등록

↓

Scheduler 관리

---

## k_msleep()

```c
k_msleep(1000);
```

현재 Thread를 Sleeping 상태로 변경한다.

Scheduler는 다른 Ready Thread를 실행한다.

---

## Thread 상태

    Running

    ↓

    Sleeping

    ↓

    Ready

    ↓

    Running

---

## Thread 예제

- LED Task
- UART Task

Scheduler가 두 Task를 번갈아 실행한다.

---

## Application 개발자

하는 일

- Task 생성
- 우선순위 설정
- Queue
- Semaphore
- Mutex
- Driver 사용
- GPIO/UART/SPI/I2C 제어

커널은 구현하지 않는다.

---

## Kernel 개발자

구현하는 것

- Scheduler
- Context Switch
- Ready Queue
- Sleep
- Timer
- k_thread_create()
- k_msleep()

---

## Linux와 차이

Linux

Application

↓

System Call

↓

Linux Kernel

↓

Driver

↓

Hardware

Zephyr

Task

↓

Zephyr Kernel

↓

Driver

↓

Hardware

Task가 Driver API를 사용하는 경우가 많다.

---

## 공부 순서

1.  Blinky
2.  GPIO
3.  Thread
4.  Queue
5.  Semaphore
6.  Mutex
7.  Timer
8.  UART
9.  SPI
10. I2C
11. PWM
12. ADC

---

## 면접 포인트

- main()도 Thread이다.
- k_thread_create()는 Kernel API이다.
- k_msleep()은 Sleep 요청이다.
- Scheduler가 Ready Queue를 관리한다.
- RTOS Application 개발자는 Kernel을 구현하지 않는다.
- Thread 우선순위를 설계하는 것이 중요하다.
