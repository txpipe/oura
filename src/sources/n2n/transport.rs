use pallas::network::{miniprotocols::handshake, multiplexer};

pub struct Transport {
    pub channel2: multiplexer::StdChannel,
    pub channel3: multiplexer::StdChannel,
    pub version: handshake::VersionNumber,
}

impl Transport {
    fn do_handshake(
        channel: multiplexer::StdChannel,
        magic: u64,
    ) -> Result<handshake::VersionNumber, crate::Error> {
        log::debug!("doing handshake");

        let versions = handshake::n2n::VersionTable::v6_and_above(magic);
        let mut client = handshake::Client::new(channel);

        let output = client
            .handshake(versions)
            .map_err(crate::Error::ouroboros)?;

        log::info!("handshake output: {:?}", output);

        match output {
            handshake::Confirmation::Accepted(version, _) => Ok(version),
            _ => Err(crate::Error::ouroboros(
                "couldn't agree on handshake version",
            )),
        }
    }

    pub fn setup(address: &str, magic: u64) -> Result<Self, crate::Error> {
        log::debug!("connecting muxer");

        let bearer =
            multiplexer::bearers::Bearer::connect_tcp(address).map_err(crate::Error::network)?;
        let mut plexer = multiplexer::StdPlexer::new(bearer);

        let channel0 = plexer.use_channel(0);
        let channel2 = plexer.use_channel(2);
        let channel3 = plexer.use_channel(3);

        plexer.muxer.spawn();
        plexer.demuxer.spawn();

        let version = Self::do_handshake(channel0, magic)?;

        Ok(Self {
            channel2,
            channel3,
            version,
        })
    }
}
