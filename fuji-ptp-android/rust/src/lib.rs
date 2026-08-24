//! JNI entry point for the Android integration.
//!
//! The Kotlin `UsbIoBridge` owns `UsbDeviceConnection`. This crate is kept
//! separate from fuji-ptp-core so the core stays platform independent.

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::jstring;

/// Initial smoke-test entry point. The full opaque controller will be added
/// when the Android application is created; protocol code must continue to
/// use `fuji_ptp_core::FujiPtp` and an Android `Transport` implementation.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_alpefe_fujiptp_FujiNative_nativeVersion(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let value = JString::from(
        env.new_string("fuji-ptp-android/0.1.0")
            .expect("JNI string"),
    );
    value.into_raw()
}

#[allow(dead_code)]
fn _transport_boundary_documentation(_bridge: JObject<'_>) {
    // UsbIoBridge.send(byte[]) and receive(int) are the only operations that
    // AndroidTransport needs to call through JNI.
}
