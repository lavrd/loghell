use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

// todo: delete
pub(crate) type FnRes<T> = Result<T, Box<dyn Error>>;

pub(crate) fn now_as_nanos_u64() -> Result<u64, Box<dyn Error>> {
    let now_as_nanos_u128 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let now_as_nanos_u64 = u64::try_from(now_as_nanos_u128)?;
    Ok(now_as_nanos_u64)
}
