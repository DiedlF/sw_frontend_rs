use super::{file_sys::FILE_SYS, timestamp_us};
use core::fmt::Write;
use corelib::{CoreError, DebugLogCode, DebugLogRecord};
use embedded_sdmmc::{Mode, VolumeIdx};
use heapless::String;

pub fn write_debug_log(record: DebugLogRecord) -> Result<(), CoreError> {
    FILE_SYS.lock_during_use(|opt_fs| {
        if let Some(fs) = opt_fs {
            let mut volume = fs
                .vol_mgr()
                .open_volume(VolumeIdx(0))
                .map_err(|_| CoreError::SdCard)?;
            let mut root_dir = volume.open_root_dir().map_err(|_| CoreError::SdCard)?;
            let mut file = root_dir
                .open_file_in_dir("DEBUGLOG.CSV", Mode::ReadWriteCreateOrAppend)
                .map_err(|_| CoreError::SdCard)?;

            let mut line: String<128> = String::new();
            let event = match record.code {
                DebugLogCode::Stats => "stats",
                DebugLogCode::NmeaQueueDrop => "nmea_drop",
                DebugLogCode::NmeaParseError => "nmea_parse_err",
                DebugLogCode::SchedulerOverflow => "scheduler_overflow",
                DebugLogCode::NmeaTxOverflow => "nmea_tx_overflow",
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
