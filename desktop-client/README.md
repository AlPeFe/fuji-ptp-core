# fuji-ptp-desktop-client

Cliente mínimo de escritorio para probar `fuji-ptp-core` con una cámara Fujifilm.

La cámara debe estar en el modo USB **USB RAW CONV./BACKUP RESTORE** y exponer una interfaz PTP.

```bash
cargo run -p fuji-ptp-desktop-client -- read
cargo run -p fuji-ptp-desktop-client -- write
```

`read` muestra los nombres de C1-C7. `write` escribe una receta por defecto en los siete slots,
con nombres `recipe1` hasta `recipe7`.

En Windows puede ser necesario instalar WinUSB/libusb para la interfaz PTP con Zadig. El programa
no edita ni solicita datos: `write` es una prueba destructiva sobre los siete slots.
