use std::fs;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use assert_cmd::Command;
use futures_util::SinkExt;
use oura::sources::hydra::{HydraMessage, HydraMessagePayload};
use predicates::prelude::*;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::time;
use tokio_tungstenite::accept_async;
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
fn hydra_oura_stdout_scenario_1() -> TestResult {
    let tx_event = r#"{"event":"apply","point":{"hash":"84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab","slot":7},"record":{"hex":"84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858401b13ee550f3167a1b94796f2a2f5e22d782d628336a7797c5b798f358fa564dbe92ea75a4e2449eb2cef59c097d8497545ef1e4ea441b88a481194323ae7c608f5f6"}}"#;
    let events = vec![
        "PeerConnected".to_string(),
        "PeerConnected".to_string(),
        "Greetings".to_string(),
        "HeadIsInitializing".to_string(),
        "Committed".to_string(),
        "Committed".to_string(),
        "Committed".to_string(),
        "HeadIsOpen".to_string(),
        "TxValid".to_string(),
        tx_event.to_string(),
        "SnapshotConfirmed".to_string(),
        "HeadIsClosed".to_string(),
        "ReadyToFanout".to_string(),
        "HeadIsFinalized".to_string(),
    ];
    hydra_oura_stdout_test("tests/hydra/scenario_1.txt".to_string(), events)
}

#[test]
fn hydra_oura_stdout_scenario_2() -> TestResult {
    let tx_event_1 = r#"{"event":"apply","point":{"hash":"84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab","slot":7},"record":{"hex":"84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858407342c0c4de1b55bc9e56c86829a1fb5906e964f109fd698d37d5933ed230b1a878bfee20980bb90b48aa32c472fdd465c2eb770551b84de7041838415faed502f5f6"}}"#;
    let tx_event_2 = r#"{"event":"apply","point":{"hash":"84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab","slot":9},"record":{"hex":"84a300d901028182582065d64ade1fa9da5099107e3ab9efeea6f305c3c831ca8b9c8f87594289e5161701018282581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a0016e36082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a014810600200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf85840b991c62af8e2b2d06f821fb6064f98c2fc8909b0b2d81435c7e075a61fc92ee6c9224f23d817de35d5529f54034c2ab8dfaded387e99fc525344846bb5dc860af5f6"}}"#;
    let tx_event_3 = r#"{"event":"apply","point":{"hash":"84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab","slot":11},"record":{"hex":"84a300d90102818258207b27f432e04984dc21ee61e8b1539775cd72cc8669f72cf39aebf6d87e35c69700018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a00a7d8c082581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a025317c00200a100d9010281825820aa268d154185c9ea06ea73442fd8143c34c1dd543b7142bcb132aac0d1ed6ece5840fc6e2b0750259deedd5a73eeadf481138bf82edc3425614871a0ef09bfcf8cae52a80240fb895a7e6a8ad94d4acb32dffe567ed0d338afcd7878f745737f420df5f6"}}"#;
    let tx_event_4 = r#"{"event":"apply","point":{"hash":"84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab","slot":13},"record":{"hex":"84a300d9010281825820c9a5fb7ca6f55f07facefccb7c5d824eed00ce18719d28ec4c4a2e4041e85d9700018282581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a00c65d4082581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a052f83c00200a100d9010281825820f953b2d6b6f319faa9f8462257eb52ad73e33199c650f0755e279e21882399c05840ac8f1632d9a636d3627328ffd09cd32e1b654cbf318f0ce499a9870b05530041aa0badf07cd43fec8f1456537ada71227bea8123c1ed641ae3cb22b7313d5f08f5f6"}}"#;
    let events = vec![
        "PeerConnected".to_string(),
        "PeerConnected".to_string(),
        "Greetings".to_string(),
        "HeadIsInitializing".to_string(),
        "Committed".to_string(),
        "Committed".to_string(),
        "Committed".to_string(),
        "HeadIsOpen".to_string(),
        "TxValid".to_string(),
        tx_event_1.to_string(),
        "SnapshotConfirmed".to_string(),
        "TxValid".to_string(),
        tx_event_2.to_string(),
        "SnapshotConfirmed".to_string(),
        "TxValid".to_string(),
        tx_event_3.to_string(),
        "SnapshotConfirmed".to_string(),
        "TxValid".to_string(),
        tx_event_4.to_string(),
        "SnapshotConfirmed".to_string(),
        "HeadIsClosed".to_string(),
        "ReadyToFanout".to_string(),
        "HeadIsFinalized".to_string(),
    ];
    hydra_oura_stdout_test("tests/hydra/scenario_2.txt".to_string(), events)
}

// Run:
// cargo test hydra_oura -- --nocapture
// in order to see println
fn hydra_oura_stdout_test(file: String, expected: Vec<String>) -> TestResult {
    let rt = Runtime::new().unwrap();
    let (tx, _rx) = mpsc::channel();
    let _ = rt.block_on(async move {
        let addr = "127.0.0.1:4001".to_string();
        let server = TcpListener::bind(&addr).await?;
        println!("WebSocket server started on ws://{}", addr);

        let _ = tokio::spawn(async move { oura_pipeline().await });

        while let Ok((stream, _)) = server.accept().await {
            tokio::spawn(handle_connection(stream, file, tx));
            time::sleep(Duration::from_secs(3)).await;
            break;
        }

        let jsons = fs::read_to_string("tests/hydra/logs.txt")?;
        let mut predicates = vec![];
        let mut count = 0;
        for json in jsons.lines() {
            let predicate_fn = predicate::str::contains(&expected[count]);
            predicates.push(predicate_fn.eval(json));
            count += 1;
        }
        assert_eq!(predicates, vec![true; expected.len()]);

        Ok::<(), std::io::Error>(())
    });
    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    file: String,
    tx: mpsc::Sender<usize>,
) -> Result<()> {
    let mut ws_stream = accept_async(stream).await?;
    println!("WebSocket server oura connection established");

    let to_send = fs::read_to_string(file)?;

    let mut lines = 0;
    for line in to_send.lines() {
        ws_stream.send(Message::Text(line.to_string())).await?;
        lines += 1;
    }
    tx.send(lines).unwrap();

    Ok(())
}

async fn oura_pipeline() -> Result<()> {
    //Clean output file
    let _ = std::process::Command::new("truncate")
        .arg("-s 0")
        .arg("tests/hydra/logs.txt")
        .spawn();

    tokio::spawn(invoke_pipeline());
    time::sleep(Duration::from_secs(1)).await;

    Ok(())
}

async fn invoke_pipeline() -> Result<()> {
    let mut cmd = Command::cargo_bin("oura")?;
    cmd.args(vec!["daemon", "--config", "tests/daemon.toml"])
        .assert()
        .success();

    Ok(())
}
