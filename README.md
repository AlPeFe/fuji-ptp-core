# fuji-ptp-core

Librería Rust para leer y escribir las Custom Recipes de cámaras Fujifilm mediante PTP.

El proyecto está separado en un core independiente del transporte y un cliente de escritorio para
probarlo con una cámara real.

## Estado

Probado con una cámara Fujifilm real:

- Lectura de los siete slots C1-C7.
- Lectura de recipes vacías (`0xFFFF` en el bloque de propiedades).
- Lectura y escritura de nombres PTP UTF-16LE.
- Apertura y cierre de sesiones PTP.
- Escritura y lectura de una recipe completa en C7.
- Film Simulation, Grain Effect, Color Chrome, tonos, Clarity y High ISO NR.
- Conversión bidireccional de los valores de High ISO NR.

La compatibilidad de algunas propiedades puede variar según el modelo y firmware. En particular,
la cámara probada rechaza `Mono WC` y `Mono MG` para simulaciones de película en color; el writer
omite esas propiedades en ese caso.

## Arquitectura

```text
Transport
    ↓
PtpSession
    ↓
FujiPtp
    ↓
Profile / Recipe
```

`Transport` es el único punto de integración con el hardware:

```rust
pub trait Transport {
    fn send(&mut self, data: &[u8]) -> Result<(), TransportError>;
    fn receive(&mut self) -> Result<Vec<u8>, TransportError>;
}
```

Esto permite utilizar el mismo core con:

- `MockTransport` para tests.
- `UsbTransport` en el cliente desktop.
- Un futuro `AndroidTransport` para Android/JNI/Flutter.

## Uso del core

```rust
let mut fuji = FujiPtp::new(transport);
fuji.open_session(1)?;

let profile = fuji.read_recipes()?;
fuji.write_recipes(&profile)?;

fuji.close_session()?;
```

El perfil siempre contiene exactamente siete recipes, correspondientes a C1-C7.

## Cliente desktop

El cliente desktop está en `desktop-client/` y utiliza `rusb` para comunicarse con la cámara.

### Requisitos de la cámara

Seleccionar en la cámara:

```text
USB RAW CONV./BACKUP RESTORE
```

En Windows puede ser necesario instalar `WinUSB` para la interfaz `USB PTP Camera` usando
[Zadig](https://zadig.akeo.ie/). Debe comprobarse que el USB ID comienza por `04CB`.

### Comandos

Desde la raíz del proyecto:

```powershell
cargo run -p fuji-ptp-desktop-client -- read
```

Lee y muestra los siete nombres.

```powershell
cargo run -p fuji-ptp-desktop-client -- write
```

Lee los siete perfiles y cambia únicamente sus nombres, anteponiendo `1` a `7`. Sirve como prueba
segura de selección y escritura de nombres.

```powershell
cargo run -p fuji-ptp-desktop-client -- test-slot C7
```

Escribe una recipe nueva completa únicamente en el slot indicado.

```powershell
cargo run -p fuji-ptp-desktop-client -- test-values C7
```

Escribe en un único slot una recipe de prueba con valores conocidos y la lee de nuevo para
verificar las conversiones.

Los comandos `test-slot`, `test-values` y `write` modifican datos de la cámara. No modifican el
firmware.

## Protocolo Fuji utilizado

- `GET_DEVICE_PROP_VALUE = 0x1015`
- `SET_DEVICE_PROP_VALUE = 0x1016`
- Selector de slot: `0xD18C`
- Nombre del preset: `0xD18D`
- Bloque conocido de propiedades: `0xD190..0xD1A2`
- Slots: valores `1..7` codificados como `uint16LE`
- Nombres: strings PTP con longitud y caracteres UTF-16LE

La referencia de protocolo utilizada está en el proyecto
[fujifilm-ptp-recipes](https://github.com/ILFforever/fujifilm-ptp-recipes).

## Tests

```bash
cargo test --workspace
```

Los tests utilizan `MockTransport` y cubren la construcción de containers, transaction IDs,
lectura de data/response y errores de protocolo.

## Android

El directorio `fuji-ptp-android/` contiene la base de integración Android:

```text
fuji-ptp-android/
├── README.md
├── android/                 # módulo Android/Kotlin
│   └── app/src/main/kotlin/
│       └── com/alpefe/fujiptp/
│           ├── FujiUsbManager.kt  # discovery + UsbIoBridge
│           └── FujiNative.kt
└── rust/                    # crate JNI separado
```

Android debe proporcionar la conexión USB mediante `UsbManager`, `UsbDeviceConnection` y
`bulkTransfer`. `FujiUsbManager` busca el vendor Fujifilm `0x04CB`, la interfaz PTP `0x06` y los
endpoints bulk IN/OUT. `UsbIoBridge` mantiene la conexión y deja el protocolo PTP en Rust.

El core no depende de USB, JNI ni APIs Android. La integración Android se mantiene en un crate
separado y debe conectar sus operaciones USB con una implementación de `Transport`. Consulta
[`fuji-ptp-android/README.md`](fuji-ptp-android/README.md) para preparar el módulo y compilar el
bridge con `cargo-ndk`.

## Limitaciones actuales

- El cliente desktop es una herramienta de prueba, no una aplicación final.
- La escritura completa de propiedades puede requerir ajustes por modelo/firmware.
- No se implementan todavía backup/restore de blobs completos de cámara.
- No se incluye UI ni integración Android.
