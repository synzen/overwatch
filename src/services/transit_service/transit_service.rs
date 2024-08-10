use std::collections::{HashMap, HashSet};

use ::futures::future::try_join_all;
use chrono::{DateTime, Utc};
use urlencoding::encode;

use crate::{
    services::maps_client::maps_service::MapsService,
    types::lat_long_location::GetStopsAtLocationInput,
};

use super::types::{
    mta_get_routes_response::GetRoutesResponse,
    mta_get_stop_response::GetStopInfoResponse,
    mta_get_stops_at_location_response::GetStopsAtLocationResponse,
    mta_get_stops_for_route_response::{
        GetStopsForRouteResponse, GetStopsForRouteResponseDataEntryStopGroupingStopGroup,
        GetStopsForRouteResponseDataReferencesStop,
    },
};

#[derive(Clone)]
pub struct TransitServiceConfig {
    pub host: String,
    pub api_key: String,
    pub maps_service: MapsService,
}

#[derive(Clone)]
pub struct TransitService {
    config: TransitServiceConfig,
    client: reqwest::Client,
}

pub struct StopInformation {
    pub expected_arrival_time: String,
    pub minutes_until_arrival: i64,
    pub stop_id: String,
    pub route_label: String,
    pub route_id: String,
}

pub struct GetGroupedStopsAtLocation {
    pub groups: Vec<GetStopsForRouteResultGroup>,
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
    pub route_id: String,
    pub route_name: String,
    pub stops: Vec<GetStopsForRouteResultGroupStop>,
}

pub struct GetStopsForRouteResult {
    pub groups: Vec<GetStopsForRouteResultGroup>,
}

pub enum TransitClientError {
    Internal(String),
    ResourceNotFound,
}

impl std::fmt::Display for TransitClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransitClientError::Internal(e) => write!(f, "Internal error: {}", e),
            TransitClientError::ResourceNotFound => write!(f, "Resource not found"),
        }
    }
}

impl TransitService {
    pub fn new(config: TransitServiceConfig) -> Self {
        let request_client = reqwest::Client::new();

        TransitService {
            config,
            client: request_client,
        }
    }

    pub async fn get_stops_at_location(
        &self,
        loc: GetStopsAtLocationInput,
    ) -> Result<GetGroupedStopsAtLocation, TransitClientError> {
        let (lat, lon) = match loc {
            GetStopsAtLocationInput::LatLong(lat, lon) => (lat, lon),
            GetStopsAtLocationInput::GooglePlaceId(loc) => self
                .config
                .maps_service
                .extract_coordinates_from_place_id(&loc)
                .await
                .map_err(|e| {
                    TransitClientError::Internal(format!("Failed to extract coordinates: {}", e))
                })?,
        };

        let url = &format!(
            "{}/api/where/stops-for-location.json?lat={}&lon={}&latSpan=0.005&lonSpan=0.005&key={}",
            self.config.host, lat, lon, self.config.api_key
        );

        let routes_for_location = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                TransitClientError::Internal(format!(
                    "Failed to send stops for location API request: {}",
                    e.to_string()
                ))
            })?
            .json::<GetStopsAtLocationResponse>()
            .await
            .map_err(|e| {
                TransitClientError::Internal(format!(
                    "Failed to parse json for stops-for-location request: {}",
                    e.to_string()
                ))
            })?;

        let route_ids: HashSet<String> =
            routes_for_location
                .data
                .stops
                .iter()
                .fold(HashSet::new(), |mut acc, stop| {
                    stop.routes.iter().for_each(|r| {
                        acc.insert(r.id.clone());
                    });
                    acc
                });

        let stop_ids: HashSet<String> =
            routes_for_location
                .data
                .stops
                .iter()
                .fold(HashSet::new(), |mut acc, stop| {
                    acc.insert(stop.id.clone());
                    acc
                });

        let mut fetches = Vec::new();
        for route_id in route_ids {
            fetches.push(self.get_stops_for_route(route_id));
        }

        let mut result = GetGroupedStopsAtLocation { groups: Vec::new() };

        try_join_all(fetches).await?.iter().for_each(|r| {
            r.groups.iter().for_each(|g| {
                let group = GetStopsForRouteResultGroup {
                    id: g.id.clone(),
                    name: g.name.clone(),
                    route_id: g.route_id.clone(),
                    route_name: g.route_name.clone(),
                    stops: g
                        .stops
                        .iter()
                        .filter(|s| stop_ids.contains(&s.id))
                        .map(|s| GetStopsForRouteResultGroupStop {
                            id: s.id.clone(),
                            name: s.name.clone(),
                        })
                        .collect(),
                };

                result.groups.push(group);
            });
        });

        Ok(result)
    }

    pub async fn get_stops_for_route(
        &self,
        route_id: String,
    ) -> Result<GetStopsForRouteResult, TransitClientError> {
        let res = self
            .client
            .get(&format!(
                "{}/api/where/stops-for-route/{}.json?key={}&includePolylines=false&version=2",
                self.config.host,
                encode(&route_id),
                self.config.api_key
            ))
            .send()
            .await
            .map_err(|e| TransitClientError::Internal(e.to_string()))?;

        let json = match res.error_for_status() {
            Ok(r) => r
                .json::<GetStopsForRouteResponse>()
                .await
                .map_err(|e| TransitClientError::Internal(e.to_string()))?,
            Err(e) => match e.status() {
                Some(s) if s == 404 => {
                    return Err(TransitClientError::ResourceNotFound);
                }
                _ => {
                    return Err(TransitClientError::Internal(e.to_string()));
                }
            },
        };

        let route_name = json
            .data
            .references
            .routes
            .iter()
            .find(|r| r.id == route_id)
            .map(|r| r.shortName.clone())
            .unwrap_or("".to_string());

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
                    route_id: route_id.clone(),
                    route_name: route_name.clone(),
                });
            }
        }

        Ok(result)
    }

    pub async fn get_routes(
        &self,
        search: &str,
    ) -> Result<FindTransitRoutesResult, TransitClientError> {
        let res = self
            .client
            .get(&format!(
                "{}/api/where/routes-for-agency/MTA%20NYCT.json?key={}",
                self.config.host, self.config.api_key
            ))
            .send()
            .await
            .map_err(|e| TransitClientError::Internal(e.to_string()))?;

        let mapped_routes = match res.error_for_status() {
            Ok(r) => r
                .json::<GetRoutesResponse>()
                .await
                .map_err(|e| TransitClientError::Internal(e.to_string()))?
                .data
                .list
                .iter()
                .filter(|d| d.shortName.to_lowercase().contains(&search.to_lowercase()))
                .map(|d| FindTransitRoutesResultRoute {
                    id: d.id.clone(),
                    name: d.shortName.clone(),
                })
                .collect(),
            Err(e) => return Err(TransitClientError::Internal(e.to_string())),
        };

        Ok(FindTransitRoutesResult {
            routes: mapped_routes,
        })
    }

    pub async fn fetch_stop_info(
        &self,
        stop_id: &str,
    ) -> Result<Vec<StopInformation>, TransitClientError> {
        let url = &format!(
            "{}/api/siri/stop-monitoring.json?key={}&MonitoringRef={}",
            self.config.host, self.config.api_key, stop_id
        );

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TransitClientError::Internal(e.to_string()))?
            .error_for_status()
            .map_err(|e| TransitClientError::Internal(e.to_string()))?
            .json::<GetStopInfoResponse>()
            .await
            .map_err(|e| TransitClientError::Internal(e.to_string()))?;

        let stop_monitoring_delivery = response.Siri.ServiceDelivery.StopMonitoringDelivery.first();

        let mut output = Vec::<StopInformation>::new();

        let mut tracked_routes: HashSet<String> = HashSet::<String>::new();

        if let Some(delivery) = stop_monitoring_delivery {
            for stop_visit in delivery.MonitoredStopVisit.iter() {
                let hash_key = format!(
                    "{}{}",
                    stop_visit.MonitoredVehicleJourney.PublishedLineName,
                    stop_visit.MonitoredVehicleJourney.DirectionRef
                );

                if tracked_routes.contains(&hash_key) {
                    continue;
                }

                let minutes_until_arrival = match &stop_visit
                    .MonitoredVehicleJourney
                    .MonitoredCall
                    .ExpectedArrivalTime
                {
                    Some(s) => match DateTime::parse_from_rfc3339(&s) {
                        Ok(d) => Some(d.signed_duration_since(Utc::now()).num_minutes()),
                        Err(_) => Option::None,
                    },
                    None => None,
                };

                let expected_arrival_time = match &stop_visit
                    .MonitoredVehicleJourney
                    .MonitoredCall
                    .ExpectedArrivalTime
                {
                    Some(s) => Some(s.clone()),
                    None => None,
                };

                if let (Some(minutes_until_arrival), Some(expected_arrival_time)) =
                    (minutes_until_arrival, expected_arrival_time)
                {
                    output.push(StopInformation {
                        expected_arrival_time: expected_arrival_time,
                        minutes_until_arrival,
                        route_id: stop_visit.MonitoredVehicleJourney.LineRef.clone(),
                        route_label: stop_visit.MonitoredVehicleJourney.PublishedLineName.clone(),
                        stop_id: stop_id.to_string(),
                    });

                    tracked_routes.insert(hash_key);
                }
            }

            return Ok(output);
        } else {
            return Ok(Vec::new());
        }
    }

    pub async fn fetch_multiple_stop_arrivals(
        &self,
        stop_ids: Vec<&str>,
    ) -> Result<Vec<StopInformation>, TransitClientError> {
        let mut fetches = Vec::new();

        for stop_id in stop_ids {
            fetches.push(self.fetch_stop_info(stop_id));
        }

        let mut output = Vec::<StopInformation>::new();

        try_join_all(fetches)
            .await
            .map_err(|e| TransitClientError::Internal(e.to_string()))?
            .iter()
            .for_each(|v| {
                v.iter().for_each(|s| {
                    output.push(StopInformation {
                        expected_arrival_time: s.expected_arrival_time.clone(),
                        minutes_until_arrival: s.minutes_until_arrival.clone(),
                        route_id: s.route_id.clone(),
                        route_label: s.route_label.clone(),
                        stop_id: s.stop_id.clone(),
                    });
                });
            });

        // sort by lowest minutes until arrival to highest
        output.sort_by(|a, b| a.minutes_until_arrival.cmp(&b.minutes_until_arrival));

        Ok(output)
    }
}
