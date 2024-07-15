#![allow(non_snake_case)]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MonitoredCall {
    // pub AimedArrivalTime: String,
    pub ExpectedArrivalTime: Option<String>,
}

#[derive(Deserialize)]
pub struct MonitoredVehicleJourney {
    pub MonitoredCall: MonitoredCall,
}

#[derive(Deserialize)]
pub struct MonitoredStopVisit {
    pub MonitoredVehicleJourney: MonitoredVehicleJourney,
}

#[derive(Deserialize)]
pub struct StopMonitoringDelivery {
    pub MonitoredStopVisit: Vec<MonitoredStopVisit>,
    // pub ValidUntil: String,
}

#[derive(Deserialize)]
pub struct ServiceDelivery {
    // pub ResponseTimestamp: String,
    pub StopMonitoringDelivery: Vec<StopMonitoringDelivery>,
}

#[derive(Deserialize)]
pub struct Siri {
    pub ServiceDelivery: ServiceDelivery,
}

#[derive(Deserialize)]
pub struct GetStopInfoResponse {
    pub Siri: Siri,
}
