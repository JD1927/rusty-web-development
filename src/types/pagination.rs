use handle_errors::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Pagination {
    pub start: usize,
    pub end: usize,
}

pub fn extract_pagination(
    params: HashMap<String, String>,
    store_length: usize,
) -> Result<Pagination, Error> {
    if let (Some(start), Some(end)) = (params.get("start"), params.get("end")) {
        let start = start.parse::<usize>().map_err(Error::ParseInt)?;
        let end = end.parse::<usize>().map_err(Error::ParseInt)?;

        if start < end && start <= store_length && end <= store_length {
            return Ok(Pagination { start, end });
        } else {
            return Err(Error::InvalidRange);
        }
    }
    Err(Error::MissingParameters(
        "Pagination requires 'start' and 'end' filters".to_string(),
    ))
}
