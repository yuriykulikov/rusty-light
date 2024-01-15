#![no_std]
#![no_main]

use core::mem;

use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_nrf::{bind_interrupts, pac, peripherals, usb};
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::USBD;
use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use embassy_usb::{Builder, Config, UsbDevice};
use embassy_usb::class::cdc_acm;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    POWER_CLOCK => usb::vbus_detect::InterruptHandler;
});

enum LedAction {
    RED,
    GREEN,
    YELLOW,
    BLUE,
    PURPLE,
    BLINK,
}
static CONSOLE_RECEIVER: StaticCell<Channel<NoopRawMutex, ([u8; 64], usize), 1>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    enable_external_osc();

    static LED_CHANNEL: StaticCell<Channel<NoopRawMutex, LedAction, 1>> = StaticCell::new();
    static CONSOLE_OUT_CHANNEL: StaticCell<Channel<NoopRawMutex,  [u8; 64], 1>> = StaticCell::new();

    let led_channel: &mut Channel<NoopRawMutex, LedAction, 1> = LED_CHANNEL.init(Channel::new());
    let _console_out: &mut Channel<NoopRawMutex, [u8; 64], 1> = CONSOLE_OUT_CHANNEL.init(Channel::new());
    // TODO use consistent approach to creating channels
    let console_in = CONSOLE_RECEIVER.init(Channel::new());

    let mut usb_builder = usb_builder(p.USBD);
    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());
    let class = CdcAcmClass::new(&mut usb_builder, state, 64);

    unwrap!(spawner.spawn(usb_task(usb_builder.build())));
    let (_sender, receiver) = class.split();
    unwrap!(spawner.spawn(reader_task(receiver, console_in.sender())));

    unwrap!(spawner.spawn(cli_task(console_in.receiver(), led_channel.sender())));

    unwrap!(spawner.spawn(led_task(
        p.P0_26.degrade(),
        p.P0_30.degrade(),
        p.P0_06.degrade(),
        led_channel.receiver(),
    )));

    unwrap!(spawner.spawn(button_task(
        p.P1_15.degrade(),
        led_channel.sender(),
    )));
}

fn enable_external_osc() {
    let clock: pac::CLOCK = unsafe { mem::transmute(()) };
    clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
    while clock.events_hfclkstarted.read().bits() != 1 {}
}

type USBDDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

/// TODO lifetime
fn usb_builder(usbd: USBD) -> Builder<'static, USBDDriver> {
    let driver = Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs));

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    static DEVICE_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 128]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    Builder::new(
        driver,
        config,
        &mut DEVICE_DESC.init([0; 256])[..],
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 128])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    )
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, USBDDriver>) {
    device.run().await;
}

/// Passed params need to be moved and static
#[embassy_executor::task]
async fn led_task(r: AnyPin, g: AnyPin, b: AnyPin, receiver: Receiver<'static, NoopRawMutex, LedAction, 1>) {
    let mut led_r = Output::new(r, Level::High, OutputDrive::Standard);
    let mut led_g = Output::new(g, Level::High, OutputDrive::Standard);
    let mut led_b = Output::new(b, Level::High, OutputDrive::Standard);

    let mut next_action = receiver.receive().await;
    loop {
        let select = select(
            receiver.receive(),
            handle_action(next_action, &mut led_r, &mut led_g, &mut led_b),
        ).await;

        next_action = match select {
            Either::First(interrupting_action) => interrupting_action,
            Either::Second(()) => receiver.receive().await,
        };
    }
}

async fn handle_action<'a>(led_action: LedAction, led_r: &mut Output<'a, AnyPin>, led_g: &mut Output<'a, AnyPin>, led_b: &mut Output<'a, AnyPin>) {
    led_r.set_high();
    led_g.set_high();
    led_b.set_high();
    match led_action {
        LedAction::RED => {
            led_r.set_low();
            Timer::after(Duration::from_millis(2300)).await;
            led_r.set_high();
        }
        LedAction::GREEN => {
            led_g.set_low();
            Timer::after(Duration::from_millis(2300)).await;
            led_g.set_high();
        }
        LedAction::BLUE => {
            led_b.set_low();
            Timer::after(Duration::from_millis(2300)).await;
            led_b.set_high();
        }
        LedAction::PURPLE => {
            led_b.set_low();
            led_r.set_low();
            Timer::after(Duration::from_millis(2300)).await;
            led_b.set_high();
            led_r.set_high();
        }
        LedAction::YELLOW => {
            led_g.set_low();
            led_r.set_low();
            Timer::after(Duration::from_millis(2300)).await;
            led_g.set_high();
            led_r.set_high();
        }
        LedAction::BLINK => {
            blink(led_r, led_g, led_b).await;
        }
    }
}

async fn blink<'a>(led_r: &mut Output<'a, AnyPin>, led_g: &mut Output<'a, AnyPin>, led_b: &mut Output<'a, AnyPin>) {
    led_b.set_low();
    Timer::after(Duration::from_millis(300)).await;
    led_b.set_high();
    led_g.set_low();
    Timer::after(Duration::from_millis(300)).await;
    led_g.set_high();
    led_r.set_low();
    Timer::after(Duration::from_millis(300)).await;
    led_r.set_high();
}

#[embassy_executor::task]
async fn button_task(pin: AnyPin, led: Sender<'static, NoopRawMutex, LedAction, 1>) {
    let mut input = Input::new(pin, Pull::Up);
    loop {
        input.wait_for_low().await;
        match select(
            Timer::after(Duration::from_millis(500)),
            input.wait_for_high(),
        )
            .await {
            Either::First(_) => {
                // long click
                led.send(LedAction::RED).await;
                // wait for the button to be released
                input.wait_for_high().await;
                // long click released here
            }
            Either::Second(_) => {
                // short click
                led.send(LedAction::BLINK).await;
            }
        }
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

#[embassy_executor::task]
async fn reader_task(
    mut receiver: cdc_acm::Receiver<'static, USBDDriver>,
    console_in_sender: Sender<'static, NoopRawMutex, ([u8; 64], usize), 1>,
) {
    loop {
        receiver.wait_connection().await;
        info!("Connected");
        let _ = try_read(&mut receiver, &console_in_sender).await;
        info!("Disconnected");
    }
}

/// TODO how can I fix lifetimes?
async fn try_read<'a>(
    receiver: &mut cdc_acm::Receiver<'static, USBDDriver>,
    console_in_sender: &Sender<'a, NoopRawMutex, ([u8; 64], usize), 1>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = receiver.read_packet(&mut buf).await?;
        console_in_sender.send((buf, n)).await;
    }
}

#[embassy_executor::task]
async fn cli_task(
    console_in_receiver: Receiver<'static, NoopRawMutex, ([u8; 64], usize), 1>,
    led_sender: Sender<'static, NoopRawMutex, LedAction, 1>,
) {
    loop {
        let (buf, n) = console_in_receiver.receive().await;
        let data = &buf[..n];
        on_console_command(&led_sender, data).await;
    }
}

async fn on_console_command<'a>(led: &Sender<'a, NoopRawMutex, LedAction, 1>, data: &'a [u8]) {
    if data == "reset".as_bytes() {
        let power: pac::POWER = unsafe { mem::transmute(()) };
        power.gpregret.write(|w| unsafe { w.bits(0x57) });
        cortex_m::peripheral::SCB::sys_reset();
    }
    if data == "red".as_bytes() {
        led.send(LedAction::RED).await;
    }
    if data == "green".as_bytes() {
        led.send(LedAction::GREEN).await;
    }
    if data == "blue".as_bytes() {
        led.send(LedAction::BLUE).await;
    }
    if data == "purple".as_bytes() {
        led.send(LedAction::PURPLE).await;
    }
    if data == "yellow".as_bytes() {
        led.send(LedAction::YELLOW).await;
    }
    if data == "blink".as_bytes() {
        led.send(LedAction::BLINK).await;
    }
}
