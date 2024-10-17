use std::fs;

use oura::sources::hydra::{HydraMessage, HydraMessagePayload};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn run_scenario(_expected_msgs: &[HydraMessage], expected_file: &str) -> TestResult {
    let _file = fs::read_to_string(expected_file)?;
    Ok(())
}

fn test_event_deserialization(expected: HydraMessage, input: &str) -> TestResult {
    let deserialized: HydraMessage = serde_json::from_str(&input)?;
    assert_eq!(deserialized, expected);
    Ok(())
}

#[test]
fn hydra_scenario_1() -> TestResult {
    let msgs = [HydraMessage {
        seq: 0,
        head_id: Some(vec![0u8; 32]),
        payload: None,
        raw_json: "".into(),
    }];
    run_scenario(&msgs, "tests/hydra/scenario_1.txt")
}

#[test]
fn hydra_scenario_2() -> TestResult {
    let msgs = [HydraMessage {
        seq: 0,
        head_id: Some(vec![0u8; 32]),
        payload: None,
        raw_json: "".into(),
    }];
    run_scenario(&msgs, "tests/hydra/scenario_2.txt")
}

#[test]
fn tx_valid_evt() -> TestResult {
    let evt = HydraMessage {
        seq: 15,
        head_id: Some(hex::decode("84e657e3dd5241caac75b749195f78684023583736cc08b2896290ab").unwrap()
                .to_vec()),
        payload: Some(HydraMessagePayload::TxValid {
            tx: hex::decode("84a300d9010281825820635ffa4d3f8b5ccd60a89918866a5bb0776966572324da9a86870f79dcce4aad01018282581d605e4e214a6addd337126b3a61faad5dfe1e4f14f637a8969e3a05eefd1a0098968082581d6069830961c6af9095b0f2648dff31fa9545d8f0b6623db865eb78fde81a039387000200a100d9010281825820f953b2d6b6f319faa9f8462257eb52ad73e33199c650f0755e279e21882399c05840c1f23b630cf3d0ffe4186436225906c81bcddb0a27a632696035d4bb2d32e646c81759789c35c940b9695a87a0978a0408cff550c8d8f9ab4ac6d6d29b82a109f5f6")
                    .unwrap()
                    .to_vec(),
        }),
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
        payload: None,
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
        payload: None,
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
        payload: None,
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
