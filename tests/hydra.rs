#![cfg(feature = "hydra")]

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use serde::Deserialize;
use oura::framework::IntersectConfig;
use anyhow::Result;
use futures_util::SinkExt;
use oura::sources::hydra::{HydraMessage, HydraMessagePayload};
use oura::sinks::Config::FileRotate;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use oura::sources::Config::Hydra;
use oura::daemon::{run_daemon, ConfigRoot};
use gasket::daemon::Daemon;
use goldenfile::Mint;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;
use tokio::time;
use tokio_tungstenite::accept_async;
use port_selector::random_free_port;
use tokio_tungstenite::tungstenite::protocol::Message;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq)]
enum LineParseResult<T> {
    LineParsed(T),
    LineNotParsed,
}

fn test_events_deserialisation(
    expected_msgs: Vec<LineParseResult<HydraMessage>>,
    input: &str,
) -> TestResult {
    let mut deserialized: Vec<LineParseResult<HydraMessage>> = Vec::new();
    for line in input.lines() {
        match serde_json::from_str(&line) {
            Ok(msg) => {
                deserialized.push(LineParseResult::LineParsed(msg));
            }
            _ => {
                deserialized.push(LineParseResult::LineNotParsed);
            }
        }
    }
    assert_eq!(deserialized, expected_msgs);
    Ok(())
}

fn test_scenario(
    expected_msgs: Vec<LineParseResult<HydraMessagePayload>>,
    file: &str,
) -> TestResult {
    let mut deserialized: Vec<LineParseResult<HydraMessagePayload>> = Vec::new();
    let input = fs::read_to_string(file)?;
    for line in input.lines() {
        match serde_json::from_str::<HydraMessage>(&line) {
            Ok(msg) => {
                deserialized.push(LineParseResult::LineParsed(msg.payload));
            }
            _ => {
                deserialized.push(LineParseResult::LineNotParsed);
            }
        }
    }
    assert_eq!(deserialized, expected_msgs);
    Ok(())
}

fn test_event_deserialization(expected: HydraMessage, input: &str) -> TestResult {
    let deserialized: HydraMessage = serde_json::from_str(&input)?;
    assert_eq!(deserialized, expected);
    Ok(())
}

#[test]
fn tx_valid_evt() -> TestResult {
    let evt = HydraMessage {
        seq: 15,
        head_id: Some(hex::decode("84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab").unwrap()
                .to_vec()),
        payload: HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d9010281825820635ffa4d3f8b5ccd60a89918866a5bb0776966572324da9a86870f79dcce4aad01018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a0098968082581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a039387000200a100d9010281825820f953b2d6b6f319faa9f8462257eb52ad73e33199c650f0755e279e21882399c05840c1f23b630cf3d0ffe4186436225906c81bcddb0a27a632696035d4bb2d32e646c81759789c35c940b9695a87a0978a0408cff550c8d8f9ab4ac6d6d29b82a109f5f6")
                    .unwrap()
                    .to_vec(),
        },
        raw_json: json!(
            { "headId": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab"
               , "seq": 15
               , "tag": "TxValid"
               , "timestamp": "2024-10-03T11:38:45.449663464Z"
               , "transaction":
               { "cborHex": "84a300d9010281825820635ffa4d3f8b5ccd60a89918866a5bb0776966572324da9a86870f79dcce4aad01018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a0098968082581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a039387000200a100d9010281825820f953b2d6b6f319faa9f8462257eb52ad73e33199c650f0755e279e21882399c05840c1f23b630cf3d0ffe4186436225906c81bcddb0a27a632696035d4bb2d32e646c81759789c35c940b9695a87a0978a0408cff550c8d8f9ab4ac6d6d29b82a109f5f6"
                  , "description": "Ledger Cddl Format"
                  , "txId": "08bb77374329ca28cd3023cace2948d0fc23e2812e8998c966db8b457e6390fe"
                  , "type": "Witnessed Tx ConwayEra"
               }
            }),
    };

    let raw_str = r#"
 {
  "headId": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab",
  "seq": 15,
  "timestamp": "2024-10-03T11:38:45.449663464Z",
  "tag":"TxValid",
  "transaction": {
         "cborHex": "84a300d9010281825820635ffa4d3f8b5ccd60a89918866a5bb0776966572324da9a86870f79dcce4aad01018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a0098968082581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a039387000200a100d9010281825820f953b2d6b6f319faa9f8462257eb52ad73e33199c650f0755e279e21882399c05840c1f23b630cf3d0ffe4186436225906c81bcddb0a27a632696035d4bb2d32e646c81759789c35c940b9695a87a0978a0408cff550c8d8f9ab4ac6d6d29b82a109f5f6",
         "description": "Ledger Cddl Format",
         "txId": "08bb77374329ca28cd3023cace2948d0fc23e2812e8998c966db8b457e6390fe",
         "type": "Witnessed Tx ConwayEra"
     }
 }
"#;
    test_event_deserialization(evt, &raw_str)
}

#[test]
fn peer_connected_evt() -> TestResult {
    let evt = HydraMessage {
        seq: 0,
        payload: HydraMessagePayload::Other,
        head_id: None,
        raw_json: json!(
        { "peer": "3"
           , "seq": 0
           , "tag": "PeerConnected"
           , "timestamp": "2024-10-08T13:01:20.556003751Z"
        }),
    };

    let raw_str = r#"
 {
   "peer": "3",
   "seq": 0,
   "tag": "PeerConnected",
   "timestamp": "2024-10-08T13:01:20.556003751Z"
 }
"#;
    test_event_deserialization(evt, &raw_str)
}

#[test]
fn idle_evt() -> TestResult {
    let evt = HydraMessage {
        seq: 2,
        payload: HydraMessagePayload::Other,
        head_id: None,
        raw_json: json!(
        { "headStatus": "Idle"
           , "hydraNodeVersion": "0.19.0-1ffe7c6b505e3f38b5546ae5e5b97de26bc70425"
           , "me":
           { "vkey": "b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"
           }
           , "seq": 2
           , "tag": "Greetings"
           , "timestamp": "2024-10-08T13:04:56.445761285Z"
        }),
    };

    let raw_str = r#"
 {
   "headStatus": "Idle",
   "hydraNodeVersion": "0.19.0-1ffe7c6b505e3f38b5546ae5e5b97de26bc70425",
   "me": {
     "vkey": "b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"
   },
   "seq": 2,
   "tag": "Greetings",
   "timestamp": "2024-10-08T13:04:56.445761285Z"
 }
"#;
    test_event_deserialization(evt, &raw_str)
}

#[test]
fn committed_evt() -> TestResult {
    let evt = HydraMessage {
        seq: 3,
        payload: HydraMessagePayload::Other,
        head_id: Some(
            hex::decode("84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab")
                .unwrap()
                .to_vec(),
        ),
        raw_json: json!(
        { "headId": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab"
           , "party": {"vkey": "b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"}
           , "seq": 3
           , "tag": "Committed"
           , "timestamp": "2024-10-08T13:05:56.918549005Z"
           , "utxo": {"c9a5fb7ca6f55f07facefccb7c5d824eed00ce18719d28ec4c4a2e4041e85d97#0":
                      {"address": "addr_test1vp5cxztpc6hep9ds7fjgmle3l225tk8ske3rmwr9adu0m6qchmx5z"
                       , "datum": null
                       , "datumhash": null
                       , "inlineDatum": null
                       , "referenceScript": null
                       , "value": {"lovelace": 100000000}
                      }
           }
        }),
    };

    let raw_str = r#"
 {
   "headId": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab",
   "party": {
     "vkey": "b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"
   },
   "seq": 3,
   "tag": "Committed",
   "timestamp": "2024-10-08T13:05:56.918549005Z",
   "utxo": {
     "c9a5fb7ca6f55f07facefccb7c5d824eed00ce18719d28ec4c4a2e4041e85d97#0": {
       "address": "addr_test1vp5cxztpc6hep9ds7fjgmle3l225tk8ske3rmwr9adu0m6qchmx5z",
       "datum": null,
       "datumhash": null,
       "inlineDatum": null,
       "referenceScript": null,
       "value": {
         "lovelace": 100000000
       }
     }
   }
 }
"#;
    test_event_deserialization(evt, &raw_str)
}

#[test]
fn two_valid_evts() -> TestResult {
    let evts = vec![
       LineParseResult::LineParsed(HydraMessage {
        seq: 7,
        head_id: Some(hex::decode("84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab").unwrap()
                .to_vec()),
        payload: HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858401b13ee550f3167a1b94796f2a2f5e22d782d628336a7797c5b798f358fa564dbe92ea75a4e2449eb2cef59c097d8497545ef1e4ea441b88a481194323ae7c608f5f6")
                    .unwrap()
                    .to_vec(),
        },
        raw_json: json!(
            { "headId": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab"
               , "seq": 7
               , "tag": "TxValid"
               , "timestamp": "2024-10-08T13:07:18.008847436Z"
               , "transaction":
               { "cborHex": "84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858401b13ee550f3167a1b94796f2a2f5e22d782d628336a7797c5b798f358fa564dbe92ea75a4e2449eb2cef59c097d8497545ef1e4ea441b88a481194323ae7c608f5f6"
                  , "description": "Ledger Cddl Format"
                  , "txId": "633777d68a85fe989f88aa839aa84743f64d68a931192c41f4df8ed0f16e03d1"
                  , "type": "Witnessed Tx ConwayEra"
               }
            }),
    }), LineParseResult::LineParsed(HydraMessage {
        seq: 0,
        payload: HydraMessagePayload::Other,
        head_id: None,
        raw_json: json!(
        { "peer": "3"
           , "seq": 0
           , "tag": "PeerConnected"
           , "timestamp": "2024-10-08T13:01:20.556003751Z"
        }),
    })];

    let raw_str = r#"{"headId":"84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab","seq":7,"tag":"TxValid","timestamp":"2024-10-08T13:07:18.008847436Z","transaction":{"cborHex":"84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858401b13ee550f3167a1b94796f2a2f5e22d782d628336a7797c5b798f358fa564dbe92ea75a4e2449eb2cef59c097d8497545ef1e4ea441b88a481194323ae7c608f5f6","description":"Ledger Cddl Format","txId":"633777d68a85fe989f88aa839aa84743f64d68a931192c41f4df8ed0f16e03d1","type":"Witnessed Tx ConwayEra"}}
{"peer":"3","seq":0,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.556003751Z"}
"#;
    test_events_deserialisation(evts, &raw_str)
}

#[test]
fn three_valid_evts() -> TestResult {
    let evts = vec![
        LineParseResult::LineParsed(HydraMessage {
            seq: 0,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "peer": "3"
               , "seq": 0
               , "tag": "PeerConnected"
               , "timestamp": "2024-10-08T13:01:20.556003751Z"
            }),
        }),
        LineParseResult::LineParsed(HydraMessage {
            seq: 1,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "peer": "2"
               , "seq": 1
               , "tag": "PeerConnected"
               , "timestamp": "2024-10-08T13:01:20.559653645Z"
            }),
        }),
        LineParseResult::LineParsed(HydraMessage {
            seq: 2,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "headStatus": "Idle"
               , "hydraNodeVersion": "0.19.0-1ffe7c6b505e3f38b5546ae5e5b97de26bc70425"
               , "me":
               { "vkey": "b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"
               }
               , "seq": 2
               , "tag": "Greetings"
               , "timestamp": "2024-10-08T13:04:56.445761285Z"
            }),
        }),
    ];

    let raw_str = r#"{"peer":"3","seq":0,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.556003751Z"}
{"peer":"2","seq":1,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.559653645Z"}
{"headStatus":"Idle","hydraNodeVersion":"0.19.0-1ffe7c6b505e3f38b5546ae5e5b97de26bc70425","me":{"vkey":"b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"},"seq":2,"tag":"Greetings","timestamp":"2024-10-08T13:04:56.445761285Z"}
"#;
    test_events_deserialisation(evts, &raw_str)
}

#[test]
fn three_valid_two_invalid_evts() -> TestResult {
    let evts = vec![
        LineParseResult::LineParsed(HydraMessage {
            seq: 0,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "peer": "3"
               , "seq": 0
               , "tag": "PeerConnected"
               , "timestamp": "2024-10-08T13:01:20.556003751Z"
            }),
        }),
        LineParseResult::LineParsed(HydraMessage {
            seq: 1,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "peer": "2"
               , "seq": 1
               , "tag": "PeerConnected"
               , "timestamp": "2024-10-08T13:01:20.559653645Z"
            }),
        }),
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessage {
            seq: 2,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "headStatus": "Idle"
               , "hydraNodeVersion": "0.19.0-1ffe7c6b505e3f38b5546ae5e5b97de26bc70425"
               , "me":
               { "vkey": "b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"
               }
               , "seq": 2
               , "tag": "Greetings"
               , "timestamp": "2024-10-08T13:04:56.445761285Z"
            }),
        }),
    ];

    let raw_str = r#"{"peer":"3","seq":0,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.556003751Z"}
{"peer":"2","seq":1,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.559653645Z"}
1
2
{"headStatus":"Idle","hydraNodeVersion":"0.19.0-1ffe7c6b505e3f38b5546ae5e5b97de26bc70425","me":{"vkey":"b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"},"seq":2,"tag":"Greetings","timestamp":"2024-10-08T13:04:56.445761285Z"}
"#;
    test_events_deserialisation(evts, &raw_str)
}

#[test]
fn scenario_1() -> TestResult {
    let payloads = vec![
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858401b13ee550f3167a1b94796f2a2f5e22d782d628336a7797c5b798f358fa564dbe92ea75a4e2449eb2cef59c097d8497545ef1e4ea441b88a481194323ae7c608f5f6")
                .unwrap()
                .to_vec(),
        }),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed
    ];
    test_scenario(payloads, "tests/hydra/scenario_1.txt")
}

#[test]
fn scenario_2() -> TestResult {
    let payloads = vec![
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858407342c0c4de1b55bc9e56c86829a1fb5906e964f109fd698d37d5933ed230b1a878bfee20980bb90b48aa32c472fdd465c2eb770551b84de7041838415faed502f5f6")
                .unwrap()
                .to_vec(),
        }),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d901028182582065d64ade1fa9da5099107e3ab9efeea6f305c3c831ca8b9c8f87594289e5161701018282581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a0016e36082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a014810600200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf85840b991c62af8e2b2d06f821fb6064f98c2fc8909b0b2d81435c7e075a61fc92ee6c9224f23d817de35d5529f54034c2ab8dfaded387e99fc525344846bb5dc860af5f6")
                .unwrap()
                .to_vec(),
        }),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d90102818258207b27f432e04984dc21ee61e8b1539775cd72cc8669f72cf39aebf6d87e35c69700018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a00a7d8c082581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a025317c00200a100d9010281825820aa268d154185c9ea06ea73442fd8143c34c1dd543b7142bcb132aac0d1ed6ece5840fc6e2b0750259deedd5a73eeadf481138bf82edc3425614871a0ef09bfcf8cae52a80240fb895a7e6a8ad94d4acb32dffe567ed0d338afcd7878f745737f420df5f6")
                .unwrap()
                .to_vec(),
        }),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d9010281825820c9a5fb7ca6f55f07facefccb7c5d824eed00ce18719d28ec4c4a2e4041e85d9700018282581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a00c65d4082581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a052f83c00200a100d9010281825820f953b2d6b6f319faa9f8462257eb52ad73e33199c650f0755e279e21882399c05840ac8f1632d9a636d3627328ffd09cd32e1b654cbf318f0ce499a9870b05530041aa0badf07cd43fec8f1456537ada71227bea8123c1ed641ae3cb22b7313d5f08f5f6")
                .unwrap()
                .to_vec(),
        }),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineParsed(HydraMessagePayload::Other),
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed,
        LineParseResult::LineNotParsed
    ];
    test_scenario(payloads, "tests/hydra/scenario_2.txt")
}

#[test]
fn hydra_oura_stdout_scenario_1() {
    hydra_oura_stdout_test("scenario_1.txt".to_string(), "golden_1".to_string())
}

#[test]
fn hydra_oura_stdout_scenario_2() {
    hydra_oura_stdout_test("scenario_2.txt".to_string(), "golden_2".to_string())
}

#[test]
fn hydra_restore_from_intersection_success() {
    let scenario= fs::read_to_string("tests/hydra/scenario_1.txt").unwrap();
    let intersect = IntersectConfig::Point(
        6,
        "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab".to_string()
    );
    let events = oura_events_from_mock_chain(scenario, intersect);

    assert_ne!(events.len(), 0);
    assert_eq!(events[0].point, json!({"slot": 7, "hash": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab"}));
    for e in events {
        if e.point == json!({"slot": 6, "hash": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab"}) {
            panic!("only events /after/ the intersection should be emitted");
        }
    }
}

#[test]
fn hydra_restore_from_intersection_tip() {
    let scenario = fs::read_to_string("tests/hydra/scenario_1.txt").unwrap();
    let intersect = IntersectConfig::Point(
        11,
        "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab".to_string()
    );
    let events = oura_events_from_mock_chain(scenario, intersect);
    assert_eq!(events, vec![]);
}

#[test]
fn hydra_restore_from_intersection_point_with_dummy_hash_and_shared_slot_1() {
    let scenario = fs::read_to_string("tests/hydra/scenario_1.txt").unwrap();
    let intersect = IntersectConfig::Point(
        2,
        "00000000000000000000000000000000000000000000000000000000".to_string()
    );
    let events = oura_events_from_mock_chain(scenario, intersect);
    // It appears the Greetings and HeadIsInitializing messages share the same seq / slot.
    assert_eq!(events[0].point, json!({"slot": 2, "hash": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab"}))
}

#[test]
fn hydra_restore_from_intersection_point_with_shared_slot_2() {
    let scenario = fs::read_to_string("tests/hydra/scenario_1.txt").unwrap();
    let intersect = IntersectConfig::Point(
        2,
        "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab".to_string()
    );
    let events = oura_events_from_mock_chain(scenario, intersect);
    assert_eq!(events[0].point, json!({"slot": 3, "hash": "84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab"}))
}

#[test]
fn hydra_restore_from_intersection_failure() {
    let scenario = fs::read_to_string("tests/hydra/scenario_1.txt").unwrap();
    let bad_intersect= IntersectConfig::Point(
        6,
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string()
    );
    let events = oura_events_from_mock_chain(scenario, bad_intersect);
    assert_eq!(events, vec![]);
}

/// Wraps the json format of oura::framework::ChainEvent::Apply with just enough
/// structure to test point equality without having to implement the full json
/// deserializers.
#[derive(Debug, Deserialize, PartialEq)]
struct JsonApplyChainEvent {
    point: Value,
    record: Value,
}

fn oura_output_from_mock_chain(scenario: String, intersect: IntersectConfig) -> String {
    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        let port: u16 = random_free_port().unwrap();
        let addr: String = format!("127.0.0.1:{}", port);
        let url: String = format!("ws://{}", addr.to_string());
        let server = TcpListener::bind(&addr).await.unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let mut config = test_config(&output_file, &url);
        config.intersect = intersect;

        println!("WebSocket server starting on {}", url);

        let _ = tokio::spawn(async move { run_oura(config) });
        let _ = mock_hydra_node(server, scenario).await;

        // After the connection is established, give oura time to process
        // the chain data before we read it.
        time::sleep(Duration::from_secs(3)).await;
        fs::read_to_string(&output_file).unwrap()
    })
}

fn oura_events_from_mock_chain(scenario: String, intersect: IntersectConfig) -> Vec<JsonApplyChainEvent> {
    oura_output_from_mock_chain(scenario, intersect)
        .lines()
        .map(|line| serde_json::from_str(line).expect("Invalid JSON line"))
        .collect()
}

// Run:
// cargo test hydra_oura -- --nocapture
// in order to see println
fn hydra_oura_stdout_test(scenario_name: String, golden_name: String) {
    let mut mint = Mint::new("tests/hydra");
    let mut golden = mint.new_goldenfile(golden_name.clone()).unwrap();

    let scenario = fs::read_to_string(format!("tests/hydra/{}", scenario_name)).unwrap();
    let output = oura_output_from_mock_chain(scenario, IntersectConfig::Origin);

    golden.write_all(output.as_bytes()).unwrap();
}

/// Will await the first connection, and then return while handling it in the
/// background.
async fn mock_hydra_node(server: TcpListener, mock_data: String) {
    async fn handle_connection(
        stream: tokio::net::TcpStream,
        mock_data: String,
        tx: mpsc::Sender<usize>,
    ) -> Result<()> {
        let mut ws_stream = accept_async(stream).await?;
        println!("WebSocket server oura connection established");

        let mut lines = 0;
        for line in mock_data.lines() {
            ws_stream.send(Message::Text(line.to_string())).await?;
            lines += 1;
        }
        tx.send(lines).unwrap();
        Ok(())
    }

    let (tx, _rx) = mpsc::channel();
    let (stream, _) = server.accept().await.unwrap();
    let _ = tokio::spawn(handle_connection(stream, mock_data , tx));
}


fn test_config(tmp_output_file: &NamedTempFile, ws_url: &String) -> ConfigRoot {
    let mut config = ConfigRoot::new(&Some(PathBuf::from("tests/daemon.toml"))).unwrap();

    if let FileRotate(ref mut file_rotate) = config.sink {
        file_rotate.output_path = Some(tmp_output_file.path().to_string_lossy().to_string());
    } else {
        panic!("assumed config template to use file_rotate sink");
    }

    if let Hydra(ref mut hydra_config) = config.source {
        hydra_config.ws_url = ws_url.to_string();
    } else {
        panic!("assumed config template to use hydra source");
    }

    config
}

fn run_oura(config: ConfigRoot) -> Result<Daemon> {
    run_daemon(config).map_err(|e| anyhow::anyhow!(e))
}
