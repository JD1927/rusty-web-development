use handle_errors::Error;
use std::collections::HashMap;

/// Pagination struct that is getting extracted
/// from query params
#[derive(Default, Debug, PartialEq)]
pub struct Pagination {
    /// The index of the last item which has to be returned
    pub limit: Option<i32>,
    /// The index of the first item which has to be returned
    pub offset: i32,
}

/// Extract query parameters from the `/questions` route
/// # Example query
/// GET requests to this route can have a pagination attached so we just
/// return the questions we need
/// `/questions?start=1&end=10`
/// # Example usage
/// ```rust
/// let mut query = HashMap::new();
/// query.insert("limit".to_string(), "1".to_string());
/// query.insert("offset".to_string(), "10".to_string());
/// let p = types::pagination::extract_pagination(query).unwrap();
/// assert_eq!(p.limit, 1);
/// assert_eq!(p.offset, 10);
/// ```

const PAGINATION_ERROR: &str =
    "Pagination requires 'limit' and 'offset' params!";

pub fn extract_pagination(
    params: HashMap<String, String>,
) -> Result<Pagination, Error> {
    // Could be improved in the future
    if params.contains_key("limit") && params.contains_key("offset") {
        return Ok(Pagination {
            // Takes the "limit" parameter in the query and tries to convert it to a number
            limit: Some(
                params
                    .get("limit")
                    .unwrap()
                    .parse::<i32>()
                    .map_err(Error::ParseInt)?,
            ),
            // Takes the "offset" parameter in the query and tries to convert it to a number
            offset: params
                .get("offset")
                .unwrap()
                .parse::<i32>()
                .map_err(Error::ParseInt)?,
        });
    }
    Err(Error::MissingParameters(PAGINATION_ERROR.to_string()))
}

mod pagination_tests {
    use super::{
        extract_pagination, Error, HashMap, Pagination, PAGINATION_ERROR,
    };

    #[test]
    fn valid_pagination() {
        // Arrange
        let mut params = HashMap::new();
        params.insert(String::from("limit"), String::from("1"));
        params.insert(String::from("offset"), String::from("1"));
        let expected = Pagination {
            limit: Some(1),
            offset: 1,
        };
        // Act
        let pagination_result = extract_pagination(params);
        // Assert
        assert_eq!(pagination_result.unwrap(), expected);
    }

    #[test]
    fn missing_offset_parameter() {
        // Arrange
        let mut params = HashMap::new();
        params.insert(String::from("limit"), String::from("1"));
        let expected = format!(
            "{}",
            Error::MissingParameters(PAGINATION_ERROR.to_string())
        );
        // Act
        let pagination_result =
            format!("{}", extract_pagination(params).unwrap_err());
        // Assert
        assert_eq!(pagination_result, expected);
    }
    #[test]
    fn missing_limit_parameter() {
        // Arrange
        let mut params = HashMap::new();
        params.insert(String::from("offset"), String::from("1"));
        let expected = format!(
            "{}",
            Error::MissingParameters(PAGINATION_ERROR.to_string())
        );
        // Act
        let pagination_result =
            format!("{}", extract_pagination(params).unwrap_err());
        // Assert
        assert_eq!(pagination_result, expected);
    }

    #[test]
    fn wrong_offset_type() {
        // Arrange
        let mut params = HashMap::new();
        params.insert(String::from("limit"), String::from("1"));
        params
            .insert(String::from("offset"), String::from("NOT_A_NUMBER"));
        let expected = String::from(
            "Cannot parse parameter: invalid digit found in string",
        );
        // Act
        let pagination_result =
            format!("{}", extract_pagination(params).unwrap_err());
        // Assert
        assert_eq!(pagination_result, expected);
    }

    #[test]
    fn wrong_limit_type() {
        // Arrange
        let mut params = HashMap::new();
        params.insert(String::from("limit"), String::from("NOT_A_NUMBER"));
        params.insert(String::from("offset"), String::from("1"));
        let expected = String::from(
            "Cannot parse parameter: invalid digit found in string",
        );
        // Act
        let pagination_result =
            format!("{}", extract_pagination(params).unwrap_err());
        // Assert
        assert_eq!(pagination_result, expected);
    }
}
