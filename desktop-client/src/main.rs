use fuji_ptp_core::ptp::FujiPtp;
use fuji_ptp_core::recipe::{EffectStrength, FilmSimulation, GrainEffect, Recipe};
use fuji_ptp_core::transport::{Transport, TransportError};
use rusb::{Context, DeviceHandle, UsbContext};
use std::time::Duration;

const FUJI_VENDOR_ID: u16 = 0x04cb;
const PTP_INTERFACE_CLASS: u8 = 0x06;
const TIMEOUT: Duration = Duration::from_secs(5);

struct UsbTransport {
    handle: DeviceHandle<Context>,
    bulk_in: u8,
    bulk_out: u8,
}

impl UsbTransport {
    fn open_fujifilm() -> Result<Self, String> {
        let context = Context::new().map_err(|e| e.to_string())?;
        for device in context.devices().map_err(|e| e.to_string())?.iter() {
            if device
                .device_descriptor()
                .map_err(|e| e.to_string())?
                .vendor_id()
                != FUJI_VENDOR_ID
            {
                continue;
            }
            let config = device
                .active_config_descriptor()
                .map_err(|e| e.to_string())?;
            for interface in config.interfaces() {
                for descriptor in interface.descriptors() {
                    if descriptor.class_code() != PTP_INTERFACE_CLASS {
                        continue;
                    }
                    let mut bulk_in = None;
                    let mut bulk_out = None;
                    for endpoint in descriptor.endpoint_descriptors() {
                        if endpoint.transfer_type() == rusb::TransferType::Bulk {
                            if endpoint.direction() == rusb::Direction::In {
                                bulk_in = Some(endpoint.address());
                            } else {
                                bulk_out = Some(endpoint.address());
                            }
                        }
                    }
                    if let (Some(bulk_in), Some(bulk_out)) = (bulk_in, bulk_out) {
                        let handle = device.open().map_err(|e| e.to_string())?;
                        handle
                            .claim_interface(interface.number())
                            .map_err(|e| e.to_string())?;
                        return Ok(Self {
                            handle,
                            bulk_in,
                            bulk_out,
                        });
                    }
                }
            }
        }
        Err("No se encontró una interfaz PTP Fujifilm (USB RAW CONV./BACKUP RESTORE)".into())
    }
}

impl Transport for UsbTransport {
    fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        self.handle
            .write_bulk(self.bulk_out, data, TIMEOUT)
            .map(|_| ())
            .map_err(|_| TransportError::SendError)
    }
    fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        let mut header = [0u8; 12];
        self.handle
            .read_bulk(self.bulk_in, &mut header, TIMEOUT)
            .map_err(|_| TransportError::ReceiveError)?;
        let length = u32::from_le_bytes(header[0..4].try_into().unwrap()) as usize;
        if !(12..=1024 * 1024).contains(&length) {
            return Err(TransportError::ReceiveError);
        }
        let mut packet = header.to_vec();
        packet.resize(length, 0);
        let mut offset = 12;
        while offset < length {
            let n = self
                .handle
                .read_bulk(self.bulk_in, &mut packet[offset..], TIMEOUT)
                .map_err(|_| TransportError::ReceiveError)?;
            if n == 0 {
                return Err(TransportError::ReceiveError);
            }
            offset += n;
        }
        Ok(packet)
    }
}

fn main() -> Result<(), String> {
    let command = std::env::args().nth(1).unwrap_or_else(|| "read".into());
    let mut fuji = FujiPtp::new(UsbTransport::open_fujifilm()?);
    fuji.open_session(1)
        .map_err(|e| format!("No se pudo abrir la sesión PTP: {e:?}"))?;
    let result = match command.as_str() {
        "read" => match fuji.read_recipes() {
            Ok(profile) => {
                for (i, recipe) in profile.recipes.iter().enumerate() {
                    println!("C{}: {}", i + 1, recipe.name);
                }
                Ok(())
            }
            Err(e) => Err(format!("Error leyendo recipes: {e:?}")),
        },
        "test-values" => {
            let slot = std::env::args().nth(2).and_then(|value| {
                value
                    .strip_prefix('C')
                    .or(Some(value.as_str()))
                    .and_then(|value| value.parse::<u8>().ok())
            });
            let slot = match slot {
                Some(slot @ 1..=7) => slot,
                _ => {
                    return Err(
                        "Uso: cargo run -p fuji-ptp-desktop-client -- test-values C7".into(),
                    );
                }
            };
            println!(
                "ATENCIÓN: se reemplazará únicamente C{} con una recipe de valores de prueba.",
                slot
            );
            let mut recipe = Recipe::new(format!("TEST VALUES C{}", slot));
            recipe.film_simulation = FilmSimulation::Velvia;
            recipe.grain_effect = GrainEffect::StrongSmall;
            recipe.color_chrome = EffectStrength::Weak;
            recipe.color_chrome_fx_blue = EffectStrength::Weak;
            recipe.highlight = 1.0;
            recipe.shadow = -1.0;
            recipe.color = 2.0;
            recipe.sharpness = 1.0;
            recipe.clarity = 1.0;
            recipe.noise_reduction = -1;
            match fuji.write_recipe(slot, &recipe) {
                Ok(()) => match fuji.read_recipes() {
                    Ok(profile) => {
                        let result = &profile.recipes[(slot - 1) as usize];
                        println!("C{} verificado: {}", slot, result.name);
                        println!(
                            "  highlight={} shadow={} color={} sharpness={} clarity={} NR={}",
                            result.highlight,
                            result.shadow,
                            result.color,
                            result.sharpness,
                            result.clarity,
                            result.noise_reduction
                        );
                        Ok(())
                    }
                    Err(e) => Err(format!("Escrito, pero falló la verificación: {e:?}")),
                },
                Err(e) => Err(format!("Falló la escritura de C{}: {e:?}", slot)),
            }
        }
        "test-slot" => {
            let slot = std::env::args().nth(2).and_then(|value| {
                value
                    .strip_prefix('C')
                    .or(Some(value.as_str()))
                    .and_then(|value| value.parse::<u8>().ok())
            });
            let slot = match slot {
                Some(slot @ 1..=7) => slot,
                _ => return Err("Uso: cargo run -p fuji-ptp-desktop-client -- test-slot C7".into()),
            };
            println!(
                "ATENCIÓN: se reemplazará únicamente C{} con una recipe nueva.",
                slot
            );
            let recipe = Recipe::new(format!("TEST C{}", slot));
            match fuji.write_recipe(slot, &recipe) {
                Ok(()) => match fuji.read_recipes() {
                    Ok(profile) => {
                        println!(
                            "C{} después de escribir: {}",
                            slot,
                            profile.recipes[(slot - 1) as usize].name
                        );
                        Ok(())
                    }
                    Err(e) => Err(format!("Escrito, pero falló la verificación: {e:?}")),
                },
                Err(e) => Err(format!("Falló la escritura de C{}: {e:?}", slot)),
            }
        }
        "write" => {
            // Read first so every setting is preserved. Only the names are
            // changed, which makes this a safe round-trip verification.
            let mut profile = match fuji.read_recipes() {
                Ok(profile) => profile,
                Err(e) => return Err(format!("Error leyendo antes de escribir: {e:?}")),
            };
            for (i, recipe) in profile.recipes.iter_mut().enumerate() {
                recipe.name = if recipe.name.is_empty() {
                    format!("{}", i + 1)
                } else {
                    format!("{} {}", i + 1, recipe.name)
                };
            }
            match fuji.write_recipe_names(&profile) {
                Ok(()) => {
                    println!("Se conservaron los datos y se cambiaron los nombres a 1..7");
                    Ok(())
                }
                Err(e) => Err(format!("Error escribiendo recipes: {e:?}")),
            }
        }
        _ => Err(
            "Uso: cargo run -p fuji-ptp-desktop-client -- [read|write|test-slot C7|test-values C7]"
                .into(),
        ),
    };
    let close = fuji
        .close_session()
        .map_err(|e| format!("Error cerrando sesión PTP: {e:?}"));
    result.and(close)
}
