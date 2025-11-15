// Event stream consumers for NATS and Redis

use backoff::{exponential::ExponentialBackoff, SystemClock};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::domain::Result;

use super::models::StreamMessage;
use super::traits::IngestionHandler;

/// NATS stream consumer
pub struct NatsConsumer<H: IngestionHandler> {
    client: async_nats::Client,
    subject: String,
    handler: Arc<H>,
    backoff: ExponentialBackoff<SystemClock>,
}

impl<H: IngestionHandler> NatsConsumer<H> {
    /// Create a new NATS consumer
    pub async fn new(urls: &[String], subject: String, handler: H) -> Result<Self> {
        use crate::domain::CostOpsError;

        info!(
            urls = ?urls,
            subject = %subject,
            "Connecting to NATS server"
        );

        let client = async_nats::connect(urls.join(",")).await
            .map_err(|e| CostOpsError::Integration(format!("NATS connection failed: {}", e)))?;

        let backoff = ExponentialBackoff {
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(30),
            max_elapsed_time: Some(Duration::from_secs(300)),
            multiplier: 2.0,
            ..Default::default()
        };

        Ok(Self {
            client,
            subject,
            handler: Arc::new(handler),
            backoff,
        })
    }

    /// Start consuming messages from NATS
    pub async fn start(&mut self) -> Result<()> {
        use crate::domain::CostOpsError;
        use futures::StreamExt;

        info!(subject = %self.subject, "Starting NATS consumer");

        let mut subscriber = self.client.subscribe(self.subject.clone()).await
            .map_err(|e| CostOpsError::Integration(format!("NATS subscribe failed: {}", e)))?;

        while let Some(message) = subscriber.next().await {
            match self.process_message(message).await {
                Ok(_) => {}
                Err(e) => {
                    error!(error = %e, "Failed to process NATS message");
                }
            }
        }

        Ok(())
    }

    /// Process a single NATS message
    async fn process_message(&self, message: async_nats::Message) -> Result<()> {
        use crate::domain::CostOpsError;

        let stream_msg: StreamMessage = serde_json::from_slice(&message.payload)?;

        info!(
            message_id = %stream_msg.message_id,
            event_type = ?stream_msg.event_type,
            retry_count = stream_msg.retry_count,
            "Processing NATS message"
        );

        match self.handler.handle_single(stream_msg.payload).await {
            Ok(response) => {
                info!(
                    message_id = %stream_msg.message_id,
                    status = ?response.status,
                    "Successfully processed NATS message"
                );

                // Acknowledge message
                if let Some(reply) = message.reply {
                    self.client.publish(reply, "ACK".into()).await
                        .map_err(|e| CostOpsError::Integration(format!("NATS publish failed: {}", e)))?;
                }

                Ok(())
            }
            Err(e) => {
                error!(
                    message_id = %stream_msg.message_id,
                    error = %e,
                    "Failed to process NATS message"
                );

                // Could implement retry logic here
                Err(e)
            }
        }
    }
}

/// Redis Streams consumer
pub struct RedisConsumer<H: IngestionHandler> {
    client: redis::Client,
    stream_key: String,
    consumer_group: String,
    consumer_name: String,
    handler: Arc<H>,
}

impl<H: IngestionHandler> RedisConsumer<H> {
    /// Create a new Redis consumer
    pub async fn new(
        url: &str,
        stream_key: String,
        consumer_group: String,
        consumer_name: String,
        handler: H,
    ) -> Result<Self> {
        use crate::domain::CostOpsError;

        info!(
            url = %url,
            stream_key = %stream_key,
            consumer_group = %consumer_group,
            "Connecting to Redis"
        );

        let client = redis::Client::open(url)
            .map_err(|e| CostOpsError::Integration(format!("Redis connection failed: {}", e)))?;

        Ok(Self {
            client,
            stream_key,
            consumer_group,
            consumer_name,
            handler: Arc::new(handler),
        })
    }

    /// Start consuming messages from Redis Streams
    pub async fn start(&mut self) -> Result<()> {
        use crate::domain::CostOpsError;

        info!(
            stream_key = %self.stream_key,
            consumer_group = %self.consumer_group,
            consumer_name = %self.consumer_name,
            "Starting Redis consumer"
        );

        let mut con = self.client.get_async_connection().await
            .map_err(|e| CostOpsError::Integration(format!("Redis connection failed: {}", e)))?;

        // Create consumer group if it doesn't exist
        let _: std::result::Result<(), redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.stream_key)
            .arg(&self.consumer_group)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut con)
            .await;

        loop {
            match self.consume_messages(&mut con).await {
                Ok(_) => {}
                Err(e) => {
                    error!(error = %e, "Failed to consume Redis messages");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Consume messages from Redis
    async fn consume_messages(
        &self,
        con: &mut redis::aio::Connection,
    ) -> Result<()> {
        use crate::domain::CostOpsError;
        use redis::AsyncCommands;

        // Read messages from stream
        let results: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = con
            .xread_options(
                &[&self.stream_key],
                &[">"],
                &redis::streams::StreamReadOptions::default()
                    .group(&self.consumer_group, &self.consumer_name)
                    .count(10)
                    .block(5000),
            )
            .await
            .map_err(|e| CostOpsError::Integration(format!("Redis xread failed: {}", e)))?;

        for (_stream, messages) in results {
            for (message_id, fields) in messages {
                match self.process_redis_message(&message_id, &fields, con).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            message_id = %message_id,
                            error = %e,
                            "Failed to process Redis message"
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a single Redis message
    async fn process_redis_message(
        &self,
        message_id: &str,
        fields: &[(String, String)],
        con: &mut redis::aio::Connection,
    ) -> Result<()> {
        use crate::domain::CostOpsError;
        use redis::AsyncCommands;

        // Extract payload from fields
        let payload_json = fields
            .iter()
            .find(|(k, _)| k == "payload")
            .map(|(_, v)| v)
            .ok_or_else(|| CostOpsError::Validation("Missing payload field".to_string()))?;

        let stream_msg: StreamMessage = serde_json::from_str(payload_json)?;

        info!(
            message_id = %message_id,
            event_type = ?stream_msg.event_type,
            "Processing Redis message"
        );

        match self.handler.handle_single(stream_msg.payload).await {
            Ok(response) => {
                info!(
                    message_id = %message_id,
                    status = ?response.status,
                    "Successfully processed Redis message"
                );

                // Acknowledge message
                let _: i64 = con
                    .xack(&self.stream_key, &self.consumer_group, &[message_id])
                    .await
                    .map_err(|e| CostOpsError::Integration(format!("Redis xack failed: {}", e)))?;

                Ok(())
            }
            Err(e) => {
                error!(
                    message_id = %message_id,
                    error = %e,
                    "Failed to process Redis message"
                );

                // Could implement retry logic here
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require actual NATS/Redis instances
    // For now, we ensure the code compiles
}
