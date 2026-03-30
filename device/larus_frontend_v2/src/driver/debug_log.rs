use super::{file_sys::FILE_SYS, timestamp_us};
use core::fmt::Write;
use corelib::{CoreError, DebugLogRecord, EventLogCode, EventLogRecord};
use embedded_sdmmc::{Mode, VolumeIdx};
use heapless::String;

pub fn write_debug_log(_record: DebugLogRecord) -> Result<(), CoreError> {
    Ok(())
}

pub fn write_event_log(record: EventLogRecord) -> Result<(), CoreError> {
    FILE_SYS.lock_during_use(|opt_fs| {
        if let Some(fs) = opt_fs {
            let mut volume = fs
                .vol_mgr()
                .open_volume(VolumeIdx(0))
                .map_err(|_| CoreError::SdCard)?;
            let mut root_dir = volume.open_root_dir().map_err(|_| CoreError::SdCard)?;
            let mut file = root_dir
                .open_file_in_dir("EVENTLOG.CSV", Mode::ReadWriteCreateOrAppend)
                .map_err(|_| CoreError::SdCard)?;

            let mut line: String<160> = String::new();
            let event = match record.code {
                EventLogCode::ThermalReset => "thermal_reset",
                EventLogCode::ThermalCircleComplete => "thermal_circle_complete",
                EventLogCode::ThermalEstimateValid => "thermal_est_valid",
                EventLogCode::ThermalEstimateInvalid => "thermal_est_invalid",
                EventLogCode::FlyModeChanged => "fly_mode_changed",
                EventLogCode::GpsStateChanged => "gps_state_changed",
                EventLogCode::UpdateFound => "update_found",
                EventLogCode::UpdateInstallStart => "update_install_start",
            };
            let _ = writeln!(
                line,
                "{},{},{},{},{},{}",
                timestamp_us(), event, record.a, record.b, record.c, record.d
            );
            file.write(line.as_bytes()).map_err(|_| CoreError::SdCard)
        } else {
            Err(CoreError::SdCard)
        }
    })
}
