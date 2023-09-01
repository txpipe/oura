import { useState } from "react";
import json2toml from "json2toml";

export function Configuration() {
  const [openedModal, setOpenedModal] = useState(false);
  const [typeModal, setTypeModal] = useState();
  const [optionModal, setoOptionModal] = useState(undefined);

  const [currentStages, setCurrentStages] = useState({});

  const addSourceStage = ({ source, intersect }) => {
    currentStages.source = source;
    currentStages.intersect = intersect;
    setCurrentStages(currentStages);
    setOpenedModal(false);
  };

  const addFilterStage = (stage) => {
    const hasFilterIndex = currentStages.filters?.findIndex(
      (s) => s.type == stage.type
    );
    console.log(hasFilterIndex)

    if (hasFilterIndex != undefined && hasFilterIndex != -1) {
      currentStages.filters[hasFilterIndex] = stage;
    } else {
      currentStages.filters = currentStages.filters?.concat(stage) || [stage];
    }

    setCurrentStages(currentStages);
    setOpenedModal(false);
  };

  const addSinkStage = (stage) => {
    currentStages.sink = stage;
    setCurrentStages(currentStages);
    setOpenedModal(false);
  };

  function openModal(type) {
    setOpenedModal(true);
    setTypeModal(type);
    setoOptionModal(undefined);
  }

  function exportConfig() {
    if (!currentStages.source) {
      alert("Add a source");
      return;
    }

    if (!currentStages.sink) {
      alert("Add a sink");
      return;
    }

    var element = document.createElement("a");
    element.setAttribute(
      "href",
      "data:text/plain;charset=utf-8," +
        encodeURIComponent(
          json2toml(currentStages, { newlineAfterSection: true })
        )
    );
    element.setAttribute("download", "daemon.toml");

    element.style.display = "none";
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);
  }

  const TYPES = {
    SOURCES: "sources",
    FILTERS: "filters",
    SINKS: "sinks",
  };

  const stages = {
    [TYPES.SOURCES]: {
      N2N: <N2NStage onAdd={addSourceStage} />,
      N2C: <N2CStage onAdd={addSourceStage} />,
      UtxoRPC: <UtxoRPCStage onAdd={addSourceStage} />,
    },
    [TYPES.FILTERS]: {
      ParseCbor: <SimpleStage type="ParseCbor" onAdd={addFilterStage} />,
      SplitBlock: <SimpleStage type="SplitBlock" onAdd={addFilterStage} />,
      Deno: <DenoStage onAdd={addFilterStage} />,
      LegacyV1: <LegacyV1Stage onAdd={addFilterStage} />,
    },
    [TYPES.SINKS]: {
      Stdout: <SimpleStage type="Stdout" onAdd={addSinkStage} />,
      FileRotate: <FileRotateStage onAdd={addSinkStage} />,
      Redis: <RedisStage onAdd={addSinkStage} />,
      AwsLambda: <AwsLambdaStage onAdd={addSinkStage} />,
      AwsS3: <AwsS3Stage onAdd={addSinkStage} />,
      AwsSqs: <AwsSqsStage onAdd={addSinkStage} />,
      GcpPubSub: <GcpPubSubStage onAdd={addSinkStage} />,
      GcpCloudFunction: <GcpCloudFunctionStage onAdd={addSinkStage} />,
      Rabbitmq: <RabbitmqStage onAdd={addSinkStage} />,
      ElasticSearch: <ElasticSearchStage onAdd={addSinkStage} />,
      WebHook: <WebHookStage onAdd={addSinkStage} />,
      Kafka: <KafkaStage onAdd={addSinkStage} />,
    },
  };

  return (
    <div>
      <div className="absolute">
        {openedModal ? (
          <>
            <div className="flex items-center overflow-y-auto fixed inset-0 z-50">
              <div className="my-6 mx-auto max-w-3xl w-full">
                <div className="rounded-lg shadow-lg bg-white dark:bg-gray-700">
                  <div className="flex justify-between p-5">
                    <h3 className="text-3xl font-semibold capitalize">
                      {typeModal}
                    </h3>
                    <button
                      className="p-1 float-right text-3xl leading-none"
                      onClick={() => setOpenedModal(false)}
                    >
                      <span className="text-gray dark:text-gray-200">×</span>
                    </button>
                  </div>

                  <div className="relative p-6 flex-auto">
                    <select
                      name="stage"
                      id="stage"
                      className="w-full py-2 px-4 rounded mb-2"
                      onChange={(e) => setoOptionModal(e.target.value)}
                      value={optionModal}
                    >
                      <option>-</option>
                      {Object.keys(stages[typeModal]).map((k) => (
                        <option value={k} key={k}>
                          {k}
                        </option>
                      ))}
                    </select>
                    {optionModal ? stages[typeModal][optionModal] : null}
                  </div>
                </div>
              </div>
            </div>
            <div className="opacity-25 fixed inset-0 z-40 bg-black"></div>
          </>
        ) : null}
      </div>

      <div className="py-10">
        <div className="py-5">
          <h1 className="font-bold text-2xl dark:text-gray-200 pb-1   ">
            Configure your Oura
          </h1>
          <p> Add stages to your pipeline </p>
        </div>

        <div className="py-5 flex justify-end">
          <button
            className="px-4 py-2 rounded font-bold me-2 text-red-500"
            onClick={() => setCurrentStages({})}
          >
            reset
          </button>

          <button
            className="border border-gray-500 text-gray-500 dark:text-gray-200 hover:bg-gray-500 hover:text-white hover:dark:text-gray-200 px-4 py-2 rounded font-bold "
            onClick={exportConfig}
          >
            export config
          </button>
        </div>

        <div className="grid sm:grid-cols-3 grid-cols-1 gap-3">
          <div>
            <button
              className="w-full bg-gray-500 hover:bg-gray-700 text-white dark:text-gray-200 font-bold py-2 px-4 rounded pointer before:pointer__right dark:before:pointer__right--dark"
              onClick={() => openModal(TYPES.SOURCES)}
            >
              add source
            </button>

            {currentStages.source ? (
              <StageCard
                value={{
                  ...currentStages.source,
                  intersect: currentStages.intersect,
                }}
              />
            ) : null}
          </div>
          <div>
            <button
              className="w-full bg-gray-500 hover:bg-gray-700 text-white dark:text-gray-200 font-bold py-2 px-4 rounded pointer after:pointer__left dark:after:pointer__left--dark before:pointer__right dark:before:pointer__right--dark"
              onClick={() => openModal(TYPES.FILTERS)}
            >
              add filter
            </button>
            {currentStages[TYPES.FILTERS]?.map((value, index) => (
              <div key={index}>
                <StageCard value={value} />
              </div>
            ))}
          </div>

          <div>
            <button
              className="w-full bg-gray-500 hover:bg-gray-700 text-white dark:text-gray-200 font-bold py-2 px-4 rounded pointer after:pointer__left dark:after:pointer__left--dark"
              onClick={() => openModal(TYPES.SINKS)}
            >
              add sink
            </button>

            {currentStages.sink ? (
              <StageCard value={currentStages.sink} />
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}

function N2NStage({ onAdd }) {
  const [peers, setPeers] = useState();
  const [intersect, setIntersect] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="peers"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Peers
          </label>
          <input
            name="peers"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="relays-new.cardano-mainnet.iohk.io:3001,relays-new.cardano-mainnet.iohk.io:3001"
            onChange={(e) => setPeers(e.target.value)}
          />
        </div>

        <Intersect onChange={setIntersect} />
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!peers || !intersect) {
              return;
            }

            const source = {
              type: "N2N",
              peers: peers.split(","),
            };

            onAdd({ source, intersect });
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function N2CStage({ onAdd }) {
  const [socketPath, setSocketPath] = useState();
  const [intersect, setIntersect] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="socketPath"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Socket Path (unix)
          </label>
          <input
            id="socketPath"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setSocketPath(e.target.value)}
          />
        </div>

        <Intersect onChange={setIntersect} />
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!socketPath || !intersect) {
              return;
            }

            const source = {
              type: "N2C",
              socket_path: socketPath,
            };

            onAdd({ source, intersect });
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function UtxoRPCStage({ onAdd }) {
  const [url, setUrl] = useState();
  const [intersect, setIntersect] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="url"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            URL (Dolos Node)
          </label>
          <input
            id="url"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>

        <Intersect onChange={setIntersect} />
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!url || !intersect) {
              return;
            }

            const source = {
              type: "UtxoRPC",
              url,
            };

            onAdd({ source, intersect });
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function Intersect({ onChange }) {
  const [type, setType] = useState();

  const INTERSECTS = {
    TIP: "Tip",
    ORIGIN: "Origin",
    POINT: "Point",
    BREADCRUMBS: "Breadcrumbs",
  };

  function setIntersect(type, value) {
    let intersect = {
      type,
    };

    switch (type) {
      case INTERSECTS.TIP:
      case INTERSECTS.ORIGIN:
        onChange(intersect);
        break;

      case INTERSECTS.POINT:
        if (value) {
          intersect.value = value.split("=");
          onChange(intersect);
        }
        break;

      case INTERSECTS.BREADCRUMBS:
        if (value) {
          intersect.value = value.split(",").map((p) => p.split("="));
          onChange(intersect);
        }
        break;
    }
  }

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="intersect"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Intersect
          </label>

          <select
            name="intersect"
            id="intersect"
            className="w-full py-2 px-4 rounded shadow-sm"
            onChange={(e) => {
              setType(e.target.value);
              setIntersect(e.target.value);
            }}
          >
            <option>-</option>
            {Object.values(INTERSECTS).map((p) => (
              <option value={p} key={p}>
                {p}
              </option>
            ))}
          </select>
        </div>

        <div className="mb-2">
          {type == INTERSECTS.POINT ? (
            <>
              <label
                htmlFor="topic"
                className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
              >
                Point
              </label>
              <input
                id="topic"
                type="text"
                className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
                placeholder="slot=hash"
                onChange={(e) => setIntersect(type, e.target.value)}
              />
            </>
          ) : null}
          {type == INTERSECTS.BREADCRUMBS ? (
            <>
              <label
                htmlFor="breadcrumbs"
                className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
              >
                Points
              </label>
              <input
                id="breadcrumbs"
                type="text"
                className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
                placeholder="slot=hash,slot=hash"
                onChange={(e) => setIntersect(type, e.target.value)}
              />
            </>
          ) : null}
        </div>
      </div>
    </>
  );
}

function DenoStage({ onAdd }) {
  const [mainModule, setMainModule] = useState();
  const [useAsync, setUseAsync] = useState(false);

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="mainModule"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Main Module (JS file path)
          </label>
          <input
            id="mainModule"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="/etc/main.js"
            onChange={(e) => setMainModule(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="useAsync"
            checked={useAsync}
            onChange={(_) => setUseAsync(!useAsync)}
          />
          <label
            htmlFor="useAsync"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Use Async
          </label>
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!mainModule) {
              return;
            }

            const stage = {
              type: "Deno",
              main_module: mainModule,
              use_async: useAsync,
            };

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function SimpleStage({ type, onAdd }) {
  return (
    <div>
      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() =>
            onAdd({
              type,
            })
          }
        >
          Add stage
        </button>
      </div>
    </div>
  );
}

function LegacyV1Stage({ onAdd }) {
  const [includeBlockEndEvents, setIncludeBlockEndEvents] = useState(false);
  const [includeTransactionDetails, setIncludeTransactionDetails] =
    useState(false);
  const [includeTransactionEndEvents, setIncludeTransactionEndEvents] =
    useState(false);
  const [includeBlockDetails, setIncludeBlockDetails] = useState(false);
  const [includeBlockCbor, setIncludeBlockCbor] = useState(false);
  const [includeByronEbb, setIncludeByronEbb] = useState(false);

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <input
            type="checkbox"
            id="includeBlockEndEvents"
            checked={includeBlockEndEvents}
            onChange={(_) => setIncludeBlockEndEvents(!includeBlockEndEvents)}
          />
          <label
            htmlFor="includeBlockEndEvents"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Include Block End Events
          </label>
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="includeTransactionDetails"
            checked={includeTransactionDetails}
            onChange={(_) =>
              setIncludeTransactionDetails(!includeTransactionDetails)
            }
          />
          <label
            htmlFor="includeTransactionDetails"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Include Transaction Details
          </label>
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="includeTransactionEndEvents"
            checked={includeTransactionEndEvents}
            onChange={(_) =>
              setIncludeTransactionEndEvents(!includeTransactionEndEvents)
            }
          />
          <label
            htmlFor="includeTransactionEndEvents"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Include Transaction End Events
          </label>
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="includeBlockDetails"
            checked={includeBlockDetails}
            onChange={(_) => setIncludeBlockDetails(!includeBlockDetails)}
          />
          <label
            htmlFor="includeBlockDetails"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Include Block Details
          </label>
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="includeBlockCbor"
            checked={includeBlockCbor}
            onChange={(_) => setIncludeBlockCbor(!includeBlockCbor)}
          />
          <label
            htmlFor="includeBlockCbor"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Include Block Cbor
          </label>
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="includeByronEbb"
            checked={includeByronEbb}
            onChange={(_) => setIncludeByronEbb(!includeByronEbb)}
          />
          <label
            htmlFor="includeByronEbb"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Include Byron Ebb
          </label>
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            const stage = {
              type: "LegacyV1",
              include_block_end_events: includeBlockEndEvents,
              include_transaction_details: includeTransactionDetails,
              include_transaction_end_events: includeTransactionEndEvents,
              include_block_details: includeBlockDetails,
              include_block_cbor: includeBlockCbor,
              include_byron_ebb: includeByronEbb,
            };

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function RedisStage({ onAdd }) {
  const [url, setUrl] = useState();
  const [streamName, setStreamName] = useState();
  const [streamMaxLength, setStreamMaxLength] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="url"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            URL
          </label>
          <input
            id="url"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="redis://localhost"
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="streamName"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Stream Name
          </label>
          <input
            id="streamName"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setStreamName(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="streamMaxLength"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Stream Max Length
          </label>
          <input
            id="streamMaxLength"
            type="number"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setStreamMaxLength(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!url) {
              return;
            }

            const stage = {
              type: "Redis",
              url,
            };

            if (streamName) {
              stage.stream_name = streamName;
            }

            if (streamMaxLength) {
              stage.stream_max_length = streamMaxLength;
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function AwsLambdaStage({ onAdd }) {
  const [region, setRegion] = useState();
  const [functionName, setFunctionName] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="region"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Region
          </label>
          <input
            id="region"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setRegion(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="functionName"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Function Name
          </label>
          <input
            id="functionName"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setFunctionName(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!region || !functionName) {
              return;
            }

            const stage = {
              type: "AwsLambda",
              region,
              function_name: functionName,
            };

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function AwsS3Stage({ onAdd }) {
  const [region, setRegion] = useState();
  const [bucket, setBucket] = useState();
  const [prefix, setPrefix] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="region"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Region
          </label>
          <input
            id="region"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setRegion(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="bucket"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Bucket Name
          </label>
          <input
            id="bucket"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setBucket(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="prefix"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Prefix
          </label>
          <input
            id="prefix"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setPrefix(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!region || !bucket) {
              return;
            }

            const stage = {
              type: "AwsS3",
              region,
              bucket,
            };

            if (prefix) {
              stage.prefix = prefix;
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function AwsSqsStage({ onAdd }) {
  const [region, setRegion] = useState();
  const [queueUrl, setQueueUrl] = useState();
  const [groupId, setGroupId] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="region"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Region
          </label>
          <input
            id="region"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setRegion(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="queueUrl"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Queue URL
          </label>
          <input
            id="queueUrl"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setQueueUrl(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="groupId"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Group ID
          </label>
          <input
            id="groupId"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setGroupId(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!region || !queueUrl) {
              return;
            }

            const stage = {
              type: "AwsSqs",
              region,
              queue_url: queueUrl,
            };

            if (groupId) {
              stage.group_id = groupId;
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function GcpPubSubStage({ onAdd }) {
  const [topic, setTopic] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="topic"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Topic
          </label>
          <input
            id="topic"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setTopic(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!topic) {
              return;
            }

            const stage = {
              type: "GcpPubSub",
              topic,
            };

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function GcpCloudFunctionStage({ onAdd }) {
  const [url, setUrl] = useState();
  const [timeout, setTimeout] = useState();
  const [headers, setHeaders] = useState();
  const [authentication, setAuthentication] = useState(false);

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="url"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            URL
          </label>
          <input
            id="url"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="timeout"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Timeout
          </label>
          <input
            id="timeout"
            type="number"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setTimeout(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="authentication"
            checked={authentication}
            onChange={(_) => setAuthentication(!authentication)}
          />
          <label
            htmlFor="authentication"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Authentication
          </label>
        </div>

        <div className="mb-2">
          <label
            htmlFor="headers"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Headers
          </label>
          <input
            id="headers"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="key=value,key=value"
            onChange={(e) => setHeaders(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!url) {
              return;
            }

            const stage = {
              type: "GcpCloudFunction",
              url,
            };

            if (timeout) {
              stage.timeout = timeout;
            }

            if (authentication) {
              stage.authentication = authentication;
            }

            if (headers) {
              stage.headers = {};
              headers.split(",").forEach((s) => {
                let values = s.split("=");
                stage.headers[values[0]] = values[1];
              });
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function RabbitmqStage({ onAdd }) {
  const [url, setUrl] = useState();
  const [exchange, setExchange] = useState();
  const [routingKey, setRoutingKey] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="url"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            URL
          </label>
          <input
            id="url"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="amqp://rabbitmq:rabbitmq@localhost:5672"
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="exchange"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Exchange
          </label>
          <input
            id="exchange"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setExchange(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="routingKey"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Routing Key
          </label>
          <input
            id="routingKey"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setRoutingKey(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!url || !exchange) {
              return;
            }

            const stage = {
              type: "Rabbitmq",
              url,
              exchange,
            };

            if (routingKey) {
              stage.routing_key = routingKey;
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function FileRotateStage({ onAdd }) {
  const [outputFormat, setOutputFormat] = useState();
  const [outputPath, setOutputPath] = useState();
  const [maxBytesPerFile, setMaxBytesPerFile] = useState(50 * 1024 * 1024);
  const [maxTotalFiles, setMaxTotalFiles] = useState(200);
  const [compressFiles, setCompressFiles] = useState(false);

  const formats = ["JSONL"];

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="outputFormat"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Output Format
          </label>

          <select
            name="outputFormat"
            id="outputFormat"
            className="w-full py-2 px-4 rounded shadow-sm"
            onChange={(e) => setOutputFormat(e.target.value)}
            value={outputFormat}
          >
            <option>-</option>
            {formats.map((f) => (
              <option value={f} key={f}>
                {f}
              </option>
            ))}
          </select>
        </div>

        <div className="mb-2">
          <label
            htmlFor="outputPath"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Output Path
          </label>
          <input
            id="outputPath"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setOutputPath(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="maxBytesPerFile"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Max Bytes Per File
          </label>
          <input
            id="maxBytesPerFile"
            type="number"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            value={maxBytesPerFile}
            onChange={(e) => setMaxBytesPerFile(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="maxTotalFiles"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Max Total Files
          </label>
          <input
            id="maxTotalFiles"
            type="number"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            value={maxTotalFiles}
            onChange={(e) => setMaxTotalFiles(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="compressFiles"
            checked={compressFiles}
            onChange={(_) => setCompressFiles(!compressFiles)}
          />
          <label
            htmlFor="compressFiles"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Compress Files
          </label>
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            const stage = {
              type: "FileRotate",
            };

            if (outputFormat) {
              stage.output_format = outputFormat;
            }
            if (outputPath) {
              stage.output_path = outputPath;
            }
            if (maxBytesPerFile) {
              stage.max_bytes_per_file = maxBytesPerFile;
            }
            if (maxTotalFiles) {
              stage.max_total_files = maxTotalFiles;
            }
            if (compressFiles) {
              stage.compress_files = compressFiles;
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function ElasticSearchStage({ onAdd }) {
  const [url, setUrl] = useState();
  const [index, setIndex] = useState();
  const [idempotency, setIdempotency] = useState(false);
  const [username, setUsername] = useState();
  const [password, setPassword] = useState();

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="url"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            URL
          </label>
          <input
            id="url"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="http://localhost:9200"
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="index"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Index
          </label>
          <input
            id="index"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setIndex(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="idempotency"
            checked={idempotency}
            onChange={(_) => setIdempotency(!idempotency)}
          />
          <label
            htmlFor="idempotency"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Idempotency
          </label>
        </div>

        <div className="mb-2">
          <label
            htmlFor="username"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Username
          </label>
          <input
            id="username"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setUsername(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="password"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Password
          </label>
          <input
            id="password"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!url || !index) {
              return;
            }

            const stage = {
              type: "ElasticSearch",
              url,
              index,
              idempotency,
            };

            if (username && password) {
              stage.credentials = {
                type: "Basic",
                username,
                password,
              };
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function WebHookStage({ onAdd }) {
  const [url, setUrl] = useState();
  const [timeout, setTimeout] = useState();
  const [headers, setHeaders] = useState();
  const [authorization, setAuthorization] = useState();
  const [allowInvalidCerts, setAllowInvalidCerts] = useState(false);

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="url"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            URL
          </label>
          <input
            id="url"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="timeout"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Timeout
          </label>
          <input
            id="timeout"
            type="number"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setTimeout(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="authorization"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Authorization
          </label>
          <input
            id="authorization"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setAuthorization(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="allowInvalidCerts"
            checked={allowInvalidCerts}
            onChange={(_) => setAllowInvalidCerts(!allowInvalidCerts)}
          />
          <label
            htmlFor="allowInvalidCerts"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Allow Invalid Certs
          </label>
        </div>

        <div className="mb-2">
          <label
            htmlFor="headers"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Headers
          </label>
          <input
            id="headers"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="key=value,key=value"
            onChange={(e) => setHeaders(e.target.value)}
          />
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!url) {
              return;
            }

            const stage = {
              type: "GcpCloudFunction",
              url,
            };

            if (timeout) {
              stage.timeout = timeout;
            }

            if (authorization) {
              stage.authorization = authorization;
            }

            if (headers) {
              stage.headers = {};
              headers.split(",").forEach((s) => {
                let values = s.split("=");
                stage.headers[values[0]] = values[1];
              });
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function KafkaStage({ onAdd }) {
  const [brokers, setBrokers] = useState();
  const [topic, setTopic] = useState();
  const [ackTimeoutSecs, setAckTimeoutSecs] = useState();
  const [paritioning, setParitioning] = useState();

  const partitionStrategy = ["ByBlock", "Random"];

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="brokers"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Brokers
          </label>
          <input
            id="brokers"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            placeholder="localhost:19092,localhost:19092"
            onChange={(e) => setBrokers(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="topic"
            className="after:content-['*'] after:ml-0.5 after:text-red-500 text-sm font-medium text-gray dark:text-gray-200"
          >
            Topic
          </label>
          <input
            id="topic"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setTopic(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="ackTimeoutSecs"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Ack Timeout Secs
          </label>
          <input
            id="ackTimeoutSecs"
            type="number"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setAckTimeoutSecs(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="paritioning"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Paritioning
          </label>

          <select
            name="paritioning"
            id="paritioning"
            className="w-full py-2 px-4 rounded shadow-sm"
            onChange={(e) => setParitioning(e.target.value)}
            value={paritioning}
          >
            <option>-</option>
            {partitionStrategy.map((p) => (
              <option value={p} key={p}>
                {p}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            if (!brokers || !topic) {
              return;
            }

            const stage = {
              type: "Kafka",
              brokers: brokers.split(","),
              topic,
            };

            if (ackTimeoutSecs) {
              stage.ackTimeoutSecs = ackTimeoutSecs;
            }

            if (paritioning) {
              stage.paritioning = paritioning;
            }

            onAdd(stage);
          }}
        >
          Add stage
        </button>
      </div>
    </>
  );
}

function StageCard({ value }) {
  return (
    <div className="border border-gray-500 rounded mt-2 p-2 relative">
      {Object.keys(value).map((k) => (
        <div key={k} className="text-gray dark:text-gray-200">
          {typeof value[k] == "object" && !Array.isArray(value[k]) ? (
            <div>
              <strong className="font-bold">{k}</strong>
              {Object.keys(value[k]).map((x) => (
                <div key={x} className="ms-2">
                  <span className="font-bold">{x}: </span>
                  {value[k][x].toString()}
                </div>
              ))}
            </div>
          ) : (
            <div>
              <span className="font-bold">{k}: </span>
              {value[k].toString()}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
