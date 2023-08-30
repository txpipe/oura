import { Component, useState } from "react";

import stages from "../data/stages.json";

const TYPES = {
  SOURCES: "sources",
  FILTERS: "filters",
  SINKS: "sinks",
};

export function Configuration() {
  const [openedModal, setOpenedModal] = useState(false);
  const [typeModal, setTypeModal] = useState(TYPES.SOURCES);
  const [optionModal, setoOptionModal] = useState(undefined);
  const [tempModal, setTempModal] = useState({});
  const [currentStages, setCurrentStages] = useState({});

  function openModal(type) {
    setOpenedModal(true);
    setTypeModal(type);
    setoOptionModal(undefined);
    setTempModal({});
  }

  function addStage() {
    if (typeModal == TYPES.SOURCES || typeModal == TYPES.SINKS) {
      setCurrentStages(
        Object.assign(currentStages, {
          [`${typeModal}`]: { type: optionModal, ...tempModal },
        })
      );
    } else {
      setCurrentStages(
        Object.assign(currentStages, {
          [`${typeModal}`]: (currentStages[typeModal] || []).concat({
            type: optionModal,
            ...tempModal,
          }),
        })
      );
    }

    setOpenedModal(false);
  }

  return (
    <div>
      <div className="absolute">
        {openedModal ? (
          <>
            <div className="justify-center items-center flex overflow-x-hidden overflow-y-auto fixed inset-0 z-50 outline-none focus:outline-none">
              <div className="relative w-auto my-6 mx-auto max-w-3xl w-full">
                <div className="border-0 rounded-lg shadow-lg relative flex flex-col w-full bg-white dark:bg-gray-700 outline-none focus:outline-none">
                  <div className="flex items-start justify-between p-5 border-b border-solid border-slate-200 rounded-t">
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
                    {optionModal
                      ? stages[typeModal][optionModal]?.fields?.map((f) => (
                          <label className="block mb-2" key={f.key}>
                            <span
                              className={
                                f.required
                                  ? "after:content-['*'] after:ml-0.5 after:text-red-500 block text-sm font-medium text-slate-700"
                                  : "block text-sm font-medium text-slate-700"
                              }
                            >
                              {f.name}
                            </span>
                            <input
                              type="text"
                              name={f.name}
                              className="mt-1 px-3 py-2 bg-white border shadow-sm border-slate-300 focus:outline-none w-full rounded-md sm:text-sm"
                              onChange={(e) =>
                                setTempModal(
                                  Object.assign(tempModal, {
                                    [`${f.key}`]: e.target.value,
                                  })
                                )
                              }
                            />
                          </label>
                        ))
                      : null}
                  </div>

                  <div className="flex items-center justify-end p-6 border-t border-solid border-slate-200 rounded-b">
                    <button
                      className="bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded"
                      onClick={() => addStage()}
                    >
                      Add stage
                    </button>
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
        <div className="grid grid-cols-3 gap-3">
          <div>
            <button
              className="w-full bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded btn_pointer btn_pointer_right"
              onClick={() => openModal(TYPES.SOURCES)}
            >
              add source
            </button>

            {currentStages[TYPES.SOURCES]
              ? StageCard(currentStages[TYPES.SOURCES])
              : null}
          </div>
          <div>
            <button
              className="w-full bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded btn_pointer btn_pointer_left btn_pointer_right"
              onClick={() => openModal(TYPES.FILTERS)}
            >
              add filter
            </button>
            {currentStages[TYPES.FILTERS]?.map((filter, index) => (
              <div key={index}>{StageCard(filter)}</div>
            ))}
          </div>

          <div>
            <button
              className="w-full bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded btn_pointer btn_pointer_left"
              onClick={() => openModal(TYPES.SINKS)}
            >
              add sink
            </button>

            {currentStages[TYPES.SINKS]
              ? StageCard(currentStages[TYPES.SINKS])
              : null}
          </div>
        </div>
      </div>
    </div>
  );
}

function StageCard(value) {
  return (
    <div className="border rounded mt-2 p-2">
      {Object.keys(value).map((k) => (
        <div key={k}>
          <strong>{k}: </strong>
          {value[k]}
        </div>
      ))}
    </div>
  );
}
