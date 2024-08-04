#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MonitoredCall {
    // pub AimedArrivalTime: String,
    pub ExpectedArrivalTime: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct MonitoredVehicleJourney {
    pub MonitoredCall: MonitoredCall,
    pub LineRef: String,
    pub DirectionRef: String,
    pub PublishedLineName: String,
}

#[derive(Deserialize, Serialize)]
pub struct MonitoredStopVisit {
    pub MonitoredVehicleJourney: MonitoredVehicleJourney,
}

#[derive(Deserialize, Serialize)]
pub struct StopMonitoringDelivery {
    pub MonitoredStopVisit: Vec<MonitoredStopVisit>,
    // pub ValidUntil: String,
}

#[derive(Deserialize, Serialize)]
pub struct ServiceDelivery {
    // pub ResponseTimestamp: String,
    pub StopMonitoringDelivery: Vec<StopMonitoringDelivery>,
}

#[derive(Deserialize, Serialize)]
pub struct Siri {
    pub ServiceDelivery: ServiceDelivery,
}

#[derive(Deserialize, Serialize)]
pub struct GetStopInfoResponse {
    pub Siri: Siri,
}
