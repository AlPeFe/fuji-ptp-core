# fuji-ptp-android

Aplicación Android para leer y escribir las 7 Custom Recipes (C1–C7) de una
**Fujifilm X100VI** por USB. El protocolo PTP/Fujifilm vive 100% en Rust
(`fuji-ptp-core`); Kotlin solo gestiona USB Host, la UI y la persistencia.

```text
Jetpack Compose / Material 3 (Kotlin)
        ↓
FujiUsbManager + UsbIoBridge (bulk IN/OUT)
        ↓ JNI
Rust AndroidTransport (fuji-ptp-android/rust)
        ↓
fuji-ptp-core (FujiPtp, containers PTP, recipes)
```

## Funcionalidades

- **Biblioteca**: crea recipes sin cámara conectada; backlog local ilimitado
  con persistencia Room (SQLite). Duplicar, editar, borrar.
- **7 slots activos**: asigna cualquier recipe de la biblioteca a C1–C7.
- **Cámara**: detecta la X100VI por USB (vendor 0x04CB), pide permiso,
  abre sesión PTP, lee C1–C7 (importa a la biblioteca con dedupe) y escribe
  una recipe completa en cualquier slot. Confirmación explícita antes de
  escribir en la cámara.
- **Controles X100VI**: simulación de película (20), DR (100/200/400),
  Prioridad DR (Off/Auto/Strong/Weak), Grain, Color Chrome, FX Blue, Smooth
  Skin, Balance de blancos (modo, shift R/B, temperatura), Highlight, Shadow,
  Color, Sharpness, Clarity, High ISO NR y WC/MG en monocromo.

## Compilar

```bash
# 1. Bridge Rust → .so (desde la raíz del repo)
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk
ANDROID_NDK_HOME="$ANDROID_HOME/ndk/<versión>" \
  cargo ndk -t arm64-v8a -t x86_64 \
  -o fuji-ptp-android/android/app/src/main/jniLibs \
  build --release --manifest-path fuji-ptp-android/rust/Cargo.toml

# 2. APK
cd fuji-ptp-android/android
./gradlew assembleDebug          # requiere JDK 17+ y Android SDK (local.properties)
```

El dispositivo debe estar en modo USB **«RAW CONV./BACKUP RESTORE»**. Android
no necesita Zadig/WinUSB.

## Arquitectura de la capa nativa

La fachada JNI es pequeña y de alto nivel (JSON como DTO):

| Kotlin (`FujiNative`)         | Rust                                  |
|-------------------------------|---------------------------------------|
| `nativeConnect(bridge, id)`   | guarda `GlobalRef` del puente USB     |
| `nativeOpenSession(id)`       | `FujiPtp::open_session`               |
| `nativeReadRecipes()`         | `FujiPtp::read_recipes` → JSON        |
| `nativeWriteRecipe(slot, json)` | `FujiPtp::write_recipe`             |
| `nativeWriteRecipeNames(json)`| `FujiPtp::write_recipe_names`         |
| `nativeCloseSession()`        | `FujiPtp::close_session`              |
| `nativeClose()`               | libera el controlador                 |

El transporte Android implementa `fuji_ptp_core::transport::Transport` y llama
por JNI a `UsbIo.send(byte[])` / `receive(int)` de Kotlin. El loop de receive
es idéntico al del cliente desktop probado con cámara real.

## Tests

```bash
cargo test --workspace                     # core + bridge (cámara simulada)
```

El bridge incluye tests con una cámara PTP simulada (script de paquetes) que
validan el mapeo DTO y el contrato JSON.

## Pendiente de verificar con hardware

- Mapeo de «Prioridad DR» → valores de cable (Off/Auto/Strong/Weak).
- Escribir Mono WC/MG en simulaciones monocromo (el core ya omite esas
  propiedades para película en color, como exige la cámara real).
