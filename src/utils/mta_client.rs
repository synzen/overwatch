use chrono::{DateTime, Utc};

use crate::types::{
    mta_get_location_routes_response::GetRoutesForLocationResponse,
    mta_get_routes_response::GetRoutesResponse, response_formats,
};

#[derive(Clone)]
pub struct MtaClient {
    api_key: String,
    client: reqwest::Client,
}

pub struct StopInformation {
    pub expected_arrival_time: String,
    pub minutes_until_arrival: i64,
}

pub struct TransitRoute {
    pub name: String,
    pub description: String,
    pub id: String,
}

pub struct TransitRoutes {
    pub routes: Vec<TransitRoute>,
}

pub struct FindTransitRoutesResultRoute {
    pub id: String,
    pub name: String,
}

pub struct FindTransitRoutesResult {
    pub routes: Vec<FindTransitRoutesResultRoute>,
}

impl MtaClient {
    pub fn new(api_key: String) -> Self {
        let request_client = reqwest::Client::new();

        MtaClient {
            api_key,
            client: request_client,
        }
    }

    pub async fn _get_routes_at_location(
        &self,
        latitude: String,
        longitude: String,
    ) -> Result<TransitRoutes, reqwest::Error> {
        let mapped_routes = self
            .client
            .get(&format!(
                "https://bustime.mta.info/api/where/routes-for-location.json?lat={}&lon={}&latSpan=0.005&lonSpan=0.005&key={}",
                latitude,
                longitude,
                self.api_key
            ))
            .send()
            .await?
            .json::<GetRoutesForLocationResponse>()
            .await?
            .data
            .routes
            .iter()
            .map(|d| TransitRoute {
                name: d.shortName.clone(),
                description: d.description.clone(),
                id: d.id.clone(),
            })
            .collect();

        Ok(TransitRoutes {
            routes: mapped_routes,
        })
    }

    pub async fn get_routes(
        &self,
        search: String,
    ) -> Result<FindTransitRoutesResult, reqwest::Error> {
        let mapped_routes = self
            .client
            .get(&format!(
                "https://bustime.mta.info/api/where/routes-for-agency/MTA%20NYCT.json?key={}",
                self.api_key
            ))
            .send()
            .await?
            .json::<GetRoutesResponse>()
            .await?
            .data
            .list
            .iter()
            .filter(|d| d.shortName.to_lowercase().contains(&search.to_lowercase()))
            .map(|d| FindTransitRoutesResultRoute {
                id: d.id.clone(),
                name: d.shortName.clone(),
            })
            .collect();

        Ok(FindTransitRoutesResult {
            routes: mapped_routes,
        })
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
