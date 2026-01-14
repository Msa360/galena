//! Audio device selection and management utilities
//!
//! This module provides common functionality for selecting and reconstructing
//! audio output devices from the AudioDevice enum representation.

use crate::app::AudioDevice;
use cpal::traits::{DeviceTrait, HostTrait};

/// Find audio device by exact name match
pub fn find_audio_device_by_name(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Some(device);
                }
            }
        }
    }
    None
}

/// Reconstruct a cpal::Device from an AudioDevice enum with graceful fallback.
///
/// This function attempts to find a device matching the AudioDevice specification.
/// If the specified device is not found, it gracefully falls back to the default
/// audio device.
///
/// # Arguments
/// * `audio_device` - The AudioDevice enum specifying which device to use
///
/// # Returns
/// Some(device) if found, or None if no audio devices are available at all
pub fn reconstruct_audio_device(audio_device: &AudioDevice) -> Option<cpal::Device> {
    match audio_device {
        AudioDevice::Default => cpal::default_host().default_output_device(),
        AudioDevice::Named { name, .. } => find_audio_device_by_name(name),
    }
}
