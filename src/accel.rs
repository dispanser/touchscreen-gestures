use std::{future::Future, pin::Pin};

use log::trace;
use zbus::{proxy, Connection};

use crate::error::{GesturesError, Result};

pub trait OrientationSensor: Send + Sync {
    fn orientation<'life>(
        &'life mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Orientation>> + Send + 'life>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Normal,
    LeftUp,
    BottomUp,
    RightUp,
    /// fall back when reding intermittently fails.
    Unknown,
}

pub struct ZbusOMeter<'a> {
    proxy: IioSensorProxy<'a>,
    last: Orientation,
}

impl ZbusOMeter<'_> {
    pub async fn try_new() -> Result<Self> {
        let connection = Connection::system().await?;
        let proxy = IioSensorProxy::new(&connection).await?;
        proxy.ClaimAccelerometer().await?;

        let has_accel: bool = proxy.0.get_property("HasAccelerometer").await?;
        if !has_accel {
            Err(GesturesError::AccelerometerMissing)
        } else {
            let last: String = proxy.0.get_property("AccelerometerOrientation").await?;
            Ok(Self {
                proxy,
                last: last.into(),
            })
        }
    }
}

impl From<String> for Orientation {
    fn from(value: String) -> Self {
        match value.as_str() {
            "normal" => Orientation::Normal,
            "left-up" => Orientation::LeftUp,
            "bottom-up" => Orientation::BottomUp,
            "right-up" => Orientation::RightUp,
            _ => Orientation::Unknown,
        }
    }
}

impl OrientationSensor for ZbusOMeter<'_> {
    fn orientation<'life>(
        &'life mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Orientation>> + Send + 'life>> {
        Box::pin(async move {
            let reading: String = self
                .proxy
                .0
                .get_property("AccelerometerOrientation")
                .await?;
            let orientation: Orientation = reading.into();
            if self.last != orientation {
                trace!("changed orientation: {:?} -> {orientation:?}", self.last);
                self.last = orientation;
            }
            Ok(orientation)
        })
    }
}

#[proxy(
    interface = "net.hadess.SensorProxy",
    default_service = "net.hadess.SensorProxy",
    default_path = "/net/hadess/SensorProxy"
)]
trait IioSensor {
    fn ClaimAccelerometer(&self) -> zbus::Result<()>;
    fn ReleaseAccelerometer(&self) -> zbus::Result<()>;
}
