use gasket::error::AsWorkError;
use pallas::network::miniprotocols::handshake;
use pallas::network::multiplexer;
use pallas::network::multiplexer::bearers::Bearer;
use pallas::network::multiplexer::demux::{Demuxer, Egress};
use pallas::network::multiplexer::mux::{Ingress, Muxer};
use pallas::network::multiplexer::sync::SyncPlexer;
use tracing::{debug, error, info, warn};

use super::prelude::*;

struct GasketEgress(DemuxOutputPort);

impl Egress for GasketEgress {
    fn send(
        &mut self,
        payload: multiplexer::Payload,
    ) -> Result<(), multiplexer::demux::EgressError> {
        self.0
            .send(gasket::messaging::Message::from(payload))
            .map_err(|_| multiplexer::demux::EgressError(vec![]))
    }
}

struct GasketIngress(MuxInputPort);

impl Ingress for GasketIngress {
    fn recv_timeout(
        &mut self,
        duration: std::time::Duration,
    ) -> Result<multiplexer::Message, multiplexer::mux::IngressError> {
        self.0
            .recv_timeout(duration)
            .map(|msg| msg.payload)
            .map_err(|err| match err {
                gasket::error::Error::RecvIdle => multiplexer::mux::IngressError::Empty,
                _ => multiplexer::mux::IngressError::Disconnected,
            })
    }
}

type IsBusy = bool;

fn handle_demux_outcome(
    outcome: Result<multiplexer::demux::TickOutcome, multiplexer::demux::DemuxError>,
) -> Result<IsBusy, gasket::error::Error> {
    match outcome {
        Ok(x) => match x {
            multiplexer::demux::TickOutcome::Busy => Ok(true),
            multiplexer::demux::TickOutcome::Idle => Ok(false),
        },
        Err(err) => match err {
            multiplexer::demux::DemuxError::BearerError(err) => {
                error!("{}", err.kind());
                Err(gasket::error::Error::ShouldRestart)
            }
            multiplexer::demux::DemuxError::EgressDisconnected(x, _) => {
                error!(protocol = x, "egress disconnected");
                Err(gasket::error::Error::WorkPanic)
            }
            multiplexer::demux::DemuxError::EgressUnknown(x, _) => {
                error!(protocol = x, "unknown egress");
                Err(gasket::error::Error::WorkPanic)
            }
        },
    }
}

fn handle_mux_outcome(
    outcome: multiplexer::mux::TickOutcome,
) -> Result<IsBusy, gasket::error::Error> {
    match outcome {
        multiplexer::mux::TickOutcome::Busy => Ok(true),
        multiplexer::mux::TickOutcome::Idle => Ok(false),
        multiplexer::mux::TickOutcome::BearerError(err) => {
            warn!(%err);
            Err(gasket::error::Error::ShouldRestart)
        }
        multiplexer::mux::TickOutcome::IngressDisconnected => {
            error!("ingress disconnected");
            Err(gasket::error::Error::WorkPanic)
        }
    }
}

pub struct Worker {
    peer_address: String,
    network_magic: u64,
    input: MuxInputPort,
    channel2_out: Option<DemuxOutputPort>,
    channel3_out: Option<DemuxOutputPort>,
    demuxer: Option<Demuxer<GasketEgress>>,
    muxer: Option<Muxer<GasketIngress>>,
}

impl Worker {
    pub fn new(
        peer_address: String,
        network_magic: u64,
        input: MuxInputPort,
        channel2_out: Option<DemuxOutputPort>,
        channel3_out: Option<DemuxOutputPort>,
    ) -> Self {
        Self {
            peer_address,
            network_magic,
            input,
            channel2_out,
            channel3_out,
            demuxer: None,
            muxer: None,
        }
    }

    fn handshake(&self, bearer: Bearer) -> Result<Bearer, gasket::error::Error> {
        info!("excuting handshake");

        let plexer = SyncPlexer::new(bearer, 0);
        let versions = handshake::n2n::VersionTable::v7_and_above(self.network_magic);
        let mut client = handshake::Client::new(plexer);

        let output = client.handshake(versions).or_panic()?;
        debug!("handshake output: {:?}", output);

        let bearer = client.unwrap().unwrap();

        match output {
            handshake::Confirmation::Accepted(version, _) => {
                info!(version, "connected to upstream peer");
                Ok(bearer)
            }
            _ => {
                error!("couldn't agree on handshake version");
                Err(gasket::error::Error::WorkPanic)
            }
        }
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        // TODO: define networking metrics (bytes in / out, etc)
        gasket::metrics::Builder::new().build()
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        debug!("connecting muxer");

        let bearer = multiplexer::bearers::Bearer::connect_tcp(&self.peer_address).or_restart()?;

        let bearer = self.handshake(bearer)?;

        let mut demuxer = Demuxer::new(bearer.clone());

        if let Some(c2) = &self.channel2_out {
            demuxer.register(2, GasketEgress(c2.clone()));
        }

        if let Some(c3) = &self.channel3_out {
            demuxer.register(3, GasketEgress(c3.clone()));
        }

        self.demuxer = Some(demuxer);

        let muxer = Muxer::new(bearer, GasketIngress(self.input.clone()));
        self.muxer = Some(muxer);

        Ok(())
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let muxer = self.muxer.as_mut().unwrap();
        let demuxer = self.demuxer.as_mut().unwrap();

        let span = tracing::span::Span::current();

        let mut mux_res = None;
        let mut demux_res = None;

        rayon::scope(|s| {
            s.spawn(|_| {
                let _guard = span.enter();
                info!("mux ticking");
                let outcome = muxer.tick();
                mux_res = Some(handle_mux_outcome(outcome));
            });
            s.spawn(|_| {
                let _guard = span.enter();
                info!("demux ticking");
                let outcome = demuxer.tick();
                demux_res = Some(handle_demux_outcome(outcome));
            });
        });

        mux_res.unwrap()?;
        demux_res.unwrap()?;

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}
