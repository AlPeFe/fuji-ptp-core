package com.alpefe.fujiptp

import android.content.Context
import android.hardware.usb.UsbConstants
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbDeviceConnection
import android.hardware.usb.UsbEndpoint
import android.hardware.usb.UsbInterface
import android.hardware.usb.UsbManager

/** Android USB Host discovery and bulk I/O. PTP remains in Rust. */
class FujiUsbManager(context: Context) {
    private val usb = context.getSystemService(Context.USB_SERVICE) as UsbManager

    fun findPtpCamera(): UsbDevice? = usb.deviceList.values.firstOrNull { device ->
        device.vendorId == FUJI_VENDOR_ID && findPtpInterface(device) != null
    }

    fun open(device: UsbDevice): UsbIoBridge {
        val iface = findPtpInterface(device) ?: error("Fujifilm PTP interface not found")
        val connection = usb.openDevice(device) ?: error("USB permission denied")
        check(connection.claimInterface(iface.usbInterface, true)) { "Unable to claim PTP interface" }
        return UsbIoBridge(connection, iface.usbInterface, iface.bulkIn, iface.bulkOut)
    }

    private fun findPtpInterface(device: UsbDevice): PtpInterface? {
        for (i in 0 until device.interfaceCount) {
            val iface = device.getInterface(i)
            if (iface.interfaceClass != PTP_CLASS) continue
            var input: UsbEndpoint? = null
            var output: UsbEndpoint? = null
            for (e in 0 until iface.endpointCount) {
                val endpoint = iface.getEndpoint(e)
                if (endpoint.type != UsbConstants.USB_ENDPOINT_XFER_BULK) continue
                if (endpoint.direction == UsbConstants.USB_DIR_IN) input = endpoint else output = endpoint
            }
            if (input != null && output != null) return PtpInterface(iface, input!!, output!!)
        }
        return null
    }

    private data class PtpInterface(val usbInterface: UsbInterface, val bulkIn: UsbEndpoint, val bulkOut: UsbEndpoint)

    companion object { const val FUJI_VENDOR_ID = 0x04CB; private const val PTP_CLASS = 0x06 }
}

class UsbIoBridge(
    private val connection: UsbDeviceConnection,
    private val claimedInterface: UsbInterface,
    private val bulkIn: UsbEndpoint,
    private val bulkOut: UsbEndpoint,
) : AutoCloseable {
    fun send(data: ByteArray): Int = connection.bulkTransfer(bulkOut, data, data.size, TIMEOUT)
    fun receive(size: Int): ByteArray {
        val buffer = ByteArray(size)
        val count = connection.bulkTransfer(bulkIn, buffer, size, TIMEOUT)
        check(count >= 0) { "USB receive failed: $count" }
        return buffer.copyOf(count)
    }
    override fun close() { connection.releaseInterface(claimedInterface) ; connection.close() }
    companion object { private const val TIMEOUT = 5000 }
}
