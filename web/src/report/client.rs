use crate::system_info::SystemInfo;
use crate::vo::fusion::Fusion;
use async_trait::async_trait;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Copy, Clone, Debug, PartialEq)]
enum ReportMode {
    Realtime,
    Interval,
}

#[derive(Debug)]
pub enum Call {
    Start,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventModel {
    event: String,
    data: Fusion,
}

pub struct Client {
    sys: Arc<RwLock<SystemInfo>>,
    mode: Arc<RwLock<ReportMode>>,
    cancel_token: CancellationToken,
    handle: ezsockets::Client<Self>,
    interval: Arc<RwLock<Duration>>,
}

impl Client {
    pub fn new(handle: ezsockets::Client<Self>, interval: Option<Duration>) -> Self {
        Self {
            handle,
            sys: Arc::new(RwLock::new(SystemInfo::new())),
            mode: Arc::new(RwLock::new(ReportMode::Interval)),
            cancel_token: CancellationToken::new(),
            interval: Arc::new(RwLock::new(interval.unwrap_or(Duration::from_secs(60)))),
        }
    }
    pub fn start(&mut self) {
        let interval = self.interval.clone();
        let mode = self.mode.clone();
        let cancel_token = self.cancel_token.clone();
        let handle = self.handle.clone();
        let sys = self.sys.clone();
        tokio::spawn(async move {
            loop {
                let sleep_duration = if *mode.read().await == ReportMode::Realtime {
                    Duration::from_secs(1)
                } else {
                    *interval.read().await
                };
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        debug!("task sleep_duration: {sleep_duration:?} cancelled");
                        break;
                    }
                    _ = tokio::time::sleep(sleep_duration) => {
                        let fusion = sys.write().await.get_fusion_with_full_process();
                        debug!("sending message: {fusion:?}");
                        let event = EventModel {
                            event: "report".into(),
                            data: fusion,
                        };
                        let json_str = serde_json::to_string(&event).unwrap();
                        debug!("json_str: {json_str}");
                        handle.text(json_str);
                    }
                }
            }
        });
    }

    async fn set_mode(&mut self, mode: ReportMode) {
        *self.mode.write().await = mode;
    }

    async fn set_interval(&mut self, interval: Duration) {
        *self.interval.write().await = interval;
    }

    fn cancel(&mut self) {
        self.cancel_token.cancel();
        self.cancel_token = CancellationToken::new();
    }
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        debug!("received text message: {text}");
        match text.as_str() {
            "realtime" => self.set_mode(ReportMode::Realtime).await,
            "interval" => self.set_mode(ReportMode::Interval).await,
            _ => {}
        };
        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        debug!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        match call {
            Call::Start => self.start(),
        }
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
        info!("report server connected successfully");
        self.start();
        Ok(())
    }

    async fn on_close(&mut self) -> Result<(), ezsockets::Error> {
        info!("report server closed");
        self.cancel();
        Ok(())
    }
}
