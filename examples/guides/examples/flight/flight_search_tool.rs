use chrono::{Duration, Utc};
use rig::tool::{Tool, ToolContext};
use serde::Deserialize;
use serde_json::{json, Value};


#[derive(Deserialize)]
pub struct FlightSearchArgs {
    source: String,
    destination: String,
    date: Option<String>,
}


#[derive(Debug, thiserror::Error)]
pub enum FlightSearchError {
    #[error("RAPIDAPI_KEY is not set")]
    MissingApiKey,
    #[error("flight search request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid request URL: {0}")]
    Url(String),
}


pub struct FlightSearchTool;


impl Tool for FlightSearchTool {
    const NAME: &'static str = "search_flights";


    type Args = FlightSearchArgs;
    type Output = String;
    type Error = FlightSearchError;


    fn description(&self) -> String {
        "Search for one-way flights between two airports".to_string()
    }


    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "source": { "type": "string", "description": "Departure airport IATA code, e.g. 'JFK'" },
                "destination": { "type": "string", "description": "Arrival airport IATA code, e.g. 'LAX'" },
                "date": { "type": "string", "description": "Flight date as YYYY-MM-DD; defaults to 30 days from today" }
            },
            "required": ["source", "destination"]
        })
    }


    async fn call(
        &self,
        _ctx: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let api_key = std::env::var("RAPIDAPI_KEY").map_err(|_| FlightSearchError::MissingApiKey)?;
        let date = args
            .date
            .unwrap_or_else(|| (Utc::now() + Duration::days(30)).format("%Y-%m-%d").to_string());


        let url = reqwest::Url::parse_with_params(
            "https://tripadvisor16.p.rapidapi.com/api/v1/flights/searchFlights",
            [
                ("sourceAirportCode", args.source.as_str()),
                ("destinationAirportCode", args.destination.as_str()),
                ("date", date.as_str()),
                ("itineraryType", "ONE_WAY"),
                ("sortOrder", "PRICE"),
                ("numAdults", "1"),
                ("classOfService", "ECONOMY"),
                ("currencyCode", "USD"),
            ],
        )
        .map_err(|e| FlightSearchError::Url(e.to_string()))?;


        let data: Value = reqwest::Client::new()
            .get(url)
            .header("X-RapidAPI-Host", "tripadvisor16.p.rapidapi.com")
            .header("X-RapidAPI-Key", api_key)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;


        Ok(format_flights(&data))
    }
}


/// Turn the API response into a short list the model can read.
fn format_flights(data: &Value) -> String {
    let flights = data["data"]["flights"].as_array().map(Vec::as_slice).unwrap_or_default();
    let mut out = String::new();
    for flight in flights.iter().take(5) {
        let legs = flight["segments"][0]["legs"].as_array().map(Vec::as_slice).unwrap_or_default();
        let (Some(first), Some(last)) = (legs.first(), legs.last()) else {
            continue;
        };
        out.push_str(&format!(
            "- {} {}{}: departs {} from {}, arrives {} at {}, {} stop(s), {} USD\n",
            first["marketingCarrier"]["displayName"].as_str().unwrap_or("Unknown airline"),
            first["marketingCarrierCode"].as_str().unwrap_or_default(),
            first["flightNumber"],
            first["departureDateTime"].as_str().unwrap_or("?"),
            first["originStationCode"].as_str().unwrap_or("?"),
            last["arrivalDateTime"].as_str().unwrap_or("?"),
            last["destinationStationCode"].as_str().unwrap_or("?"),
            legs.len() - 1,
            flight["purchaseLinks"][0]["totalPrice"],
        ));
    }
    if out.is_empty() {
        "No flights found for that route and date.".to_string()
    } else {
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats_empty_and_one_flight() {
        assert_eq!(format_flights(&json!({})), "No flights found for that route and date.");
        let data = json!({"data":{"flights":[{"segments":[{"legs":[{
            "marketingCarrier":{"displayName":"Test Air"},"marketingCarrierCode":"TA",
            "flightNumber":42,"departureDateTime":"2030-01-01T10:00","originStationCode":"JFK",
            "arrivalDateTime":"2030-01-01T12:00","destinationStationCode":"LAX"
        }]}],"purchaseLinks":[{"totalPrice":123.45}]}]}});
        let rendered=format_flights(&data);
        assert!(rendered.contains("Test Air TA42"));
        assert!(rendered.contains("0 stop(s), 123.45 USD"));
    }
}
