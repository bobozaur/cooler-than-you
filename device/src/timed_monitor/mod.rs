mod interrupts;
mod monitor;
mod pins;

use arduino_hal::{
    pac::TC0,
    port::{
        Pin,
        mode::{Input, PullUp},
    },
};
use avr_device::interrupt;
use monitor::{ButtonMonitorOps, MonitorButtons, MonitorState};
use pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};

use crate::{interrupt_cell::InterruptCell, shared_state::SHARED_STATE};

/// Timer context that gets setup prior to enabling interrupts
/// and is used exclusively from the timer interrupt.
static MONITOR_CTX: InterruptCell<MonitorContext> = InterruptCell::uninit();

/// Configure timer0 to overflow every millisecond for time tracking
/// and constructs the [`InterruptCell`] used exclusively
/// in the timer overflow interrupt.
///
/// Formula: 16 MHz / (64 * (1 + 249)) = 1000 Hz
pub fn setup_timed_monitor(
    timer: &TC0,
    speed_up_mon_pin: Pin<Input<PullUp>, SpeedUpMonitorPin>,
    speed_down_mon_pin: Pin<Input<PullUp>, SpeedDownMonitorPin>,
    power_mon_pin: Pin<Input<PullUp>, PowerMonitorPin>,
    led_mon_pin: Pin<Input<PullUp>, LedMonitorPin>,
    backlight_mon_pin: Pin<Input<PullUp>, BacklightMonitorPin>,
) {
    // WGM
    timer.tccr0a.write(|w| w.wgm0().bits(0b10));
    timer.tccr0b.write(|w| w.wgm02().clear_bit());

    // Prescaler
    timer.tccr0b.write(|w| w.cs0().prescale_64());
    timer.ocr0a.write(|w| w.bits(249));

    // Enable the timer interrupt
    timer.timsk0.write(|w| w.ocie0a().set_bit());

    // Initialize the timer context.
    MONITOR_CTX.init(MonitorContext::new(
        MonitorButtons::new(
            speed_up_mon_pin,
            speed_down_mon_pin,
            power_mon_pin,
            led_mon_pin,
        ),
        MonitorState::new(backlight_mon_pin),
    ));
}

/// Contains components used exclusively in the timer interrupt.
struct MonitorContext {
    buttons: MonitorButtons,
    state: MonitorState,
}

impl MonitorContext {
    #[inline]
    fn new(buttons: MonitorButtons, state: MonitorState) -> Self {
        Self { buttons, state }
    }

    #[inline]
    fn monitor(&mut self) {
        interrupt::free(|cs| {
            let shared_state = &mut SHARED_STATE.borrow(cs).borrow_mut();
            self.buttons.monitor(shared_state, &mut self.state);
        });
    }
}
