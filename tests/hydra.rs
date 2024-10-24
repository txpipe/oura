use oura::sources::hydra::{HydraMessage, HydraMessagePayload};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn test_events_deserialisation(expected_msgs: Vec<HydraMessage>, input: &str) -> TestResult {
    let mut deserialized: Vec<HydraMessage> = Vec::new();
    for line in input.lines() {
        let msg: HydraMessage = serde_json::from_str(&line)?;
        deserialized.push(msg);
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
       HydraMessage {
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
    }, HydraMessage {
        seq: 0,
        payload: HydraMessagePayload::Other,
        head_id: None,
        raw_json: json!(
        { "peer": "3"
           , "seq": 0
           , "tag": "PeerConnected"
           , "timestamp": "2024-10-08T13:01:20.556003751Z"
        }),
    }];

    let raw_str = r#"{"headId":"84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab","seq":7,"tag":"TxValid","timestamp":"2024-10-08T13:07:18.008847436Z","transaction":{"cborHex":"84a300d9010281825820f0a39560ea80ccc68e8dffb6a4a077c8927811f06c5d9058d0fa2d1a8d047d2000018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a001e848082581d600d45f2b310a98e766cee2ab2f6756c91719bd7b35929cef058365b651a015ef3c00200a100d90102818258200f193a88190f6dace0a3db1e0e50797a6e28cd4b6e289260dc96b5a8d7934bf858401b13ee550f3167a1b94796f2a2f5e22d782d628336a7797c5b798f358fa564dbe92ea75a4e2449eb2cef59c097d8497545ef1e4ea441b88a481194323ae7c608f5f6","description":"Ledger Cddl Format","txId":"633777d68a85fe989f88aa839aa84743f64d68a931192c41f4df8ed0f16e03d1","type":"Witnessed Tx ConwayEra"}}
{"peer":"3","seq":0,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.556003751Z"}
"#;
    test_events_deserialisation(evts, &raw_str)
}

#[test]
fn three_valid_evts() -> TestResult {
    let evts = vec![
        HydraMessage {
            seq: 0,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "peer": "3"
               , "seq": 0
               , "tag": "PeerConnected"
               , "timestamp": "2024-10-08T13:01:20.556003751Z"
            }),
        },
        HydraMessage {
            seq: 1,
            payload: HydraMessagePayload::Other,
            head_id: None,
            raw_json: json!(
            { "peer": "2"
               , "seq": 1
               , "tag": "PeerConnected"
               , "timestamp": "2024-10-08T13:01:20.559653645Z"
            }),
        },
        HydraMessage {
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
        },
    ];

    let raw_str = r#"{"peer":"3","seq":0,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.556003751Z"}
{"peer":"2","seq":1,"tag":"PeerConnected","timestamp":"2024-10-08T13:01:20.559653645Z"}
{"headStatus":"Idle","hydraNodeVersion":"0.19.0-1ffe7c6b505e3f38b5546ae5e5b97de26bc70425","me":{"vkey":"b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb"},"seq":2,"tag":"Greetings","timestamp":"2024-10-08T13:04:56.445761285Z"}
"#;
    test_events_deserialisation(evts, &raw_str)
}
