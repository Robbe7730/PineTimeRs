#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

mod drivers;
mod ui;
mod devicestate;
mod pinetimers;

extern crate alloc;

// This module is basically a pass-through for pinetimers::tasks_impl, which 
// makes it possible to have separate files for tasks.
#[rtic::app(device = nrf52832_hal::pac, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod tasks {
    use crate::drivers::timer::MonoTimer;
    use crate::drivers::display::Display;
    use crate::drivers::touchpanel::TouchPanel;
    use crate::drivers::flash::FlashMemory;
    use crate::drivers::bluetooth::Bluetooth;

    use crate::devicestate::DeviceState;

    use crate::ui::screen::Screen;

    use nrf52832_hal::pac::TIMER0;
    use nrf52832_hal::gpiote::Gpiote;
    use nrf52832_hal::rtc::Rtc;
    use nrf52832_hal::spim::Spim;

    use rubble::link::MIN_PDU_BUF;
    use rubble_nrf5x::radio::PacketBuffer;
    use rubble::link::queue::SimpleQueue;

    use crate::pinetimers::{ConnectedRtc, ConnectedSpim, PixelType};

    use alloc::boxed::Box;

    use spin::Mutex;

    #[monotonic(binds = TIMER0, default = true)]
    type Mono0 = MonoTimer<TIMER0>;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
        rtc: Rtc<ConnectedRtc>,

        display: Display<PixelType, ConnectedSpim>,
        touchpanel: TouchPanel,
        flash: FlashMemory,
        bluetooth: Bluetooth,

        current_screen: Box<dyn Screen<Display<PixelType, ConnectedSpim>>>,
        devicestate: DeviceState,
    }

    #[local]
    struct Local {}

    // I'm using a separate struct and into() to allow the init function
    // to be in a separate crate as Shared and Local cannot be made pub
    impl From<crate::pinetimers::init::Shared> for Shared {
        fn from(init_shared: crate::pinetimers::init::Shared) -> Shared {
            Shared {
                gpiote: init_shared.gpiote,
                rtc: init_shared.rtc,

                display: init_shared.display,
                touchpanel: init_shared.touchpanel,
                flash: init_shared.flash,
                bluetooth: init_shared.bluetooth,

                current_screen: init_shared.current_screen,
                devicestate: init_shared.devicestate
            }
        }
    }

    impl From<crate::pinetimers::init::Local> for Local {
        fn from(_init_local: crate::pinetimers::init::Local) -> Local {
            Local {}
        }
    }

    // Allocate here to make them 'static
    #[init(local = [
            spi_lock: Mutex<Option<Spim<crate::pinetimers::ConnectedSpim>>> = Mutex::new(None),
            ble_tx_buf: PacketBuffer = [0; MIN_PDU_BUF],
            ble_rx_buf: PacketBuffer = [0; MIN_PDU_BUF],
            ble_tx_queue: SimpleQueue = SimpleQueue::new(),
            ble_rx_queue: SimpleQueue = SimpleQueue::new(),
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let (shared, local, mono) = crate::pinetimers::init::init(ctx);

        (shared.into(), local.into(), mono)
    }

    #[idle]
    fn idle(ctx: idle::Context) -> ! {
        crate::pinetimers::tasks_impl::idle(ctx)
    }

    #[task(shared = [display])]
    fn display_init(ctx: display_init::Context) {
        crate::pinetimers::tasks_impl::display_init(ctx)
    }

    #[task(binds = GPIOTE, shared = [gpiote, touchpanel, current_screen, devicestate])]
    fn gpiote_interrupt(ctx: gpiote_interrupt::Context) {
        crate::pinetimers::tasks_impl::gpiote_interrupt(ctx)
    }

    #[task(shared = [devicestate, rtc])]
    fn periodic_update_device_state(ctx: periodic_update_device_state::Context) {
        crate::pinetimers::tasks_impl::periodic_update_device_state(ctx)
    }

    #[task(shared = [display, current_screen, devicestate])]
    fn redraw_screen(ctx: redraw_screen::Context) {
        crate::pinetimers::tasks_impl::redraw_screen(ctx)
    }

    #[task(shared = [display, current_screen, devicestate])]
    fn init_screen(ctx: init_screen::Context) {
        crate::pinetimers::tasks_impl::init_screen(ctx)
    }

    #[task(shared = [flash])]
    fn self_test(ctx: self_test::Context) {
        crate::pinetimers::tasks_impl::self_test(ctx)
    }

    #[task(shared = [current_screen])]
    fn transition(ctx: transition::Context, new_screen: Box<dyn Screen<Display<PixelType, ConnectedSpim>>>) {
        crate::pinetimers::tasks_impl::transition(ctx, new_screen)
    }

    #[task(binds = RADIO, shared = [bluetooth], priority = 3)]
    fn ble_radio(ctx: ble_radio::Context) {
        crate::pinetimers::tasks_impl::ble_radio(ctx)
    }

    #[task(shared = [bluetooth], priority = 2)]
    fn ble_worker(ctx: ble_worker::Context) {
        crate::pinetimers::tasks_impl::ble_worker(ctx)
    }

    #[task(binds = TIMER2, shared = [bluetooth], priority = 3)]
    fn ble_timer(ctx: ble_timer::Context) {
        crate::pinetimers::tasks_impl::ble_timer(ctx)
    }
}

use rtt_target::rprintln;

use core::panic::PanicInfo;

use linked_list_allocator::LockedHeap;

use alloc::alloc::Layout;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("----- PANIC -----");
    rprintln!("{:#?}", info);
    loop {
        cortex_m::asm::bkpt();
    }
}

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    rprintln!("----- OOM -----");
    rprintln!("{:#?}", layout);
    loop {
        cortex_m::asm::bkpt();
    }
}
