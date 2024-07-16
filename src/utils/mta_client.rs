use chrono::{DateTime, Utc};

use crate::types::response_formats;

#[derive(Clone)]
pub struct MtaClient {
    api_key: String,
    client: reqwest::Client,
}

pub struct StopInformation {
    pub expected_arrival_time: String,
    pub minutes_until_arrival: i64,
}

impl MtaClient {
    pub fn new(api_key: String) -> Self {
        let request_client = reqwest::Client::new();

        MtaClient {
            api_key,
            client: request_client,
        }
    }

    pub async fn fetch_stop_info(&self) -> Result<Option<StopInformation>, reqwest::Error> {
        let expected_arrival_time = match self.client.get(&format!("https://bustime.mta.info/api/siri/stop-monitoring.json?key={}&MonitoringRef=MTA_400080", self.api_key))
            .send()
            .await?
            .json::<response_formats::GetStopInfoResponse>().await?.Siri.ServiceDelivery.StopMonitoringDelivery
            .get(0)
            .and_then(|f| f.MonitoredStopVisit.get(0))
            .and_then(|d|  d.MonitoredVehicleJourney
                    .MonitoredCall
                    .ExpectedArrivalTime
                    .clone()
            ) {
                Some(s) => s,
                None => return Ok(None),
            };

        match DateTime::parse_from_rfc3339(&expected_arrival_time) {
            Ok(d) => {
                let delta = d.signed_duration_since(Utc::now());

                Ok(Some(StopInformation {
                    expected_arrival_time,
                    minutes_until_arrival: delta.num_minutes(),
                }))
            }
            Err(_) => Ok(None),
        }
    }
}
