use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::types::{
    mta_get_location_routes_response::GetRoutesForLocationResponse,
    mta_get_routes_response::GetRoutesResponse,
    mta_get_stops_for_route_response::{
        GetStopsForRouteResponse, GetStopsForRouteResponseDataEntryStopGroupingStopGroup,
        GetStopsForRouteResponseDataReferencesStop,
    },
    response_formats,
};

#[derive(Clone)]
pub struct MtaClient {
    host: String,
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

pub struct GetStopsForRouteResultGroupStop {
    pub id: String,
    pub name: String,
}

pub struct GetStopsForRouteResultGroup {
    pub id: String,
    pub name: String,
    pub stops: Vec<GetStopsForRouteResultGroupStop>,
}

pub struct GetStopsForRouteResult {
    pub groups: Vec<GetStopsForRouteResultGroup>,
}

pub struct MtaClientError {
    pub message: String,
}

impl std::fmt::Display for MtaClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MtaClientError: {}", self.message)
    }
}

impl MtaClient {
    pub fn new(host: String, api_key: String) -> Self {
        let request_client = reqwest::Client::new();

        MtaClient {
            host,
            api_key,
            client: request_client,
        }
    }

    pub async fn _get_routes_at_location(
        &self,
        latitude: String,
        longitude: String,
    ) -> Result<TransitRoutes, MtaClientError> {
        let mapped_routes = self
            .client
            .get(&format!(
                "https://bustime.mta.info/api/where/routes-for-location.json?lat={}&lon={}&latSpan=0.005&lonSpan=0.005&key={}",
                latitude,
                longitude,
                self.api_key
            ))
            .send()
            .await
            .map_err(|e| MtaClientError {
                message: e.to_string()
            })?
            .json::<GetRoutesForLocationResponse>()
            .await
            .map_err(|e| MtaClientError {
                message: e.to_string()
            })?
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

    pub async fn get_stops_for_route(
        &self,
        route: String,
    ) -> Result<GetStopsForRouteResult, MtaClientError> {
        let res = self
            .client
            .get(&format!(
                "{}/api/where/stops-for-route/MTA%20NYCT_{}.json?key={}&includePolylines=false&version=2",
                self.host, route, self.api_key
            ))
            .send()
            .await
            .map_err(|e| MtaClientError {
                message: e.to_string(),
            })?;

        let json = match res.error_for_status() {
            Ok(r) => r
                .json::<GetStopsForRouteResponse>()
                .await
                .map_err(|e| MtaClientError {
                    message: e.to_string(),
                })?,
            Err(e) => {
                return Err(MtaClientError {
                    message: e.to_string(),
                })
            }
        };

        let stops_by_id = json
            .data
            .references
            .stops
            .iter()
            .map(|s| (s.id.clone(), s))
            .collect::<HashMap<String, &GetStopsForRouteResponseDataReferencesStop>>();

        let mut result = GetStopsForRouteResult { groups: vec![] };

        for (_, stop_group) in json.data.entry.stopGroupings.iter().enumerate() {
            for (_, stop_group_nested) in stop_group.stopGroups.iter().enumerate() {
                let GetStopsForRouteResponseDataEntryStopGroupingStopGroup {
                    id: grouping_id,
                    name: grouping_name,
                    stopIds: stop_ids,
                } = stop_group_nested;

                let mut group_stops: Vec<GetStopsForRouteResultGroupStop> = vec![];

                for (_, stop_id) in stop_ids.iter().enumerate() {
                    let stop_info = match stops_by_id.get(stop_id) {
                        Some(i) => i,
                        None => continue,
                    };

                    group_stops.push(GetStopsForRouteResultGroupStop {
                        id: stop_id.clone(),
                        name: stop_info.name.clone(),
                    });
                }

                result.groups.push(GetStopsForRouteResultGroup {
                    id: grouping_id.clone(),
                    name: grouping_name.name.clone(),
                    stops: group_stops,
                });
            }
        }

        Ok(result)
    }

    pub async fn get_routes(
        &self,
        search: String,
    ) -> Result<FindTransitRoutesResult, MtaClientError> {
        let res = self
            .client
            .get(&format!(
                "{}/api/where/routes-for-agency/MTA%20NYCT.json?key={}",
                self.host, self.api_key
            ))
            .send()
            .await
            .map_err(|e| MtaClientError {
                message: e.to_string(),
            })?;

        let mapped_routes = match res.error_for_status() {
            Ok(r) => r
                .json::<GetRoutesResponse>()
                .await
                .map_err(|e| MtaClientError {
                    message: e.to_string(),
                })?
                .data
                .list
                .iter()
                .filter(|d| d.shortName.to_lowercase().contains(&search.to_lowercase()))
                .map(|d| FindTransitRoutesResultRoute {
                    id: d.id.clone(),
                    name: d.shortName.clone(),
                })
                .collect(),
            Err(e) => {
                return Err(MtaClientError {
                    message: e.to_string(),
                })
            }
        };

        Ok(FindTransitRoutesResult {
            routes: mapped_routes,
        })
    }

    pub async fn fetch_stop_info(&self) -> Result<Option<StopInformation>, MtaClientError> {
        let res = self
            .client
            .get(&format!(
                "{}/api/siri/stop-monitoring.json?key={}&MonitoringRef=MTA_400080",
                self.host, self.api_key
            ))
            .send()
            .await
            .map_err(|e| MtaClientError {
                message: e.to_string(),
            })?;

        let expected_arrival_time = match res.error_for_status() {
            Ok(r) => match r
                .json::<response_formats::GetStopInfoResponse>()
                .await
                .map_err(|e| MtaClientError {
                    message: e.to_string(),
                })?
                .Siri
                .ServiceDelivery
                .StopMonitoringDelivery
                .get(0)
                .and_then(|f| f.MonitoredStopVisit.get(0))
                .and_then(|d| {
                    d.MonitoredVehicleJourney
                        .MonitoredCall
                        .ExpectedArrivalTime
                        .clone()
                }) {
                Some(s) => s,
                None => return Ok(None),
            },
            Err(e) => {
                return Err(MtaClientError {
                    message: e.to_string(),
                })
            }
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
