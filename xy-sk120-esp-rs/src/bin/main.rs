#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_hal::{
    clock::CpuClock,
    timer::timg::TimerGroup,
    uart::{Config as UartConfig, Uart},
};

use xy_sk120_esp_rs::{device::DeviceController, modbus::ModbusRtuClient, tui::run_tui};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let console_cfg = UartConfig::default().with_baudrate(115_200);
    let mut console = Uart::new(peripherals.UART0, console_cfg)
        .expect("console uart init failed")
        .into_async();

    let modbus_cfg = UartConfig::default().with_baudrate(9_600);
    let modbus_uart = Uart::new(peripherals.UART1, modbus_cfg)
        .expect("modbus uart init failed")
        .with_rx(peripherals.GPIO7)
        .with_tx(peripherals.GPIO6)
        .into_async();

    let modbus = ModbusRtuClient::new(modbus_uart, 1, Duration::from_millis(8));
    let mut device = DeviceController::new(modbus);

    run_tui(&mut console, &mut device).await
}
