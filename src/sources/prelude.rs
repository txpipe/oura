use pallas::network::multiplexer;
use tracing::error;

// ports used by plexer
pub type MuxOutputPort = gasket::messaging::OutputPort<(u16, multiplexer::Payload)>;
pub type DemuxInputPort = gasket::messaging::InputPort<multiplexer::Payload>;

// ports used by mini-protocols
pub type MuxInputPort = gasket::messaging::InputPort<(u16, multiplexer::Payload)>;
pub type DemuxOutputPort = gasket::messaging::OutputPort<multiplexer::Payload>;

pub struct ProtocolChannel(u16, MuxOutputPort, DemuxInputPort);

impl multiplexer::agents::Channel for ProtocolChannel {
    fn enqueue_chunk(
        &mut self,
        payload: multiplexer::Payload,
    ) -> Result<(), multiplexer::agents::ChannelError> {
        match self
            .1
            .send(gasket::messaging::Message::from((self.0, payload)))
        {
            Ok(_) => Ok(()),
            Err(error) => {
                error!(?error, "enqueue chunk failed");
                Err(multiplexer::agents::ChannelError::NotConnected(None))
            }
        }
    }

    fn dequeue_chunk(&mut self) -> Result<multiplexer::Payload, multiplexer::agents::ChannelError> {
        match self.2.recv_or_idle() {
            Ok(msg) => Ok(msg.payload),
            Err(error) => {
                error!(?error, "dequeue chunk failed");
                Err(multiplexer::agents::ChannelError::NotConnected(None))
            }
        }
    }
}

pub fn protocol_channel(
    id: u16,
    plexer_input: &mut MuxInputPort,
    plexer_output: &mut DemuxOutputPort,
) -> ProtocolChannel {
    let mut muxer = MuxOutputPort::default();
    let mut demuxed = DemuxInputPort::default();

    gasket::messaging::connect_ports(&mut muxer, plexer_input, 1000);
    gasket::messaging::connect_ports(plexer_output, &mut demuxed, 1000);

    ProtocolChannel(id, muxer, demuxed)
}
