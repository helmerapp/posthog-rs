use std::time::Duration;

use reqwest::{header::CONTENT_TYPE, Client as HttpClient};

use crate::{
    event::InnerEvent,
    feature_flag::{DecideRequestData, DecideResponse},
    Error, Event, FeatureFlagPayload,
};

use super::ClientOptions;

pub struct Client {
    options: ClientOptions,
    client: HttpClient,
}

pub async fn client<C: Into<ClientOptions>>(options: C) -> Client {
    let options = options.into();
    let client = HttpClient::builder()
        .timeout(Duration::from_secs(options.request_timeout_seconds))
        .build()
        .unwrap(); // Unwrap here is as safe as `HttpClient::new`
    Client { options, client }
}

impl Client {
    pub async fn capture(&self, event: Event) -> Result<(), Error> {
        let inner_event = InnerEvent::new(event, self.options.api_key.clone());

        let payload =
            serde_json::to_string(&inner_event).map_err(|e| Error::Serialization(e.to_string()))?;

        self.client
            .post(&self.options.api_endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(payload)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        Ok(())
    }

    pub async fn capture_batch(&self, events: Vec<Event>) -> Result<(), Error> {
        let events: Vec<_> = events
            .into_iter()
            .map(|event| InnerEvent::new(event, self.options.api_key.clone()))
            .collect();

        let payload =
            serde_json::to_string(&events).map_err(|e| Error::Serialization(e.to_string()))?;

        self.client
            .post(&self.options.api_endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(payload)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        Ok(())
    }
    pub async fn is_feature_enabled(&self, config: FeatureFlagPayload) -> Result<bool, Error> {
        if config.key.is_empty() || config.distinct_id.is_empty() {
            return Err(Error::Connection(
                "Invalid feature flag payload".to_string(),
            ));
        };

        let request = DecideRequestData {
            api_key: self.options.api_key.clone(),
            distinct_id: config.distinct_id.clone(),
        };

        let response = self.make_decide_request(request).await;
        if let Ok(res) = response {
            if let Some(feature_flag_active) = res.feature_flags.get(&config.key) {
                let mut event = Event::new("$feature_flag_called", config.distinct_id.as_str());
                event.insert_prop("$feature_flag", config.key).unwrap();
                event
                    .insert_prop("$feature_flag_response", feature_flag_active)
                    .unwrap();

                let _ = self.capture(event).await;
                Ok(feature_flag_active.to_owned())
            } else {
                Err(Error::Connection("Feature flag not found".to_string()))
            }
        } else {
            Err(Error::Serialization(response.err().unwrap().to_string()))
        }
    }

    pub async fn make_decide_request(
        &self,
        request: DecideRequestData,
    ) -> Result<DecideResponse, Error> {
        let decide_endpoint = "decide/?v=3";
        let endpoint = self.options.api_endpoint.clone() + decide_endpoint;
        let payload =
            serde_json::to_string(&request).map_err(|e| Error::Serialization(e.to_string()))?;

        let res = self
            .client
            .post(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(payload)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        let res_text = res
            .text()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;
        serde_json::from_str(&res_text).map_err(|e| Error::Serialization(e.to_string()))
    }
}
