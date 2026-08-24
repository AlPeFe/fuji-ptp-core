# fuji-ptp-android

Capa Android experimental para `fuji-ptp-core`. El USB se gestiona con Android USB Host y el
protocolo PTP/Fujifilm permanece en Rust.

```text
Jetpack Compose / Kotlin UI
        ↓
FujiUsbManager + UsbIoBridge
        ↓ JNI
Rust AndroidTransport
        ↓
fuji-ptp-core
```

## Estado

Incluye la base de conexión necesaria:

- Detección de Fujifilm (`0x04CB`).
- Búsqueda de interfaz PTP (`0x06`).
- Búsqueda de endpoints bulk IN/OUT.
- Solicitud de permiso USB.
- `claimInterface`.
- Objeto Kotlin `UsbIoBridge` para `bulkTransfer`.
- crate Rust JNI separado que reutiliza `fuji-ptp-core`.

La UI Compose y el empaquetado de las librerías nativas por ABI se añadirán en la aplicación Android.

## Integración

Copia `android/` como módulo Android o integra sus archivos Kotlin en la aplicación. Compila el
bridge Rust para las ABIs necesarias, por ejemplo:

```bash
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 build --release
```

Después coloca las librerías generadas en:

```text
app/src/main/jniLibs/<abi>/libfuji_ptp_android.so
```

El dispositivo debe estar en `USB RAW CONV./BACKUP RESTORE`. Android no necesita Zadig ni WinUSB.

## Permisos y seguridad

El permiso USB debe solicitarse con `UsbManager`; no se concede solo por declarar el feature. La
aplicación debe cerrar la sesión PTP y liberar la interfaz al desconectarse. Las operaciones de
escritura deben tener confirmación explícita en la UI.

Esta capa no duplica ningún código de containers PTP: todas las operaciones deben pasar por
`FujiPtp` del core.
