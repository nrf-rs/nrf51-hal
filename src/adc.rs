//! # API for the Analog to Digital converter

use embedded_hal::adc::{Channel, OneShot};

use crate::gpio::{gpio, Floating, Input};

use crate::nrf51::ADC;

/// ADC Configuration
pub struct Adc {
    adc: ADC,
    resolution: AdcResolution,
    input_selection: AdcInputSelection,
    reference_selection: AdcReferenceSelection,
}

/// ADC Sample resolution
pub enum AdcResolution {
    /// 8 bit sample resolution
    Res8bit,
    /// 9 bit sample resolution
    Res9bit,
    /// 10 bit sample resolution
    Res10bit,
}

impl AdcResolution {
    /// Gets the default sample resolution (currently 10 bits)
    pub fn default() -> Self {
        AdcResolution::Res10bit
    }
}

/// ADC Input Selection (analog or supply prescaling)
pub enum AdcInputSelection {
    /// Use analog input pin without prescaling
    AnalogInputNoPrescaling,
    /// Use analog input pin with 2/3 prescaling
    AnalogInputTwoThirdsPrescaling,
    /// Use analog input pin with 1/3 prescaling
    AnalogInputOneThirdPrescaling,
    /// Use VDD with 2/3 prescaling
    SupplyTwoThirdsPrescaling,
    /// Use VDD with 1/3 prescaling
    SupplyOneThirdPrescaling,
}

impl AdcInputSelection {
    /// Gets the default input selection (currently analog input with 1/3 prescaling)
    pub fn default() -> Self {
        AdcInputSelection::AnalogInputOneThirdPrescaling
    }
}

/// ADC reference selection
pub enum AdcReferenceSelection {
    /// Use internal 1.2 V band gap reference
    VBG,
    /// Use external reference specified by CONFIG.EXTREFSEL
    External,
    /// Use VDD with 1/2 prescaling. (Only application when VDD is in the range 1.7 V - 2.6 V)
    SupplyOneHalfPrescaling,
    /// Use VDD with 1/3 prescaling. (Only application when VDD is in the range 2.5 V - 3.6 V)
    SupplyOneThirdPrescaling,
}

impl AdcReferenceSelection {
    /// Gets the default reference selection (currently VDD with 1/3 prescaling)
    pub fn default() -> Self {
        AdcReferenceSelection::SupplyOneThirdPrescaling
    }
}

impl Adc {
    /// Initialises a new Adc
    ///
    /// Sets all configurable parameters to defaults, waits for ADC to be ready
    pub fn default(adc: ADC) -> Self {
        Adc::with_config(
            adc,
            AdcResolution::default(),
            AdcInputSelection::default(),
            AdcReferenceSelection::default(),
        )
    }

    /// Initialises a new Adc
    ///
    /// Waits for ADC to be ready
    pub fn with_config(
        adc: ADC,
        resolution: AdcResolution,
        input_selection: AdcInputSelection,
        reference_selection: AdcReferenceSelection,
    ) -> Self {
        let s = Self {
            adc,
            resolution,
            input_selection,
            reference_selection,
        };

        while s.adc.busy.read().busy().is_busy() {}

        s.adc.config.write(|w| {
            let w1 = match s.resolution {
                AdcResolution::Res8bit => w.res()._8bit(),
                AdcResolution::Res9bit => w.res()._9bit(),
                AdcResolution::Res10bit => w.res()._10bit(),
            };

            let w2 = match s.input_selection {
                AdcInputSelection::AnalogInputNoPrescaling => {
                    w1.inpsel().analog_input_no_prescaling()
                }
                AdcInputSelection::AnalogInputTwoThirdsPrescaling => {
                    w1.inpsel().analog_input_two_thirds_prescaling()
                }
                AdcInputSelection::AnalogInputOneThirdPrescaling => {
                    w1.inpsel().analog_input_one_third_prescaling()
                }
                AdcInputSelection::SupplyTwoThirdsPrescaling => {
                    w1.inpsel().supply_two_thirds_prescaling()
                }
                AdcInputSelection::SupplyOneThirdPrescaling => {
                    w1.inpsel().supply_one_third_prescaling()
                }
            };

            let w3 = match s.reference_selection {
                AdcReferenceSelection::VBG => w2.refsel().vbg(),
                AdcReferenceSelection::External => w2.refsel().external(),
                AdcReferenceSelection::SupplyOneHalfPrescaling => {
                    w2.refsel().supply_one_half_prescaling()
                }
                AdcReferenceSelection::SupplyOneThirdPrescaling => {
                    w2.refsel().supply_one_third_prescaling()
                }
            };

            w3
        });

        s.adc.enable.write(|w| w.enable().enabled());
        s
    }

    /// Sets the ADC resolution
    ///
    /// Options can be found in [AdcResolution](crate::adc::AdcResolution)
    pub fn set_resolution(&mut self, resolution: AdcResolution) {
        self.resolution = resolution;

        self.adc.config.modify(|_, w| match self.resolution {
            AdcResolution::Res8bit => w.res()._8bit(),
            AdcResolution::Res9bit => w.res()._9bit(),
            AdcResolution::Res10bit => w.res()._10bit(),
        })
    }

    /// Sets the ADC input selection
    ///
    /// Options can be found in [AdcInputSelection](crate::adc::AdcInputSelection)
    pub fn set_input_selection(&mut self, input_selection: AdcInputSelection) {
        self.input_selection = input_selection;

        self.adc.config.modify(|_, w| match self.input_selection {
            AdcInputSelection::AnalogInputNoPrescaling => w.inpsel().analog_input_no_prescaling(),
            AdcInputSelection::AnalogInputTwoThirdsPrescaling => {
                w.inpsel().analog_input_two_thirds_prescaling()
            }
            AdcInputSelection::AnalogInputOneThirdPrescaling => {
                w.inpsel().analog_input_one_third_prescaling()
            }
            AdcInputSelection::SupplyTwoThirdsPrescaling => {
                w.inpsel().supply_two_thirds_prescaling()
            }
            AdcInputSelection::SupplyOneThirdPrescaling => w.inpsel().supply_one_third_prescaling(),
        })
    }

    /// Sets the ADC reference selection
    ///
    /// Options can be found in [AdcReferenceSelection](crate::adc::AdcReferenceSelection)
    pub fn set_reference_selection(&mut self, reference_selection: AdcReferenceSelection) {
        self.reference_selection = reference_selection;

        self.adc
            .config
            .modify(|_, w| match self.reference_selection {
                AdcReferenceSelection::VBG => w.refsel().vbg(),
                AdcReferenceSelection::External => w.refsel().external(),
                AdcReferenceSelection::SupplyOneHalfPrescaling => {
                    w.refsel().supply_one_half_prescaling()
                }
                AdcReferenceSelection::SupplyOneThirdPrescaling => {
                    w.refsel().supply_one_third_prescaling()
                }
            })
    }

    fn set_channel(&mut self, channel: u8) {
        match channel {
            0 => self.adc.config.modify(|_, w| w.psel().analog_input0()),
            1 => self.adc.config.modify(|_, w| w.psel().analog_input1()),
            2 => self.adc.config.modify(|_, w| w.psel().analog_input2()),
            3 => self.adc.config.modify(|_, w| w.psel().analog_input3()),
            4 => self.adc.config.modify(|_, w| w.psel().analog_input4()),
            5 => self.adc.config.modify(|_, w| w.psel().analog_input5()),
            6 => self.adc.config.modify(|_, w| w.psel().analog_input6()),
            7 => self.adc.config.modify(|_, w| w.psel().analog_input7()),
            _ => unreachable!(),
        }
    }

    fn convert(&mut self, channel: u8) -> u16 {
        self.set_channel(channel);
        self.adc.events_end.write(|w| unsafe { w.bits(0) });
        self.adc.tasks_start.write(|w| unsafe { w.bits(1) });

        while self.adc.events_end.read().bits() == 0 {}

        self.adc.events_end.write(|w| unsafe { w.bits(0) });
        self.adc.result.read().result().bits()
    }

    /// Disables the ADC and releases the ADC peripheral
    pub fn release(self) -> ADC {
        self.adc.enable.write(|w| w.enable().disabled());
        self.adc
    }
}

impl<WORD, PIN> OneShot<ADC, WORD, PIN> for Adc
where
    WORD: From<u16>,
    PIN: Channel<ADC, ID = u8>,
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        let res = self.convert(PIN::channel());
        Ok(res.into())
    }
}

macro_rules! adc_pins {
    ($($pin:ty => $chan:expr),+ $(,)*) => {
        $(
            impl Channel<ADC> for $pin {
                type ID = u8;

                fn channel() -> u8 { $chan }
            }
        )+
    };
}

adc_pins!(
    gpio::PIN26<Input<Floating>> => 0_u8,
    gpio::PIN27<Input<Floating>> => 1_u8,
    gpio::PIN1<Input<Floating>> => 2u8,
    gpio::PIN2<Input<Floating>> => 3u8,
    gpio::PIN3<Input<Floating>> => 4u8,
    gpio::PIN4<Input<Floating>> => 5u8,
    gpio::PIN5<Input<Floating>> => 6u8,
    gpio::PIN6<Input<Floating>> => 7u8,
);
