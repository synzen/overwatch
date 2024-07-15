use crate::types::response_formats;

#[derive(Clone)]
pub struct MtaClient {
    api_key: String,
    client: reqwest::Client,
}

pub struct StopInformation {
    pub expected_arrival_time: String,
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
        Ok(self.client.get(&format!("https://bustime.mta.info/api/siri/stop-monitoring.json?key={}&MonitoringRef=MTA_400080", self.api_key))
            .send()
            .await?
            .json::<response_formats::GetStopInfoResponse>().await?.Siri.ServiceDelivery.StopMonitoringDelivery
            .get(0)
            .and_then(|f| f.MonitoredStopVisit.get(0))
            .and_then(|d|  d.MonitoredVehicleJourney
                    .MonitoredCall
                    .ExpectedArrivalTime
                    .clone()
            )).map(|d| {
              match d {
                Some(s) => Some(StopInformation {
                  expected_arrival_time: s,
                }),
                None => None,
              }
            })
    }
}
