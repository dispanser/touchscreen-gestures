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
}

pub struct ZbusOMeter<'a> {
    proxy: IioSensorProxy<'a>,
    last: Orientation,
}

impl<'a> ZbusOMeter<'a> {
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
                last: last.try_into()?,
            })
        }
    }
}

impl TryFrom<String> for Orientation {
    type Error = GesturesError;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "normal" => Ok(Orientation::Normal),
            "left-up" => Ok(Orientation::LeftUp),
            "bottom-up" => Ok(Orientation::BottomUp),
            "right-up" => Ok(Orientation::RightUp),
            invalid => Err(GesturesError::AccelerometerFailed {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid orientation: {}", invalid),
                )),
            }),
        }
    }
}

impl<'a> OrientationSensor for ZbusOMeter<'a> {
    fn orientation<'life>(
        &'life mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Orientation>> + Send + 'life>> {
        Box::pin(async move {
            let reading: String = self
                .proxy
                .0
                .get_property("AccelerometerOrientation")
                .await?;
            let orientation: Orientation = reading.try_into()?;
            if self.last != orientation {
                trace!("changed orientation: {:?} -> {orientation:?}", self.last);
                self.last = orientation;
            }
            Ok(orientation)
        })
    }
}

impl<'a> Drop for ZbusOMeter<'a> {
    fn drop(&mut self) {
        let rt = tokio::runtime::Handle::current();
        let _ = rt.block_on(self.proxy.ReleaseAccelerometer());
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
