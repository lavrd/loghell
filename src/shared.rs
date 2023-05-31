use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn now_as_nanos_u64() -> Result<u64, Box<dyn std::error::Error>> {
    let now_as_nanos_u128 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let now_as_nanos_u64 = u64::try_from(now_as_nanos_u128)?;
    Ok(now_as_nanos_u64)
}

pub(crate) fn broadcast<T>(ch: &tokio::sync::broadcast::Sender<T>, data: T) -> Result<(), String> {
    // We compare with "1" because 1 is a default receiver in server struct.
    // For each connection it is incrementing by 1, so 1 connection = 2 receivers.
    if ch.receiver_count() == 1 {
        return Ok(());
    }
    ch.send(data).map_err(|x| x.to_string())?;
    Ok(())
}
