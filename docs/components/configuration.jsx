import { useState, useReducer } from "react";
import json2toml from "json2toml";

function reducer(state, action) {
  switch (action.name) {

    case 'add_source': {
      const currentStages = state.currentStages;
      currentStages.source = action.source;
      currentStages.intersect = action.intersect;

      const _tomlContent = json2toml(currentStages, { newlineAfterSection: true });
      const tomlContent = (_tomlContent || "").trim();

      return {
        ...state,
        currentStages,
        tomlContent,
        openedModal: false,
      };
    }

    case 'add_filter': {
      const currentStages = state.currentStages;

      const hasFilterIndex = currentStages.filters?.findIndex(
        (s) => s.type == action.stage.type
      );

      if (hasFilterIndex != undefined && hasFilterIndex != -1) {
        currentStages.filters[hasFilterIndex] = action.stage;
      } else {
        currentStages.filters = currentStages.filters?.concat(action.stage) || [action.stage];
      }

      const _tomlContent = json2toml(currentStages, { newlineAfterSection: true });
      const tomlContent = (_tomlContent || "").trim();

      return {
        ...state,
        currentStages,
        tomlContent,
        openedModal: false
      };
    }

    case 'add_sink': {
      const currentStages = state.currentStages;
      currentStages.sink = action.stage;

      const _tomlContent = json2toml(currentStages, { newlineAfterSection: true });
      const tomlContent = (_tomlContent || "").trim();

      return {
        ...state,
        currentStages,
        openedModal: false,
        tomlContent
      };
    }

    case 'open_modal': {
      return {
        ...state,
        openedModal: true,
        typeModal: action.type,
        optionModal: undefined
      };
    };

    case 'close_modal': {
      return {
        ...state,
        openedModal: false
      };
    };

    case 'set_option_modal': {
      return {
        ...state,
        optionModal: action.value
      };
    };

    case 'reset': {
      return {
        ...state,
        currentStages: {},
        tomlContent: ""
      };
    };
  }
  throw Error('Unknown action: ' + action.name);
}

export function Configuration() {

  const [state, dispatch] = useReducer(reducer,
    {
      openedModal: false,
      typeModal: null,
      optionModal: undefined,
      currentStages: {},
      tomlContent: ""
    }
  );

  const addSourceStage = ({ source, intersect }) => {
    dispatch({ name: 'add_source', source, intersect });
  };

  const addFilterStage = (stage) => {
    dispatch({ name: 'add_filter', stage });
  };

  const addSinkStage = (stage) => {
    dispatch({ name: 'add_sink', stage });
  };

  function openModal(type) {
    dispatch({ name: 'open_modal', type });
  }

  function closeModal() {
    dispatch({ name: 'close_modal' });
  }

  function setOptionModal(value) {
    dispatch({ name: 'set_option_modal', value });
  }

  function reset() {
    dispatch({ name: 'reset' });
  }

  function copyToClipboard(button) {
    navigator.clipboard.writeText(state.tomlContent).then(res => {

      button.classList.remove("bg-gray-500");
      button.classList.remove("hover:bg-gray-700");
      button.classList.add("bg-lime-800");

      const buttonContent = button.innerHTML;
      button.innerHTML = "copied!";

      setTimeout(() => {
        button.classList.remove("bg-lime-800");
        button.classList.add("bg-gray-500");
        button.classList.add("hover:bg-gray-700");
        button.innerHTML = buttonContent;
      }, 1000);
    });
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
      Terminal: <TerminalStage onAdd={addSinkStage} />,
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
        {state.openedModal ? (
          <>
            <div className="flex items-center overflow-y-auto fixed inset-0 z-50">
              <div className="my-6 mx-auto max-w-3xl w-full">
                <div className="rounded-lg shadow-lg bg-white dark:bg-gray-700">
                  <div className="flex justify-between p-5">
                    <h3 className="text-3xl font-semibold capitalize">
                      {state.typeModal}
                    </h3>
                    <button
                      className="p-1 float-right text-3xl leading-none"
                      onClick={closeModal}
                    >
                      <span className="text-gray dark:text-gray-200">×</span>
                    </button>
                  </div>

                  <div className="relative p-6 flex-auto">
                    <select
                      name="stage"
                      id="stage"
                      className="w-full py-2 px-4 rounded mb-2"
                      onChange={(e) => setOptionModal(e.target.value)}
                      value={state.optionModal}
                    >
                      <option>-</option>
                      {Object.keys(stages[state.typeModal]).map((k) => (
                        <option value={k} key={k}>
                          {k}
                        </option>
                      ))}
                    </select>
                    {state.optionModal ? stages[state.typeModal][state.optionModal] : null}
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
            className="px-4 py-2 rounded font-bold me-2 text-red-500 bg-red-100 hover:bg-red-200"
            onClick={reset}
          >
            reset
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

            {state.currentStages.source ? (
              <StageCard
                value={{
                  ...state.currentStages.source,
                  intersect: state.currentStages.intersect,
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
            {state.currentStages[TYPES.FILTERS]?.map((value, index) => (
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

            {state.currentStages.sink ? (
              <StageCard value={state.currentStages.sink} />
            ) : null}
          </div>
        </div>
      </div>

      {state.tomlContent && state.tomlContent.length > 0 && (
        <div className="p-2">
          <div className="flex justify-between">
            <small className="mt-4">config.toml</small>
            <small>
              <button
                className="bg-gray-500 hover:bg-gray-700 dark:text-gray-200 text-white font-bold rounded w-36 h-10"
                onClick={(e) => copyToClipboard(e.target)}>
                copy to clipboard
              </button>
            </small>
          </div>
          <div className="py-5 mt-2 bg-slate-100 rounded-md dark:bg-gray-800 dark:text-gray-400">
            <pre>{state.tomlContent}</pre>
          </div>
        </div>
      )}
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

function TerminalStage({ onAdd }) {
  const [throttleMinSpanMillis, setThrottleMinSpanMillis] = useState();
  const [adahandlePolicy, setAdahandlePolicy] = useState();
  const [wrap, setWrap] = useState(false);

  return (
    <>
      <div className="mb-5">
        <div className="mb-2">
          <label
            htmlFor="throttleMinSpanMillis"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Throttle Min Span Millis
          </label>
          <input
            id="throttleMinSpanMillis"
            type="number"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setThrottleMinSpanMillis(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <label
            htmlFor="adahandlePolicy"
            className="text-sm font-medium text-gray dark:text-gray-200"
          >
            Ada Handle Policy
          </label>
          <input
            id="adahandlePolicy"
            type="text"
            className="w-full mt-1 px-3 py-2 focus:outline-none rounded-md sm:text-sm border shadow-sm border-slate-300 dark:border-none"
            onChange={(e) => setAdahandlePolicy(e.target.value)}
          />
        </div>

        <div className="mb-2">
          <input
            type="checkbox"
            id="wrap"
            checked={wrap}
            onChange={(_) => setWrap(!wrap)}
          />
          <label
            htmlFor="wrap"
            className="text-sm font-medium text-gray dark:text-gray-200 ms-3"
          >
            Wrap
          </label>
        </div>
      </div>

      <div className="flex items-center justify-end ">
        <button
          className="bg-gray-500 text-white dark:text-gray-200 font-bold py-2 px-4 rounded"
          onClick={() => {
            const stage = {
              type: "Terminal",
            };

            if (throttleMinSpanMillis) {
              stage.throttle_min_span_millis = throttleMinSpanMillis;
            }
            if (adahandlePolicy) {
              stage.wrap = adahandlePolicy;
            }
            if (wrap) {
              stage.adahandle_policy = wrap;
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