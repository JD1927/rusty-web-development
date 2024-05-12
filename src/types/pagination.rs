use handle_errors::Error;
use std::collections::HashMap;

/// Pagination struc that is gtting extracted
/// from query params
#[derive(Debug)]
pub struct Pagination {
    /// The index of the first item that has to be returned
    pub start: usize,
    /// The index of the last item that has to be returned
    pub end: usize,
}

/// Extract query parameters from the `/questions` route
/// # Example query
/// GET requests to this route can have a pagination attached so we just
/// return the questions we need
/// `/questions?start=1&end=10`
/// # Example usage
/// ```rust
/// let mut query = HashMap::new();
/// query.insert("start".to_string(), "1".to_string());
/// query.insert("end".to_string(), "10".to_string());
/// let p = types::pagination::extract_pagination(query).unwrap();
/// assert_eq!(p.start, 1);
/// assert_eq!(p.end, 10);
/// ```
pub fn extract_pagination(
    params: HashMap<String, String>,
    store_length: usize,
) -> Result<Pagination, Error> {
    if let (Some(start), Some(end)) = (params.get("start"), params.get("end")) {
        // Takes the "start" parameter in the query
        // and tries to convert it to a number
        let start = start.parse::<usize>().map_err(Error::ParseInt)?;
        // Takes the "start" parameter in the query
        // and tries to convert it to a number
        let end = end.parse::<usize>().map_err(Error::ParseInt)?;

        // Validates if the start and end are different and in the store length
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
