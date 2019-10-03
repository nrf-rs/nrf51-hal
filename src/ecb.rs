//! The AES-ECB peripheral.

use nrf51::ECB;

/// A safe, blocking wrapper around the AES-ECB peripheral.
///
/// It's really just blockwise AES and not an ECB stream cipher. Blocks can be
/// en- and decrypted by calling `crypt_block`.
pub struct AesEcb {
    regs: ECB,
}

impl AesEcb {
    /// Takes ownership of the `ECB` peripheral, returning a safe wrapper.
    pub fn new(regs: ECB) -> Self {
        Self { regs }
    }

    /// Destroys `self`, giving the `ECB` peripheral back.
    pub fn into_inner(self) -> ECB {
        self.regs
    }

    /// Blocking encryption and decryption.
    ///
    /// Encrypts or decrypts `block` with `key` (encryption and decryption are
    /// the same operation).
    ///
    /// # Errors
    ///
    /// An error will be returned when the AES hardware raises an `ERRORECB`
    /// event. This can happen when an operation is started that shares the AES
    /// hardware resources with the AES ECB peripheral while an en-/decryption
    /// operation is running.
    pub fn crypt_block(&mut self, block: [u8; 16], key: [u8; 16]) -> Result<[u8; 16], AesEcbError> {
        // Assumption: No operation is running when this is called.

        #[repr(C)]
        struct EcbData {
            key: [u8; 16],
            cleartext: [u8; 16],
            ciphertext: [u8; 16],
            // Cleartext and Ciphertext are a lie - `cleartext` is always the
            // input, while `ciphertext` is always the output.
        }

        // We allocate the DMA'd buffer on the stack, which means that we must
        // not panic or return before the AES operation is finished.
        let mut buf = EcbData {
            key,
            cleartext: block,
            ciphertext: [0; 16],
        };

        unsafe {
            self.regs.events_endecb.reset(); // acknowledge left-over events
            self.regs.events_errorecb.reset();
            self.regs
                .ecbdataptr
                .write(|w| w.bits(&mut buf as *mut _ as u32));
            self.regs.tasks_startecb.write(|w| w.bits(1));
        }

        loop {
            if self.regs.events_endecb.read().bits() != 0 {
                self.regs.events_endecb.reset();
                return Ok(buf.ciphertext);
            }

            if self.regs.events_errorecb.read().bits() != 0 {
                self.regs.events_errorecb.reset();
                return Err(AesEcbError);
            }
        }
    }
}

/// An `ERRORECB` event was raised during an encryption or decryption operation.
#[derive(Debug)]
pub struct AesEcbError;
