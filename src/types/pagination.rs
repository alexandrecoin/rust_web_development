use crate::error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Pagination {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub fn extract_pagination(
    params: HashMap<String, String>,
    max_possible_length: usize,
) -> Result<Pagination, error::Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let start = params
            .get("start")
            .unwrap()
            .parse::<usize>()
            .map_err(error::Error::ParseError)?;
        let end = params
            .get("end")
            .unwrap()
            .parse::<usize>()
            .map_err(error::Error::ParseError)?;

        if start > end {
            return Err(error::Error::NonProcessable);
        };

        if end > max_possible_length {
            return Err(error::Error::OutOfBounds);
        };

        return Ok(Pagination { start, end });
    }
    Err(error::Error::MissingParameters)
}
