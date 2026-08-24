package com.alpefe.fujiptp

/** Small JNI boundary; domain operations will be exposed here as the app UI is added. */
object FujiNative {
    init { System.loadLibrary("fuji_ptp_android") }
    external fun nativeVersion(): String
}
